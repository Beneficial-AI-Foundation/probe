// @kb: kb/engineering/properties.md#p14-deterministic-output
// @kb: kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination

use crate::types::Atom;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

fn get_verification_status(atom: &Atom) -> Option<&str> {
    atom.extensions
        .get("verification-status")
        .and_then(|v| v.as_str())
}

fn is_verified(atom: &Atom) -> bool {
    get_verification_status(atom).is_some_and(|s| s == "verified" || s == "transitively-verified")
}

/// Only atoms with explicit "unverified" or "failed" status are contamination
/// sources. Atoms with missing status (untracked/Grey) and "trusted" atoms
/// do not contaminate — they are outside the verification scope.
fn is_contamination_source(atom: &Atom) -> bool {
    matches!(get_verification_status(atom), Some("unverified" | "failed"))
}

/// Enrich verification status through the dependency graph using
/// reverse-BFS contamination.
///
/// For each verified atom, determines whether it is **transitively verified**
/// (all transitive dependencies are verified or trusted) or only
/// **locally verified** (the atom itself is verified but at least one
/// transitive dependency is not).
///
/// Upgrades `verification-status` from `"verified"` to `"transitively-verified"`
/// on atoms whose entire transitive dependency closure is verified or trusted.
/// Atoms that remain `"verified"` are only locally verified. Non-verified atoms
/// are untouched.
///
/// Returns `(transitive_count, local_count, missing_deps)` for reporting.
// @kb: kb/engineering/schema.md#verification-status-values
pub fn enrich_verification_status(
    atoms: &mut BTreeMap<String, Atom>,
) -> (usize, usize, Vec<String>) {
    // Use owned Strings throughout to avoid borrow-checker conflicts
    // between reading atoms (for the graph) and writing back results.

    // 1. Build reverse dependency index: for each dep, who depends on it?
    //    Uses BTreeMap/BTreeSet for deterministic iteration (P14).
    let mut reverse_deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut verified_set: BTreeSet<String> = BTreeSet::new();
    let mut missing_deps: Vec<String> = Vec::new();

    for (code_name, atom) in atoms.iter() {
        if is_verified(atom) {
            verified_set.insert(code_name.clone());
        }
        for dep in &atom.dependencies {
            if !atoms.contains_key(dep.as_str()) {
                missing_deps.push(dep.clone());
            }
            reverse_deps
                .entry(dep.clone())
                .or_default()
                .insert(code_name.clone());
        }
    }

    missing_deps.sort();
    missing_deps.dedup();
    for missing in &missing_deps {
        eprintln!("Warning: dependency {missing:?} not found in atom map (treated as trusted)");
    }

    // 2. Seed contamination: only atoms with explicit "unverified" or "failed" status.
    //    Atoms with missing verification-status (untracked/Grey) and "trusted" atoms
    //    do not contaminate — they are outside the verification scope.
    let mut contaminated: BTreeSet<String> = BTreeSet::new();
    let mut worklist: VecDeque<String> = VecDeque::new();

    for (code_name, atom) in atoms.iter() {
        if is_contamination_source(atom) {
            contaminated.insert(code_name.clone());
        }
    }

    // 3. Find direct contacts: verified atoms that depend on a contaminated atom.
    let initial_sources: Vec<String> = contaminated.iter().cloned().collect();
    for source in &initial_sources {
        if let Some(callers) = reverse_deps.get(source) {
            for caller in callers {
                if verified_set.contains(caller) && !contaminated.contains(caller) {
                    contaminated.insert(caller.clone());
                    worklist.push_back(caller.clone());
                }
            }
        }
    }

    // 4. Propagate via reverse edges.
    while let Some(atom_name) = worklist.pop_front() {
        if let Some(callers) = reverse_deps.get(&atom_name) {
            for caller in callers {
                if verified_set.contains(caller) && !contaminated.contains(caller) {
                    contaminated.insert(caller.clone());
                    worklist.push_back(caller.clone());
                }
            }
        }
    }

    // 5. Upgrade non-contaminated verified atoms to "transitively-verified".
    let mut transitive_count = 0;
    let mut local_count = 0;

    for code_name in &verified_set {
        if contaminated.contains(code_name) {
            local_count += 1;
        } else {
            transitive_count += 1;
            atoms.get_mut(code_name).unwrap().extensions.insert(
                "verification-status".to_string(),
                serde_json::Value::String("transitively-verified".to_string()),
            );
        }
    }

    (transitive_count, local_count, missing_deps)
}

/// CLI entry point: load atom file, enrich verification status, write JSON.
pub fn cmd_enrich(input: &Path, output: Option<&Path>) {
    let content = std::fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", input.display());
        std::process::exit(1);
    });

    let mut raw: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing JSON in {}: {e}", input.display());
        std::process::exit(1);
    });

    let schema_version = raw
        .get("schema-version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !schema_version.starts_with("2.") {
        eprintln!(
            "Error: {}: incompatible schema-version \"{schema_version}\" (expected 2.x)",
            input.display()
        );
        std::process::exit(1);
    }

    let data_value = raw.get("data").cloned().unwrap_or_else(|| {
        eprintln!("Error: {}: missing \"data\" field", input.display());
        std::process::exit(1);
    });

    let mut atoms: BTreeMap<String, Atom> =
        serde_json::from_value(data_value).unwrap_or_else(|e| {
            eprintln!("Error deserializing atoms from {}: {e}", input.display());
            std::process::exit(1);
        });

    let (transitive, local, _missing) = enrich_verification_status(&mut atoms);
    let not_verified = atoms.len() - transitive - local;

    eprintln!(
        "Transitively verified: {transitive}  |  Locally-scoped verified: {local}  |  Not verified: {not_verified}"
    );

    let enriched_data = serde_json::to_value(&atoms).expect("failed to serialize atoms");
    raw.as_object_mut()
        .expect("envelope is not a JSON object")
        .insert("data".to_string(), enriched_data);

    let json = serde_json::to_string_pretty(&raw).expect("failed to serialize output");

    let default_name;
    let out_path = match output {
        Some(p) => p,
        None => {
            default_name = default_output_name(input);
            Path::new(&default_name)
        }
    };

    std::fs::write(out_path, &json).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {e}", out_path.display());
        std::process::exit(1);
    });
    eprintln!("Wrote {}", out_path.display());
}

