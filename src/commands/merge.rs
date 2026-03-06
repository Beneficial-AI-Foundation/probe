use crate::types::{
    load_envelope, Atom, MergedAtomEnvelope, MergedGenericEnvelope, SchemaCategory, Tool,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Strip trailing `.` from a code-name (legacy verus-analyzer artifact).
fn normalize_code_name(name: &str) -> String {
    name.strip_suffix('.').unwrap_or(name).to_string()
}

/// Merge statistics reported after the operation.
pub struct MergeStats {
    pub total_entries: usize,
    pub stubs_replaced: usize,
    pub stubs_remaining: usize,
    pub entries_added: usize,
    pub keys_normalized: usize,
    pub conflicts: usize,
}

// ---------------------------------------------------------------------------
// Atom-specific merge (stub replacement, first-wins)
// ---------------------------------------------------------------------------

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
            Some(_) => {}
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
        total_entries: 0,
        stubs_replaced: 0,
        stubs_remaining: 0,
        entries_added: 0,
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
                    stats.entries_added += 1;
                }
            }
        }
    }

    stats.stubs_remaining = base.values().filter(|a| a.is_stub()).count();
    stats.total_entries = base.len();

    (base, stats)
}

// ---------------------------------------------------------------------------
// Generic merge for specs/proofs (last-wins, no stubs)
// ---------------------------------------------------------------------------

/// Normalize keys in a generic data map. Only normalizes the dictionary keys
/// (trailing-dot stripping); values are passed through untouched.
fn normalize_generic(
    data: BTreeMap<String, serde_json::Value>,
) -> (BTreeMap<String, serde_json::Value>, usize) {
    let mut out: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut changed = 0;

    for (key, value) in data {
        let norm_key = normalize_code_name(&key);
        if norm_key != key {
            changed += 1;
        }
        out.insert(norm_key, value);
    }

    (out, changed)
}

/// Merge multiple generic data maps into one (for specs and proofs).
///
/// Uses **last-wins** semantics: when the same code-name appears in multiple
/// inputs, the later one replaces the earlier one. This is appropriate for
/// specs/proofs where re-running a tool should override stale results.
pub fn merge_generic_maps(
    maps: Vec<BTreeMap<String, serde_json::Value>>,
) -> (BTreeMap<String, serde_json::Value>, MergeStats) {
    let mut stats = MergeStats {
        total_entries: 0,
        stubs_replaced: 0,
        stubs_remaining: 0,
        entries_added: 0,
        keys_normalized: 0,
        conflicts: 0,
    };

    let mut maps_iter = maps.into_iter();
    let first = maps_iter.next().unwrap_or_default();
    let (mut base, norm_count) = normalize_generic(first);
    stats.keys_normalized += norm_count;

    for incoming in maps_iter {
        let (incoming, norm_count) = normalize_generic(incoming);
        stats.keys_normalized += norm_count;

        for (key, value) in incoming {
            if base.contains_key(&key) {
                stats.conflicts += 1;
            } else {
                stats.entries_added += 1;
            }
            base.insert(key, value);
        }
    }

    stats.total_entries = base.len();
    (base, stats)
}

// ---------------------------------------------------------------------------
// Unified merge command
// ---------------------------------------------------------------------------

