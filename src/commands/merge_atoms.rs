use crate::types::{load_atom_file, Atom, MergedAtomEnvelope, Tool};
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
            Ok((atoms, input_provenance)) => {
                println!("    {} atoms loaded ({} provenance entries)", atoms.len(), input_provenance.len());
                maps.push(atoms);
                provenance.extend(input_provenance);
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

    #[test]
    fn test_recursive_merge_flattens_provenance() {
        use crate::types::{load_atom_file, MergedAtomEnvelope, Tool};
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();

        // --- Write two single-source atom files (A and B) ---
        let file_a = dir.path().join("a.json");
        let file_b = dir.path().join("b.json");
        let atom_a = make_real_atom("foo", "src/a.rs", "rust", "exec");
        let atom_b = make_real_atom("bar", "src/b.rs", "lean", "def");

        let mut data_a = BTreeMap::new();
        data_a.insert("probe:a/1.0/mod/foo()".to_string(), atom_a);

        let mut data_b = BTreeMap::new();
        data_b.insert("probe:b/1.0/mod/bar()".to_string(), atom_b);

        let envelope_a = serde_json::json!({
            "schema": "verus-analyzer/atoms",
            "schema-version": "2.0",
            "tool": {"name": "probe", "version": "0.1.0", "command": "extract"},
            "source": {"repo": "repo-a", "commit": "aaa", "language": "rust", "package": "pkg-a", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": data_a
        });
        let envelope_b = serde_json::json!({
            "schema": "lean-analyzer/atoms",
            "schema-version": "2.0",
            "tool": {"name": "probe", "version": "0.1.0", "command": "extract"},
            "source": {"repo": "repo-b", "commit": "bbb", "language": "lean", "package": "pkg-b", "package-version": "2.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": data_b
        });

        std::fs::File::create(&file_a).unwrap().write_all(serde_json::to_string_pretty(&envelope_a).unwrap().as_bytes()).unwrap();
        std::fs::File::create(&file_b).unwrap().write_all(serde_json::to_string_pretty(&envelope_b).unwrap().as_bytes()).unwrap();

        // --- Load A and B, verify provenance ---
        let (atoms_a, prov_a) = load_atom_file(&file_a).unwrap();
        let (atoms_b, prov_b) = load_atom_file(&file_b).unwrap();
        assert_eq!(prov_a.len(), 1);
        assert_eq!(prov_a[0].source.package, "pkg-a");
        assert_eq!(prov_b.len(), 1);
        assert_eq!(prov_b[0].source.package, "pkg-b");

        // --- Simulate first merge: A + B → merged file ---
        let (merged_data, _stats) = merge_atom_maps(vec![atoms_a, atoms_b]);

        let mut all_prov = Vec::new();
        all_prov.extend(prov_a);
        all_prov.extend(prov_b);

        let merged_envelope = MergedAtomEnvelope {
            schema: "probe/merged-atoms".to_string(),
            schema_version: "2.0".to_string(),
            tool: Tool { name: "probe".to_string(), version: "0.1.0".to_string(), command: "merge-atoms".to_string() },
            inputs: all_prov,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            data: merged_data,
        };

        let merged_file = dir.path().join("merged_ab.json");
        std::fs::File::create(&merged_file).unwrap().write_all(serde_json::to_string_pretty(&merged_envelope).unwrap().as_bytes()).unwrap();

        // --- Write a third single-source file (C) ---
        let file_c = dir.path().join("c.json");
        let atom_c = make_real_atom("baz", "src/c.rs", "rust", "exec");
        let mut data_c = BTreeMap::new();
        data_c.insert("probe:c/1.0/mod/baz()".to_string(), atom_c);

        let envelope_c = serde_json::json!({
            "schema": "verus-analyzer/atoms",
            "schema-version": "2.0",
            "tool": {"name": "probe", "version": "0.1.0", "command": "extract"},
            "source": {"repo": "repo-c", "commit": "ccc", "language": "rust", "package": "pkg-c", "package-version": "3.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": data_c
        });
        std::fs::File::create(&file_c).unwrap().write_all(serde_json::to_string_pretty(&envelope_c).unwrap().as_bytes()).unwrap();

        // --- Load merged_ab and C, verify flattened provenance ---
        let (atoms_merged, prov_merged) = load_atom_file(&merged_file).unwrap();
        let (atoms_c, prov_c) = load_atom_file(&file_c).unwrap();

        // The merged file should yield 2 provenance entries (A and B), not 1.
        assert_eq!(prov_merged.len(), 2, "merged file provenance should be flattened");
        assert_eq!(prov_c.len(), 1);

        let packages: Vec<&str> = prov_merged.iter().map(|p| p.source.package.as_str()).collect();
        assert!(packages.contains(&"pkg-a"));
        assert!(packages.contains(&"pkg-b"));
        assert_eq!(prov_c[0].source.package, "pkg-c");

        // Simulate second merge: merged_ab + C
        let (final_data, _) = merge_atom_maps(vec![atoms_merged, atoms_c]);
        let mut final_prov = Vec::new();
        final_prov.extend(prov_merged);
        final_prov.extend(prov_c);

        assert_eq!(final_data.len(), 3);
        assert_eq!(final_prov.len(), 3, "final provenance should have all 3 original sources");

        let final_packages: Vec<&str> = final_prov.iter().map(|p| p.source.package.as_str()).collect();
        assert!(final_packages.contains(&"pkg-a"));
        assert!(final_packages.contains(&"pkg-b"));
        assert!(final_packages.contains(&"pkg-c"));
    }
}
