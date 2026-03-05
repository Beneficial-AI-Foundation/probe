use crate::types::{load_atom_file, Atom, InputProvenance, MergedAtomEnvelope, Source, Tool};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Strip trailing `.` from a code-name (legacy verus-analyzer artifact).
fn normalize_code_name(name: &str) -> String {
    name.strip_suffix('.').unwrap_or(name).to_string()
}

/// Merge statistics reported after the operation.
pub struct MergeStats {
    pub total_atoms: usize,
    pub stubs_replaced: usize,
    pub stubs_remaining: usize,
    pub atoms_added: usize,
    pub keys_normalized: usize,
    pub conflicts: usize,
}

/// Normalize all keys and dependency references in an atom map.
/// Returns the normalized map and a count of keys that changed.
fn normalize_atoms(atoms: BTreeMap<String, Atom>) -> (BTreeMap<String, Atom>, usize) {
    let mut out: BTreeMap<String, Atom> = BTreeMap::new();
    let mut changed = 0;

    for (key, mut atom) in atoms {
        let norm_key = normalize_code_name(&key);
        if norm_key != key {
            changed += 1;
        }

        atom.dependencies = atom
            .dependencies
            .into_iter()
            .map(|d| normalize_code_name(&d))
            .collect();

        // Normalize code-name references inside dependencies-with-locations if present.
        if let Some(dwl) = atom.extensions.get_mut("dependencies-with-locations") {
            if let Some(arr) = dwl.as_array_mut() {
                for entry in arr {
                    if let Some(cn) = entry.get("code-name").and_then(|v| v.as_str()) {
                        let norm = normalize_code_name(cn);
                        entry
                            .as_object_mut()
                            .unwrap()
                            .insert("code-name".to_string(), serde_json::Value::String(norm));
                    }
                }
            }
        }

        match out.get(&norm_key) {
            Some(existing) if existing.is_stub() && !atom.is_stub() => {
                out.insert(norm_key, atom);
            }
            Some(_) => {} // keep existing on collision
            None => {
                out.insert(norm_key, atom);
            }
        }
    }

    (out, changed)
}

/// Merge multiple atom maps into one.
///
/// The first map is the base. For each subsequent map:
/// - Stubs in the base are replaced by real atoms from the incoming map.
/// - New atoms (not in base) are added.
/// - Real-vs-real conflicts keep the base version (first wins).
pub fn merge_atom_maps(maps: Vec<BTreeMap<String, Atom>>) -> (BTreeMap<String, Atom>, MergeStats) {
    let mut stats = MergeStats {
        total_atoms: 0,
        stubs_replaced: 0,
        stubs_remaining: 0,
        atoms_added: 0,
        keys_normalized: 0,
        conflicts: 0,
    };

    let mut maps_iter = maps.into_iter();
    let first = maps_iter.next().unwrap_or_default();
    let (mut base, norm_count) = normalize_atoms(first);
    stats.keys_normalized += norm_count;

    for incoming in maps_iter {
        let (incoming, norm_count) = normalize_atoms(incoming);
        stats.keys_normalized += norm_count;

        for (key, incoming_atom) in incoming {
            match base.get(&key) {
                Some(existing) if existing.is_stub() && !incoming_atom.is_stub() => {
                    base.insert(key, incoming_atom);
                    stats.stubs_replaced += 1;
                }
                Some(existing) if !existing.is_stub() && !incoming_atom.is_stub() => {
                    stats.conflicts += 1;
                    eprintln!(
                        "  Warning: conflict for '{}' (keeping base version from {})",
                        key, existing.code_path
                    );
                }
                Some(_) => {}
                None => {
                    base.insert(key, incoming_atom);
                    stats.atoms_added += 1;
                }
            }
        }
    }

    stats.stubs_remaining = base.values().filter(|a| a.is_stub()).count();
    stats.total_atoms = base.len();

    (base, stats)
}