fn default_output_name(input: &Path) -> String {
    let stem = input
        .file_stem()
        .map_or("atoms", |s| s.to_str().unwrap_or("atoms"));
    format!("enriched_{stem}.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CodeText;

    fn make_atom(display_name: &str) -> Atom {
        Atom {
            display_name: display_name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: "src/lib.rs".to_string(),
            code_text: CodeText {
                lines_start: 1,
                lines_end: 10,
            },
            kind: "exec".to_string(),
            language: "rust".to_string(),
            extensions: BTreeMap::new(),
        }
    }

    fn set_status(atom: &mut Atom, status: &str) {
        atom.extensions.insert(
            "verification-status".to_string(),
            serde_json::Value::String(status.to_string()),
        );
    }

    fn set_verified(atom: &mut Atom) {
        set_status(atom, "verified");
    }

    fn set_trusted(atom: &mut Atom) {
        set_status(atom, "trusted");
    }

    fn add_dep(atom: &mut Atom, dep: &str) {
        atom.dependencies.insert(dep.to_string());
    }

    fn get_vs(atom: &Atom) -> Option<&str> {
        atom.extensions
            .get("verification-status")
            .and_then(|v| v.as_str())
    }

    #[test]
    fn test_leaf_no_deps() {
        let mut atoms = BTreeMap::new();
        let mut a = make_atom("a");
        set_verified(&mut a);
        atoms.insert("a".to_string(), a);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_all_deps_verified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_verified(&mut c);
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(
            get_vs(atoms.get("b").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(
            get_vs(atoms.get("c").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_one_dep_failed_contaminates() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "failed");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("failed"));
    }

    #[test]
    fn test_one_dep_unverified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_dep_trusted_does_not_block() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_trusted(&mut b);
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_dep_missing_from_map() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "nonexistent");
        atoms.insert("a".to_string(), a);

        let (_t, _l, missing) = enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(missing, vec!["nonexistent"]);
    }

    #[test]
    fn test_transitive_chain() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_status(&mut c, "unverified");
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_diamond_dependency() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        add_dep(&mut a, "c");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "d");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_verified(&mut c);
        add_dep(&mut c, "d");
        atoms.insert("c".to_string(), c);

        let mut d = make_atom("d");
        set_status(&mut d, "unverified");
        atoms.insert("d".to_string(), d);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("d").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_cycle_all_verified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "a");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(
            get_vs(atoms.get("b").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_cycle_with_unverified_dep() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        add_dep(&mut b, "d");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_verified(&mut c);
        add_dep(&mut c, "a");
        atoms.insert("c".to_string(), c);

        let mut d = make_atom("d");
        set_status(&mut d, "unverified");
        atoms.insert("d".to_string(), d);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("d").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_missing_status_does_not_contaminate() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        // b has no verification-status at all (untracked/Grey)
        let b = make_atom("b");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_explicit_unverified_contaminates_but_missing_does_not() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        add_dep(&mut a, "c");
        atoms.insert("a".to_string(), a);

        // b is explicitly unverified — contaminates
        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        // c has no status — does NOT contaminate
        let c = make_atom("c");
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
    }

    #[test]
    fn test_non_verified_atoms_untouched() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_status(&mut a, "unverified");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "failed");
        atoms.insert("b".to_string(), b);

        let c = make_atom("c");
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("unverified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("failed"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), None);
    }

    #[test]
    fn test_idempotency() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        let first = get_vs(atoms.get("a").unwrap()).unwrap().to_string();

        enrich_verification_status(&mut atoms);
        let second = get_vs(atoms.get("a").unwrap()).unwrap().to_string();

        assert_eq!(first, second);
        assert_eq!(first, "verified");
    }

    #[test]
    fn test_idempotency_transitively_verified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        atoms.insert("a".to_string(), a);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );

        // Running again should not change the result (already upgraded)
        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_deterministic_output() {
        let build = || {
            let mut atoms = BTreeMap::new();
            let mut a = make_atom("a");
            set_verified(&mut a);
            add_dep(&mut a, "b");
            add_dep(&mut a, "c");
            atoms.insert("a".to_string(), a);

            let mut b = make_atom("b");
            set_verified(&mut b);
            atoms.insert("b".to_string(), b);

            let mut c = make_atom("c");
            set_status(&mut c, "unverified");
            atoms.insert("c".to_string(), c);
            atoms
        };

        let mut atoms1 = build();
        let mut atoms2 = build();

        enrich_verification_status(&mut atoms1);
        enrich_verification_status(&mut atoms2);

        let json1 = serde_json::to_string(&atoms1).unwrap();
        let json2 = serde_json::to_string(&atoms2).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_counts_are_correct() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_status(&mut c, "unverified");
        atoms.insert("c".to_string(), c);

        let (transitive, local, missing) = enrich_verification_status(&mut atoms);
        assert_eq!(transitive, 1);
        assert_eq!(local, 1);
        assert!(missing.is_empty());
    }
}
