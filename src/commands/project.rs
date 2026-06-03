// @kb: kb/tools/probe-project.md — graph projection from mapping seeds

use crate::types::{load_atom_file, load_mappings, Atom, InputProvenance, Tool};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};

/// Statistics reported after projection.
pub struct ProjectStats {
    pub atoms_in: usize,
    pub atoms_out: usize,
    pub seeds_requested: usize,
    pub seeds_found: usize,
    pub deps_trimmed: usize,
}

/// Result of projecting: the filtered atom map and stats.
pub type ProjectResult = (BTreeMap<String, Atom>, ProjectStats);

/// Pure projection function: extract a subgraph seeded by mapping endpoints.
///
/// Given an atom map and bidirectional mapping lookups, builds the seed set
/// (all `from` + `to` keys that exist in `atoms`), then expands via BFS:
/// - Forward (callee direction) up to `forward_depth`
/// - Backward (caller direction) up to `reverse_depth`
///
/// Returns the filtered atoms with dependencies trimmed to the included set.
// @kb: kb/engineering/properties.md#p14-deterministic-output
// @kb: kb/engineering/properties.md#p9-provenance-is-preserved
// @kb: kb/engineering/properties.md#p1-envelope-completeness
pub fn project_atoms(
    atoms: &BTreeMap<String, Atom>,
    from_to: &HashMap<String, Vec<String>>,
    to_from: &HashMap<String, Vec<String>>,
    forward_depth: usize,
    reverse_depth: usize,
) -> ProjectResult {
    let atoms_in = atoms.len();

    // Step 1: Build seed set from mapping endpoints present in atom data
    let mut seeds = BTreeSet::new();
    for key in from_to.keys() {
        if atoms.contains_key(key) {
            seeds.insert(key.clone());
        }
    }
    for key in to_from.keys() {
        if atoms.contains_key(key) {
            seeds.insert(key.clone());
        }
    }
    let seeds_requested = {
        let mut all_mapping_keys = BTreeSet::new();
        for key in from_to.keys() {
            all_mapping_keys.insert(key.clone());
        }
        for key in to_from.keys() {
            all_mapping_keys.insert(key.clone());
        }
        all_mapping_keys.len()
    };
    let seeds_found = seeds.len();

    // Step 2: Build reverse adjacency index ("who depends on me?")
    let mut reverse_adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if reverse_depth > 0 {
        for (key, atom) in atoms {
            for dep in &atom.dependencies {
                reverse_adj
                    .entry(dep.clone())
                    .or_default()
                    .insert(key.clone());
            }
        }
    }

    // Step 3: BFS forward (callees)
    let mut included = seeds.clone();
    if forward_depth > 0 {
        let mut queue: VecDeque<(String, usize)> = seeds.iter().map(|s| (s.clone(), 0)).collect();
        let mut visited = seeds.clone();

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= forward_depth {
                continue;
            }
            if let Some(atom) = atoms.get(&node) {
                for dep in &atom.dependencies {
                    if atoms.contains_key(dep) && visited.insert(dep.clone()) {
                        included.insert(dep.clone());
                        queue.push_back((dep.clone(), depth + 1));
                    }
                }
            }
        }
    }

    // Step 4: BFS backward (callers)
    if reverse_depth > 0 {
        let mut queue: VecDeque<(String, usize)> = seeds.iter().map(|s| (s.clone(), 0)).collect();
        let mut visited: BTreeSet<String> = seeds.clone();

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= reverse_depth {
                continue;
            }
            if let Some(callers) = reverse_adj.get(&node) {
                for caller in callers {
                    if visited.insert(caller.clone()) {
                        included.insert(caller.clone());
                        queue.push_back((caller.clone(), depth + 1));
                    }
                }
            }
        }
    }

    // Step 5: Filter atoms and trim dependencies
    let mut deps_trimmed = 0;
    let mut result: BTreeMap<String, Atom> = BTreeMap::new();

    for key in &included {
        if let Some(atom) = atoms.get(key) {
            let mut projected_atom = atom.clone();
            let original_dep_count = projected_atom.dependencies.len();
            projected_atom.dependencies.retain(|d| included.contains(d));
            deps_trimmed += original_dep_count - projected_atom.dependencies.len();
            result.insert(key.clone(), projected_atom);
        }
    }

    let stats = ProjectStats {
        atoms_in,
        atoms_out: result.len(),
        seeds_requested,
        seeds_found,
        deps_trimmed,
    };

    (result, stats)
}