/// Execute the merge-atoms command.
pub fn cmd_merge_atoms(inputs: Vec<PathBuf>, output: PathBuf) {
    if inputs.len() < 2 {
        eprintln!("Error: merge-atoms requires at least 2 input files");
        std::process::exit(1);
    }

    let mut maps = Vec::new();
    let mut provenance = Vec::new();

    for path in &inputs {
        println!("  Loading {}...", path.display());
        match load_atom_file(path) {
            Ok((atoms, schema, source)) => {
                println!("    {} atoms loaded", atoms.len());
                maps.push(atoms);
                provenance.push(InputProvenance {
                    schema,
                    source: source.unwrap_or_else(|| Source {
                        repo: String::new(),
                        commit: String::new(),
                        language: String::new(),
                        package: path.file_stem().map_or_else(
                            || "unknown".to_string(),
                            |s| s.to_string_lossy().to_string(),
                        ),
                        package_version: String::new(),
                    }),
                });
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
    println!();

    println!("Merging {} files...", inputs.len());
    let (merged, stats) = merge_atom_maps(maps);

    let envelope = MergedAtomEnvelope {
        schema: "probe/merged-atoms".to_string(),
        schema_version: "2.0".to_string(),
        tool: Tool {
            name: "probe".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: "merge-atoms".to_string(),
        },
        inputs: provenance,
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        data: merged,
    };

    let json = serde_json::to_string_pretty(&envelope).expect("Failed to serialize JSON");
    std::fs::write(&output, &json).expect("Failed to write output file");

    println!();
    println!("Output: {}", output.display());
    println!("  Total atoms:      {}", stats.total_atoms);
    println!("  Stubs replaced:   {}", stats.stubs_replaced);
    println!("  Stubs remaining:  {}", stats.stubs_remaining);
    println!("  New atoms added:  {}", stats.atoms_added);
    if stats.keys_normalized > 0 {
        println!("  Keys normalized:  {}", stats.keys_normalized);
    }
    if stats.conflicts > 0 {
        println!("  Conflicts (kept base): {}", stats.conflicts);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CodeText;

    fn make_real_atom(name: &str, code_path: &str, language: &str, kind: &str) -> Atom {
        Atom {
            display_name: name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: code_path.to_string(),
            code_text: CodeText {
                lines_start: 10,
                lines_end: 20,
            },
            kind: kind.to_string(),
            language: language.to_string(),
            extensions: BTreeMap::new(),
        }
    }

    fn make_stub(name: &str, language: &str) -> Atom {
        Atom {
            display_name: name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: String::new(),
            code_text: CodeText {
                lines_start: 0,
                lines_end: 0,
            },
            kind: "exec".to_string(),
            language: language.to_string(),
            extensions: BTreeMap::new(),
        }
    }

    use std::collections::BTreeSet;

    #[test]
    fn test_stub_replaced_by_real() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/helper()".to_string(),
            make_stub("helper", "rust"),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/helper()".to_string(),
            make_real_atom("helper", "src/lib.rs", "rust", "exec"),
        );

        let (merged, stats) = merge_atom_maps(vec![base, incoming]);

        assert_eq!(stats.stubs_replaced, 1);
        assert_eq!(stats.stubs_remaining, 0);
        assert_eq!(merged["probe:a/1.0/mod/helper()"].code_path, "src/lib.rs");
    }

    #[test]
    fn test_new_atoms_added() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/foo()".to_string(),
            make_real_atom("foo", "src/a.rs", "rust", "exec"),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:b/1.0/mod/bar()".to_string(),
            make_real_atom("bar", "src/b.rs", "rust", "exec"),
        );

        let (merged, stats) = merge_atom_maps(vec![base, incoming]);

        assert_eq!(stats.atoms_added, 1);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_real_vs_real_conflict_keeps_base() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f()".to_string(),
            make_real_atom("f", "src/base.rs", "rust", "exec"),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/f()".to_string(),
            make_real_atom("f", "src/other.rs", "rust", "exec"),
        );

        let (merged, stats) = merge_atom_maps(vec![base, incoming]);

        assert_eq!(stats.conflicts, 1);
        assert_eq!(merged["probe:a/1.0/mod/f()"].code_path, "src/base.rs");
    }

    #[test]
    fn test_trailing_dot_normalization() {
        let mut base = BTreeMap::new();
        base.insert("probe:a/1.0/mod/f().".to_string(), make_stub("f", "rust"));

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/f()".to_string(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );

        let (merged, stats) = merge_atom_maps(vec![base, incoming]);

        assert_eq!(stats.keys_normalized, 1);
        assert_eq!(stats.stubs_replaced, 1);
        assert!(merged.contains_key("probe:a/1.0/mod/f()"));
        assert!(!merged.contains_key("probe:a/1.0/mod/f()."));
    }

    #[test]
    fn test_cross_language_merge() {
        let mut rust_atoms = BTreeMap::new();
        rust_atoms.insert(
            "probe:dalek/4.1.3/scalar/add()".to_string(),
            make_real_atom("add", "src/scalar.rs", "rust", "exec"),
        );

        let mut lean_atoms = BTreeMap::new();
        lean_atoms.insert(
            "probe:Curve25519Dalek.Scalar.add".to_string(),
            make_real_atom("add", "Curve25519Dalek/Scalar.lean", "lean", "def"),
        );

        let (merged, stats) = merge_atom_maps(vec![rust_atoms, lean_atoms]);

        assert_eq!(stats.atoms_added, 1);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged["probe:dalek/4.1.3/scalar/add()"].language, "rust");
        assert_eq!(merged["probe:Curve25519Dalek.Scalar.add"].language, "lean");
    }

    #[test]
    fn test_is_stub() {
        let stub = make_stub("f", "rust");
        assert!(stub.is_stub());

        let real = make_real_atom("f", "src/lib.rs", "rust", "exec");
        assert!(!real.is_stub());
    }

    #[test]
    fn test_extensions_preserved() {
        let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
        atom.extensions.insert(
            "dependencies-with-locations".to_string(),
            serde_json::json!([{"code-name": "probe:a/1.0/g()", "location": "inner", "line": 42}]),
        );

        let mut base = BTreeMap::new();
        base.insert("probe:a/1.0/f()".to_string(), atom);

        let (merged, _) = merge_atom_maps(vec![base]);

        let f = &merged["probe:a/1.0/f()"];
        assert!(f.extensions.contains_key("dependencies-with-locations"));
    }
}