/// Execute the `merge` command, auto-detecting the schema category.
pub fn cmd_merge(inputs: Vec<PathBuf>, output: PathBuf) {
    if inputs.len() < 2 {
        eprintln!("Error: merge requires at least 2 input files");
        std::process::exit(1);
    }

    let mut envelopes = Vec::new();

    for path in &inputs {
        println!("  Loading {}...", path.display());
        match load_envelope(path) {
            Ok(meta) => {
                println!(
                    "    schema: \"{}\" ({}), {} provenance entries",
                    meta.schema,
                    meta.category,
                    meta.provenance.len()
                );
                envelopes.push(meta);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }

    let category = envelopes[0].category;
    for (i, meta) in envelopes.iter().enumerate().skip(1) {
        if meta.category != category {
            eprintln!(
                "Error: category mismatch -- {} is {} but {} is {}. All inputs must be the same category.",
                inputs[0].display(), category,
                inputs[i].display(), meta.category,
            );
            std::process::exit(1);
        }
    }

    let mut provenance = Vec::new();
    for meta in &envelopes {
        provenance.extend(meta.provenance.clone());
    }

    println!();
    println!("Merging {} {} files...", inputs.len(), category);

    let tool = Tool {
        name: "probe".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: "merge".to_string(),
    };
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let merged_schema = category.merged_schema().to_string();

    match category {
        SchemaCategory::Atoms => {
            let maps: Result<Vec<BTreeMap<String, Atom>>, String> = envelopes
                .into_iter()
                .enumerate()
                .map(|(i, meta)| {
                    serde_json::from_value(meta.data_value).map_err(|e| {
                        format!("{}: failed to deserialize atoms: {e}", inputs[i].display())
                    })
                })
                .collect();
            let maps = match maps {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };

            let (merged, stats) = merge_atom_maps(maps);

            let envelope = MergedAtomEnvelope {
                schema: merged_schema,
                schema_version: "2.0".to_string(),
                tool,
                inputs: provenance,
                timestamp,
                data: merged,
            };

            let json = serde_json::to_string_pretty(&envelope).expect("Failed to serialize JSON");
            std::fs::write(&output, &json).expect("Failed to write output file");

            print_stats(&output, &stats);
        }
        SchemaCategory::Specs | SchemaCategory::Proofs => {
            let maps: Result<Vec<BTreeMap<String, serde_json::Value>>, String> = envelopes
                .into_iter()
                .enumerate()
                .map(|(i, meta)| {
                    serde_json::from_value(meta.data_value).map_err(|e| {
                        format!("{}: failed to deserialize data: {e}", inputs[i].display())
                    })
                })
                .collect();
            let maps = match maps {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };

            let (merged, stats) = merge_generic_maps(maps);

            let envelope = MergedGenericEnvelope {
                schema: merged_schema,
                schema_version: "2.0".to_string(),
                tool,
                inputs: provenance,
                timestamp,
                data: merged,
            };

            let json = serde_json::to_string_pretty(&envelope).expect("Failed to serialize JSON");
            std::fs::write(&output, &json).expect("Failed to write output file");

            print_stats(&output, &stats);
        }
    }
}

fn print_stats(output: &std::path::Path, stats: &MergeStats) {
    println!();
    println!("Output: {}", output.display());
    println!("  Total entries:    {}", stats.total_entries);
    if stats.stubs_replaced > 0 {
        println!("  Stubs replaced:   {}", stats.stubs_replaced);
    }
    if stats.stubs_remaining > 0 {
        println!("  Stubs remaining:  {}", stats.stubs_remaining);
    }
    println!("  New entries added: {}", stats.entries_added);
    if stats.keys_normalized > 0 {
        println!("  Keys normalized:  {}", stats.keys_normalized);
    }
    if stats.conflicts > 0 {
        println!("  Conflicts:        {}", stats.conflicts);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CodeText;
    use std::collections::BTreeSet;

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

    // -----------------------------------------------------------------------
    // Atom merge tests
    // -----------------------------------------------------------------------

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

        assert_eq!(stats.entries_added, 1);
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

        assert_eq!(stats.entries_added, 1);
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

        std::fs::File::create(&file_a)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_a)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        std::fs::File::create(&file_b)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_b)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let (atoms_a, prov_a) = load_atom_file(&file_a).unwrap();
        let (atoms_b, prov_b) = load_atom_file(&file_b).unwrap();
        assert_eq!(prov_a.len(), 1);
        assert_eq!(prov_a[0].source.package, "pkg-a");
        assert_eq!(prov_b.len(), 1);
        assert_eq!(prov_b[0].source.package, "pkg-b");

        let (merged_data, _stats) = merge_atom_maps(vec![atoms_a, atoms_b]);

        let mut all_prov = Vec::new();
        all_prov.extend(prov_a);
        all_prov.extend(prov_b);

        let merged_envelope = MergedAtomEnvelope {
            schema: "probe/merged-atoms".to_string(),
            schema_version: "2.0".to_string(),
            tool: Tool {
                name: "probe".to_string(),
                version: "0.1.0".to_string(),
                command: "merge".to_string(),
            },
            inputs: all_prov,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            data: merged_data,
        };

        let merged_file = dir.path().join("merged_ab.json");
        std::fs::File::create(&merged_file)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&merged_envelope)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

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
        std::fs::File::create(&file_c)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_c)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let (atoms_merged, prov_merged) = load_atom_file(&merged_file).unwrap();
        let (atoms_c, prov_c) = load_atom_file(&file_c).unwrap();

        assert_eq!(
            prov_merged.len(),
            2,
            "merged file provenance should be flattened"
        );
        assert_eq!(prov_c.len(), 1);

        let packages: Vec<&str> = prov_merged
            .iter()
            .map(|p| p.source.package.as_str())
            .collect();
        assert!(packages.contains(&"pkg-a"));
        assert!(packages.contains(&"pkg-b"));
        assert_eq!(prov_c[0].source.package, "pkg-c");

        let (final_data, _) = merge_atom_maps(vec![atoms_merged, atoms_c]);
        let mut final_prov = Vec::new();
        final_prov.extend(prov_merged);
        final_prov.extend(prov_c);

        assert_eq!(final_data.len(), 3);
        assert_eq!(
            final_prov.len(),
            3,
            "final provenance should have all 3 original sources"
        );

        let final_packages: Vec<&str> = final_prov
            .iter()
            .map(|p| p.source.package.as_str())
            .collect();
        assert!(final_packages.contains(&"pkg-a"));
        assert!(final_packages.contains(&"pkg-b"));
        assert!(final_packages.contains(&"pkg-c"));
    }

    // -----------------------------------------------------------------------
    // Generic merge tests (specs/proofs)
    // -----------------------------------------------------------------------

    #[test]
    fn test_generic_last_wins_on_conflict() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f()".to_string(),
            serde_json::json!({"verified": false, "status": "failure"}),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/f()".to_string(),
            serde_json::json!({"verified": true, "status": "success"}),
        );

        let (merged, stats) = merge_generic_maps(vec![base, incoming]);

        assert_eq!(stats.conflicts, 1);
        assert_eq!(merged["probe:a/1.0/mod/f()"]["verified"], true);
        assert_eq!(merged["probe:a/1.0/mod/f()"]["status"], "success");
    }

    #[test]
    fn test_generic_new_entries_added() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f()".to_string(),
            serde_json::json!({"specified": true}),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:b/1.0/mod/g()".to_string(),
            serde_json::json!({"specified": false}),
        );

        let (merged, stats) = merge_generic_maps(vec![base, incoming]);

        assert_eq!(stats.entries_added, 1);
        assert_eq!(stats.conflicts, 0);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_generic_trailing_dot_normalization() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f().".to_string(),
            serde_json::json!({"specified": true}),
        );

        let (normalized, stats) = merge_generic_maps(vec![base]);

        assert_eq!(stats.keys_normalized, 1);
        assert!(normalized.contains_key("probe:a/1.0/mod/f()"));
        assert!(!normalized.contains_key("probe:a/1.0/mod/f()."));
    }

    #[test]
    fn test_generic_recursive_merge_flattens_provenance() {
        use crate::types::{load_generic_file, MergedGenericEnvelope, SchemaCategory, Tool};
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();

        let file_a = dir.path().join("specs_a.json");
        let file_b = dir.path().join("specs_b.json");

        let envelope_a = serde_json::json!({
            "schema": "probe-verus/specs",
            "schema-version": "2.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "specify"},
            "source": {"repo": "repo-a", "commit": "aaa", "language": "rust", "package": "pkg-a", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {
                "probe:a/1.0/mod/f()": {"specified": true, "has_requires": true, "has_ensures": false}
            }
        });
        let envelope_b = serde_json::json!({
            "schema": "probe-lean/specs",
            "schema-version": "2.0",
            "tool": {"name": "probe-lean", "version": "1.0.0", "command": "specify"},
            "source": {"repo": "repo-b", "commit": "bbb", "language": "lean", "package": "pkg-b", "package-version": "2.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {
                "probe:b/1.0/mod/g()": {"specified": false}
            }
        });

        std::fs::File::create(&file_a)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_a)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        std::fs::File::create(&file_b)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_b)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let (data_a, prov_a, cat_a) = load_generic_file(&file_a).unwrap();
        let (data_b, prov_b, cat_b) = load_generic_file(&file_b).unwrap();
        assert_eq!(cat_a, SchemaCategory::Specs);
        assert_eq!(cat_b, SchemaCategory::Specs);

        let (merged_data, _) = merge_generic_maps(vec![data_a, data_b]);

        let mut all_prov = Vec::new();
        all_prov.extend(prov_a);
        all_prov.extend(prov_b);

        let merged_envelope = MergedGenericEnvelope {
            schema: "probe/merged-specs".to_string(),
            schema_version: "2.0".to_string(),
            tool: Tool {
                name: "probe".to_string(),
                version: "0.1.0".to_string(),
                command: "merge".to_string(),
            },
            inputs: all_prov,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            data: merged_data,
        };

        let merged_file = dir.path().join("merged_specs.json");
        std::fs::File::create(&merged_file)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&merged_envelope)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        // Load the merged file back and verify provenance is flattened.
        let (_data, prov, cat) = load_generic_file(&merged_file).unwrap();
        assert_eq!(cat, SchemaCategory::Specs);
        assert_eq!(
            prov.len(),
            2,
            "merged specs should carry both original provenance entries"
        );

        let packages: Vec<&str> = prov.iter().map(|p| p.source.package.as_str()).collect();
        assert!(packages.contains(&"pkg-a"));
        assert!(packages.contains(&"pkg-b"));
    }

    #[test]
    fn test_category_detection() {
        use crate::types::detect_category;

        assert_eq!(
            detect_category("probe-verus/atoms"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe-lean/enriched-atoms"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe/merged-atoms"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe-verus/specs"),
            Some(SchemaCategory::Specs)
        );
        assert_eq!(
            detect_category("probe-lean/specs"),
            Some(SchemaCategory::Specs)
        );
        assert_eq!(
            detect_category("probe/merged-specs"),
            Some(SchemaCategory::Specs)
        );
        assert_eq!(
            detect_category("probe-verus/proofs"),
            Some(SchemaCategory::Proofs)
        );
        assert_eq!(
            detect_category("probe-lean/proofs"),
            Some(SchemaCategory::Proofs)
        );
        assert_eq!(
            detect_category("probe/merged-proofs"),
            Some(SchemaCategory::Proofs)
        );
        assert_eq!(detect_category("probe-verus/stubs"), None);
        assert_eq!(detect_category("something-else"), None);
    }

    #[test]
    fn test_category_mismatch_detected_by_loader() {
        use crate::types::load_generic_file;
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();

        let specs_file = dir.path().join("specs.json");
        let atoms_file = dir.path().join("atoms.json");

        let specs = serde_json::json!({
            "schema": "probe-verus/specs",
            "schema-version": "2.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "specify"},
            "source": {"repo": "r", "commit": "c", "language": "rust", "package": "p", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {"probe:a/1.0/f()": {"specified": true}}
        });
        let atoms = serde_json::json!({
            "schema": "probe-verus/atoms",
            "schema-version": "2.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "atomize"},
            "source": {"repo": "r", "commit": "c", "language": "rust", "package": "p", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {"probe:a/1.0/f()": {"display-name": "f", "dependencies": [], "code-module": "", "code-path": "a.rs", "code-text": {"lines-start": 1, "lines-end": 10}, "kind": "exec", "language": "rust"}}
        });

        std::fs::File::create(&specs_file)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&specs).unwrap().as_bytes())
            .unwrap();
        std::fs::File::create(&atoms_file)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&atoms).unwrap().as_bytes())
            .unwrap();

        let (_, _, cat_s) = load_generic_file(&specs_file).unwrap();
        let (_, _, cat_a) = load_generic_file(&atoms_file).unwrap();

        assert_eq!(cat_s, SchemaCategory::Specs);
        assert_eq!(cat_a, SchemaCategory::Atoms);
        assert_ne!(
            cat_s, cat_a,
            "different categories should be distinguishable"
        );
    }
}