/// Envelope for projected output — uses `probe/merged-atoms` schema with
/// an extra `projection` metadata block.
#[derive(serde::Serialize)]
struct ProjectedEnvelope {
    schema: String,
    #[serde(rename = "schema-version")]
    schema_version: String,
    tool: Tool,
    inputs: Vec<InputProvenance>,
    timestamp: String,
    projection: ProjectionMeta,
    data: BTreeMap<String, Atom>,
}

#[derive(serde::Serialize)]
struct ProjectionMeta {
    #[serde(rename = "mappings-file")]
    mappings_file: String,
    seeds: usize,
    #[serde(rename = "forward-depth")]
    forward_depth: usize,
    #[serde(rename = "reverse-depth")]
    reverse_depth: usize,
    #[serde(rename = "atoms-in")]
    atoms_in: usize,
    #[serde(rename = "atoms-out")]
    atoms_out: usize,
    #[serde(rename = "deps-trimmed")]
    deps_trimmed: usize,
}

#[derive(serde::Serialize)]
struct FocusSet {
    focus_nodes: Vec<String>,
    metadata: FocusMetadata,
}

#[derive(serde::Serialize)]
struct FocusMetadata {
    description: String,
}

/// CLI entry point for `probe project`.
pub fn cmd_project(
    input: PathBuf,
    mappings_path: PathBuf,
    forward_depth: usize,
    reverse_depth: usize,
    output: PathBuf,
    emit_focus: bool,
) {
    // Load atoms
    eprintln!("  Loading {}...", input.display());
    let (atoms, provenance) = match load_atom_file(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    eprintln!(
        "    {} atoms, {} provenance entries",
        atoms.len(),
        provenance.len()
    );

    // Load mappings
    eprintln!("  Loading mappings from {}...", mappings_path.display());
    let (from_to, to_from) = match load_mappings(&mappings_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    eprintln!(
        "    {} from→to entries, {} to→from entries",
        from_to.len(),
        to_from.len()
    );

    // Project
    eprintln!();
    eprintln!(
        "Projecting (forward-depth={}, reverse-depth={})...",
        forward_depth, reverse_depth
    );
    let (projected, stats) =
        project_atoms(&atoms, &from_to, &to_from, forward_depth, reverse_depth);

    if stats.seeds_found < stats.seeds_requested {
        eprintln!(
            "  Warning: {}/{} mapping keys found in atom data",
            stats.seeds_found, stats.seeds_requested
        );
    }

    // Build envelope
    let tool = Tool {
        name: "probe".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: "project".to_string(),
    };
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let envelope = ProjectedEnvelope {
        schema: "probe/merged-atoms".to_string(),
        schema_version: "2.0".to_string(),
        tool,
        inputs: provenance,
        timestamp,
        projection: ProjectionMeta {
            mappings_file: mappings_path.file_name().map_or_else(
                || "unknown".to_string(),
                |f| f.to_string_lossy().to_string(),
            ),
            seeds: stats.seeds_found,
            forward_depth,
            reverse_depth,
            atoms_in: stats.atoms_in,
            atoms_out: stats.atoms_out,
            deps_trimmed: stats.deps_trimmed,
        },
        data: projected.clone(),
    };

    let json = match serde_json::to_string_pretty(&envelope) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error: failed to serialize projected atoms: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = std::fs::write(&output, &json) {
        eprintln!("Error: failed to write {}: {e}", output.display());
        std::process::exit(1);
    }

    // Focus-set emission
    if emit_focus {
        let focus_path = focus_path_from(&output);
        let focus_nodes: Vec<String> = projected.keys().cloned().collect();
        let focus_set = FocusSet {
            focus_nodes,
            metadata: FocusMetadata {
                description: format!(
                    "Projection: {} seeds, forward-depth {}, reverse-depth {}, {} atoms",
                    stats.seeds_found, forward_depth, reverse_depth, stats.atoms_out
                ),
            },
        };
        let focus_json = match serde_json::to_string_pretty(&focus_set) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Error: failed to serialize focus set: {e}");
                std::process::exit(1);
            }
        };
        if let Err(e) = std::fs::write(&focus_path, &focus_json) {
            eprintln!("Error: failed to write {}: {e}", focus_path.display());
            std::process::exit(1);
        }
        eprintln!("  Focus set: {}", focus_path.display());
    }

    // Print stats
    eprintln!();
    eprintln!("Output: {}", output.display());
    eprintln!("  Seeds:          {}", stats.seeds_found);
    eprintln!("  Atoms in:       {}", stats.atoms_in);
    eprintln!("  Atoms out:      {}", stats.atoms_out);
    eprintln!("  Deps trimmed:   {}", stats.deps_trimmed);
}

/// Derive the focus-set file path from the main output path.
fn focus_path_from(output: &Path) -> PathBuf {
    let stem = output
        .file_stem()
        .map_or("projected", |s| s.to_str().unwrap_or("projected"));
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}_focus.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_atom(name: &str, language: &str, kind: &str, deps: &[&str]) -> Atom {
        Atom {
            display_name: name.to_string(),
            dependencies: deps.iter().map(|d| d.to_string()).collect(),
            code_module: String::new(),
            code_path: format!("src/{name}.rs"),
            code_text: crate::types::CodeText {
                lines_start: 1,
                lines_end: 10,
            },
            kind: kind.to_string(),
            language: language.to_string(),
            extensions: BTreeMap::new(),
        }
    }

    fn make_atoms() -> BTreeMap<String, Atom> {
        // Graph topology:
        //   rust_main -> rust_encrypt -> rust_aes
        //   lean_aead_encrypt -> lean_detse_encrypt -> lean_game0
        //   lean_game0 -> lean_theorem
        //
        // Mappings: rust_encrypt <-> lean_aead_encrypt, rust_aes <-> lean_detse_encrypt
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:crate/1.0/main()".to_string(),
            make_atom("main", "rust", "exec", &["probe:crate/1.0/encrypt()"]),
        );
        atoms.insert(
            "probe:crate/1.0/encrypt()".to_string(),
            make_atom("encrypt", "rust", "exec", &["probe:crate/1.0/aes()"]),
        );
        atoms.insert(
            "probe:crate/1.0/aes()".to_string(),
            make_atom("aes", "rust", "exec", &[]),
        );
        atoms.insert(
            "probe:crate/1.0/unrelated()".to_string(),
            make_atom("unrelated", "rust", "exec", &[]),
        );
        atoms.insert(
            "probe:AEADScheme.encrypt".to_string(),
            make_atom("encrypt", "lean", "def", &["probe:DetSEAlg.encrypt"]),
        );
        atoms.insert(
            "probe:DetSEAlg.encrypt".to_string(),
            make_atom("encrypt", "lean", "def", &["probe:game0"]),
        );
        atoms.insert(
            "probe:game0".to_string(),
            make_atom("game0", "lean", "def", &["probe:theorem"]),
        );
        atoms.insert(
            "probe:theorem".to_string(),
            make_atom("theorem", "lean", "theorem", &[]),
        );
        atoms
    }

    fn make_mappings() -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let mut to_from: HashMap<String, Vec<String>> = HashMap::new();
        from_to
            .entry("probe:crate/1.0/encrypt()".to_string())
            .or_default()
            .push("probe:AEADScheme.encrypt".to_string());
        from_to
            .entry("probe:crate/1.0/aes()".to_string())
            .or_default()
            .push("probe:DetSEAlg.encrypt".to_string());
        to_from
            .entry("probe:AEADScheme.encrypt".to_string())
            .or_default()
            .push("probe:crate/1.0/encrypt()".to_string());
        to_from
            .entry("probe:DetSEAlg.encrypt".to_string())
            .or_default()
            .push("probe:crate/1.0/aes()".to_string());
        (from_to, to_from)
    }

    #[test]
    fn test_seeds_only_depth_zero() {
        let atoms = make_atoms();
        let (from_to, to_from) = make_mappings();

        let (result, stats) = project_atoms(&atoms, &from_to, &to_from, 0, 0);

        assert_eq!(stats.seeds_found, 4, "4 mapping endpoints in atom data");
        assert_eq!(result.len(), 4, "depth 0 = seeds only");
        assert!(result.contains_key("probe:crate/1.0/encrypt()"));
        assert!(result.contains_key("probe:crate/1.0/aes()"));
        assert!(result.contains_key("probe:AEADScheme.encrypt"));
        assert!(result.contains_key("probe:DetSEAlg.encrypt"));
        assert!(!result.contains_key("probe:crate/1.0/main()"));
        assert!(!result.contains_key("probe:game0"));
    }

    #[test]
    fn test_forward_only() {
        let atoms = make_atoms();
        let (from_to, to_from) = make_mappings();

        let (result, _stats) = project_atoms(&atoms, &from_to, &to_from, 2, 0);

        // Seeds + forward deps (callees):
        // AEADScheme.encrypt -> DetSEAlg.encrypt (seed, depth 0->already seed)
        // DetSEAlg.encrypt -> game0 (depth 1) -> theorem (depth 2)
        // encrypt() -> aes() (seed, already in)
        // aes() has no deps
        assert!(result.contains_key("probe:game0"), "forward from Lean seed");
        assert!(
            result.contains_key("probe:theorem"),
            "forward depth 2 from Lean seed"
        );
        assert!(
            !result.contains_key("probe:crate/1.0/main()"),
            "callers not included in forward-only"
        );
        assert!(
            !result.contains_key("probe:crate/1.0/unrelated()"),
            "unrelated atom excluded"
        );
    }

    #[test]
    fn test_reverse_only() {
        let atoms = make_atoms();
        let (from_to, to_from) = make_mappings();

        let (result, _stats) = project_atoms(&atoms, &from_to, &to_from, 0, 1);

        // Seeds + reverse deps (callers, depth 1):
        // encrypt() is depended on by main() -> included
        // AEADScheme.encrypt has no callers in the graph
        assert!(
            result.contains_key("probe:crate/1.0/main()"),
            "caller of seed included via reverse"
        );
        assert!(
            !result.contains_key("probe:game0"),
            "callees not included in reverse-only"
        );
    }

    #[test]
    fn test_bidirectional() {
        let atoms = make_atoms();
        let (from_to, to_from) = make_mappings();

        let (result, stats) = project_atoms(&atoms, &from_to, &to_from, 2, 1);

        // Forward: seeds + game0 + theorem
        // Reverse: main()
        // Should include everything except unrelated
        assert_eq!(result.len(), 7, "all atoms except unrelated");
        assert!(!result.contains_key("probe:crate/1.0/unrelated()"));
        assert_eq!(stats.atoms_in, 8);
        assert_eq!(stats.atoms_out, 7);
    }

    #[test]
    fn test_dep_trimming() {
        let atoms = make_atoms();
        let (from_to, to_from) = make_mappings();

        // With depth 0, only seeds are included. encrypt() depends on aes()
        // (both seeds) but also nothing outside. main() is excluded, so
        // no atom should reference it.
        let (result, stats) = project_atoms(&atoms, &from_to, &to_from, 0, 0);

        // AEADScheme.encrypt depends on DetSEAlg.encrypt (both seeds) -> kept
        // DetSEAlg.encrypt depends on game0 (not a seed) -> trimmed
        assert!(
            !result["probe:DetSEAlg.encrypt"]
                .dependencies
                .contains("probe:game0"),
            "dep to non-included atom should be trimmed"
        );
        assert!(
            result["probe:AEADScheme.encrypt"]
                .dependencies
                .contains("probe:DetSEAlg.encrypt"),
            "dep to included atom should be kept"
        );
        assert!(stats.deps_trimmed > 0);
    }

    #[test]
    fn test_determinism() {
        let atoms = make_atoms();
        let (from_to, to_from) = make_mappings();

        let (result1, _) = project_atoms(&atoms, &from_to, &to_from, 2, 1);
        let (result2, _) = project_atoms(&atoms, &from_to, &to_from, 2, 1);

        let keys1: Vec<&String> = result1.keys().collect();
        let keys2: Vec<&String> = result2.keys().collect();
        assert_eq!(keys1, keys2, "output must be deterministic (P14)");
    }

    #[test]
    fn test_missing_seeds_skipped() {
        let atoms = make_atoms();
        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let mut to_from: HashMap<String, Vec<String>> = HashMap::new();

        // Ghost mapping: keys don't exist in atoms
        from_to
            .entry("probe:ghost/1.0/phantom()".to_string())
            .or_default()
            .push("probe:GhostLean.phantom".to_string());
        to_from
            .entry("probe:GhostLean.phantom".to_string())
            .or_default()
            .push("probe:ghost/1.0/phantom()".to_string());

        let (_result, stats) = project_atoms(&atoms, &from_to, &to_from, 2, 0);

        assert_eq!(stats.seeds_requested, 2);
        assert_eq!(stats.seeds_found, 0, "ghost seeds should be skipped");
        assert_eq!(
            stats.atoms_out, 0,
            "no atoms included when all seeds missing"
        );
    }

    #[test]
    fn test_provenance_passthrough_merged_input() {
        // This tests the load path through cmd_project indirectly.
        // Here we test load_atom_file with a merged envelope.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("merged.json");
        let content = serde_json::json!({
            "schema": "probe/merged-atoms",
            "schema-version": "2.0",
            "tool": {"name": "probe", "version": "0.2.0", "command": "merge"},
            "inputs": [
                {"schema": "probe-rust/extract", "source": {"repo": "r", "commit": "c", "language": "rust", "package": "pkg-a", "package-version": "1.0"}},
                {"schema": "probe-lean/extract", "source": {"repo": "r", "commit": "c", "language": "lean", "package": "pkg-b", "package-version": "1.0"}}
            ],
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {
                "probe:a/1.0/f()": {
                    "display-name": "f", "dependencies": [], "code-module": "", "code-path": "a.rs",
                    "code-text": {"lines-start": 1, "lines-end": 10}, "kind": "exec", "language": "rust"
                }
            }
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let (atoms, provenance) = load_atom_file(&path).unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(
            provenance.len(),
            2,
            "merged envelope carries both provenance entries"
        );
        assert_eq!(provenance[0].source.package, "pkg-a");
        assert_eq!(provenance[1].source.package, "pkg-b");
    }

    #[test]
    fn test_provenance_passthrough_single_tool_input() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("single.json");
        let content = serde_json::json!({
            "schema": "probe-rust/extract",
            "schema-version": "2.0",
            "tool": {"name": "probe-rust", "version": "1.0.0", "command": "extract"},
            "source": {"repo": "r", "commit": "c", "language": "rust", "package": "mypkg", "package-version": "1.0"},
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {
                "probe:mypkg/1.0/f()": {
                    "display-name": "f", "dependencies": [], "code-module": "", "code-path": "src/lib.rs",
                    "code-text": {"lines-start": 1, "lines-end": 10}, "kind": "exec", "language": "rust"
                }
            }
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let (atoms, provenance) = load_atom_file(&path).unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(
            provenance.len(),
            1,
            "single-tool wraps source into one provenance entry"
        );
        assert_eq!(provenance[0].source.package, "mypkg");
    }

    #[test]
    fn test_focus_path_derivation() {
        assert_eq!(
            focus_path_from(Path::new("out/focused.json")),
            PathBuf::from("out/focused_focus.json")
        );
        let result = focus_path_from(Path::new("projected.json"));
        assert!(
            result.file_name().unwrap() == "projected_focus.json",
            "filename should be projected_focus.json, got {:?}",
            result
        );
    }

    #[test]
    fn test_circular_dependencies_terminate() {
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a".to_string(),
            make_atom("a", "rust", "exec", &["probe:b"]),
        );
        atoms.insert(
            "probe:b".to_string(),
            make_atom("b", "rust", "exec", &["probe:c"]),
        );
        atoms.insert(
            "probe:c".to_string(),
            make_atom("c", "rust", "exec", &["probe:a"]),
        );
        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let mut to_from: HashMap<String, Vec<String>> = HashMap::new();
        from_to
            .entry("probe:a".to_string())
            .or_default()
            .push("probe:b".to_string());
        to_from
            .entry("probe:b".to_string())
            .or_default()
            .push("probe:a".to_string());

        let (result, stats) = project_atoms(&atoms, &from_to, &to_from, 10, 10);

        assert_eq!(
            result.len(),
            3,
            "cycle should terminate and include all reachable atoms"
        );
        assert_eq!(stats.deps_trimmed, 0, "all deps are within the cycle");
    }

    #[test]
    fn test_extensions_preserved_through_projection() {
        let mut atoms = BTreeMap::new();
        let mut atom = make_atom("f", "rust", "exec", &[]);
        atom.extensions.insert(
            "verification-status".to_string(),
            serde_json::json!("verified"),
        );
        atom.extensions
            .insert("rust-source".to_string(), serde_json::json!("fn f() {}"));
        atoms.insert("probe:f".to_string(), atom);

        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let to_from: HashMap<String, Vec<String>> = HashMap::new();
        from_to
            .entry("probe:f".to_string())
            .or_default()
            .push("probe:ghost".to_string());

        let (result, _) = project_atoms(&atoms, &from_to, &to_from, 0, 0);

        let projected = &result["probe:f"];
        assert_eq!(
            projected.extensions.get("verification-status"),
            Some(&serde_json::json!("verified")),
            "P10: extensions must survive projection"
        );
        assert_eq!(
            projected.extensions.get("rust-source"),
            Some(&serde_json::json!("fn f() {}")),
        );
    }

    #[test]
    fn test_stub_seeds_included() {
        let mut atoms = BTreeMap::new();
        let stub = Atom {
            display_name: "external".to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: String::new(),
            code_text: crate::types::CodeText {
                lines_start: 0,
                lines_end: 0,
            },
            kind: "exec".to_string(),
            language: "rust".to_string(),
            extensions: BTreeMap::new(),
        };
        assert!(stub.is_stub());
        atoms.insert("probe:stub".to_string(), stub);

        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let to_from: HashMap<String, Vec<String>> = HashMap::new();
        from_to
            .entry("probe:stub".to_string())
            .or_default()
            .push("probe:ghost".to_string());

        let (result, _) = project_atoms(&atoms, &from_to, &to_from, 0, 0);
        assert!(
            result.contains_key("probe:stub"),
            "stubs in seed set must be included"
        );
    }

    #[test]
    fn test_one_to_many_mapping_seeds() {
        let mut atoms = make_atoms();
        atoms.insert(
            "probe:extra_lean".to_string(),
            make_atom("extra_lean", "lean", "def", &[]),
        );

        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let mut to_from: HashMap<String, Vec<String>> = HashMap::new();
        // 1-to-many: one Rust function maps to two Lean atoms
        from_to
            .entry("probe:crate/1.0/encrypt()".to_string())
            .or_default()
            .push("probe:AEADScheme.encrypt".to_string());
        from_to
            .entry("probe:crate/1.0/encrypt()".to_string())
            .or_default()
            .push("probe:extra_lean".to_string());
        to_from
            .entry("probe:AEADScheme.encrypt".to_string())
            .or_default()
            .push("probe:crate/1.0/encrypt()".to_string());
        to_from
            .entry("probe:extra_lean".to_string())
            .or_default()
            .push("probe:crate/1.0/encrypt()".to_string());

        let (result, stats) = project_atoms(&atoms, &from_to, &to_from, 0, 0);

        assert!(
            result.contains_key("probe:extra_lean"),
            "1-to-many target must be a seed"
        );
        assert!(result.contains_key("probe:AEADScheme.encrypt"));
        assert!(result.contains_key("probe:crate/1.0/encrypt()"));
        assert_eq!(stats.seeds_found, 3);
    }
}
