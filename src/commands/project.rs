// @kb: kb/tools/probe-project.md — graph projection from mapping seeds

use crate::authority::{load_validated_atom_file, AuthorityScope};
use crate::types::{
    endpoint_lookup_maps, load_mappings, Atom, InputProvenance, Tool, PROJECTED_ATOMS_SCHEMA,
};
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
            // P15: the categorized dependency subsets are trimmed with the
            // same filter as `dependencies`, so the decomposition equality
            // (`dependencies` = union of the subsets) survives projection.
            // Non-string entries pass through untouched.
            for field in crate::types::CATEGORIZED_DEPENDENCY_ARRAYS {
                if let Some(arr) = projected_atom
                    .extensions
                    .get_mut(field)
                    .and_then(|v| v.as_array_mut())
                {
                    arr.retain(|entry| match entry.as_str() {
                        Some(name) => included.contains(name),
                        None => true,
                    });
                }
            }
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

// @kb: kb/engineering/schema.md#projection-metadata
/// Envelope for projected output — the distinct `probe/projected-atoms`
/// schema (a projection is a view; merge/enrich reject it, ADR-006) with a
/// `projection` metadata block.
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
    // Load atoms. The input passes the version-gate component of the
    // authority validator (ADR-006 Decision 7): project re-stamps its output
    // at the current hub version, which would conceal a pre-contract origin.
    // Already-projected inputs stay readable (gate only, no projection
    // rejection).
    // @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
    eprintln!("  Loading {}...", input.display());
    let loaded = match load_validated_atom_file(&input, AuthorityScope::ReadOnly) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    let provenance = loaded.provenance;
    eprintln!(
        "    {} atoms, {} provenance entries",
        loaded.atoms.len(),
        provenance.len()
    );

    // Carrier preparation (ADR-006, P23): on an authoritative input, apply
    // `prepare = enrich ∘ normalize` — labels are recomputed on the *full*
    // input graph before any trimming, otherwise a stale producer label would
    // be frozen into a depth-limited view that the projection rejection rule
    // makes unrepairable. Seed matching then runs over normalized keys (P8),
    // the same rule `load_mappings` applies to endpoints. An already-projected
    // input skips preparation entirely: it is a view — its labels are
    // inherited, never recomputed over the trimmed graph.
    let atoms = if loaded.projected {
        eprintln!("  Input is a projection: labels inherited, no recomputation");
        loaded.atoms
    } else {
        let (atoms, stats) = match crate::commands::propagate::prepare_atoms(loaded.atoms) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}: {e}", input.display());
                std::process::exit(1);
            }
        };
        // P8: a normalization collision that would discard a distinct real
        // atom is producer error at every recomputation boundary — silently
        // selecting one atom's evidence could freeze laundered labels into
        // the view. Reject, exactly as `probe enrich` does.
        if !stats.dropped_atoms.is_empty() {
            for (discarded, kept) in &stats.dropped_atoms {
                eprintln!(
                    "Error: normalization collision: atom {discarded:?} would be discarded \
                     (a distinct atom already occupies {kept:?})"
                );
            }
            eprintln!(
                "Error: {}: refusing to project — normalization collided {} distinct \
                 atom(s); fix the producer aliases and regenerate (P8)",
                input.display(),
                stats.dropped_atoms.len()
            );
            std::process::exit(1);
        }
        if stats.keys_normalized > 0 {
            eprintln!("    Keys normalized: {}", stats.keys_normalized);
        }
        eprintln!(
            "    Enrichment recomputed on the full graph: {} transitively-verified, {} locally-scoped verified",
            stats.transitive, stats.local
        );
        atoms
    };

    // Load mappings
    eprintln!("  Loading mappings from {}...", mappings_path.display());
    let mappings = match load_mappings(&mappings_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    let (from_to, to_from) = endpoint_lookup_maps(&mappings);
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
        schema: PROJECTED_ATOMS_SCHEMA.to_string(),
        schema_version: "3.1".to_string(),
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
    use crate::types::load_atom_file;

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
            "schema-version": "3.0",
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
            "schema-version": "3.0",
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

    // P15: projection trims the categorized dependency subsets with the same
    // filter as `dependencies`, so the decomposition equality survives; a
    // non-string entry passes through untouched.
    #[test]
    fn test_categorized_arrays_trimmed_with_dependencies() {
        let mut atoms = BTreeMap::new();
        let mut f = make_atom("f", "rust", "exec", &["probe:g", "probe:outside"]);
        f.extensions.insert(
            "requires-dependencies".to_string(),
            serde_json::json!(["probe:outside"]),
        );
        f.extensions.insert(
            "body-dependencies".to_string(),
            serde_json::json!(["probe:g", "probe:outside", 42]),
        );
        atoms.insert("probe:f".to_string(), f);
        atoms.insert("probe:g".to_string(), make_atom("g", "rust", "exec", &[]));
        atoms.insert(
            "probe:outside".to_string(),
            make_atom("outside", "rust", "exec", &[]),
        );

        // Seeds f and g only; depth 0 excludes probe:outside.
        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let to_from: HashMap<String, Vec<String>> = HashMap::new();
        from_to
            .entry("probe:f".to_string())
            .or_default()
            .push("probe:g".to_string());
        from_to
            .entry("probe:g".to_string())
            .or_default()
            .push("probe:f".to_string());

        let (result, _) = project_atoms(&atoms, &from_to, &to_from, 0, 0);
        let f = &result["probe:f"];
        assert!(f.dependencies.contains("probe:g"));
        assert!(!f.dependencies.contains("probe:outside"));
        assert_eq!(
            f.extensions.get("requires-dependencies").unwrap(),
            &serde_json::json!([]),
            "excluded name trimmed from the categorized array (P15)"
        );
        assert_eq!(
            f.extensions.get("body-dependencies").unwrap(),
            &serde_json::json!(["probe:g", 42]),
            "included name kept, excluded trimmed, non-string untouched"
        );
    }

    // Plan §8: projection selection is dependency-only — correspondence
    // records are never traversed. With f → g where g is absent (dangling
    // dep), ga present, mapping g ↔ ga, reverse-depth 1: the old fabricated
    // f → ga edge used to select {f, ga}; without it the selection is {ga}.
    #[test]
    fn test_selection_is_dependency_only() {
        let mut atoms = BTreeMap::new();
        let mut f = make_atom("f", "rust", "exec", &["probe:g"]);
        f.extensions.insert(
            "maps-to".to_string(),
            serde_json::json!([{ "target": "probe:ga", "confidence": "exact" }]),
        );
        atoms.insert("probe:f".to_string(), f);
        let mut ga = make_atom("ga", "lean", "def", &[]);
        ga.extensions.insert(
            "mapped-from".to_string(),
            serde_json::json!([{ "target": "probe:g", "confidence": "exact" }]),
        );
        atoms.insert("probe:ga".to_string(), ga);

        let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
        let mut to_from: HashMap<String, Vec<String>> = HashMap::new();
        from_to
            .entry("probe:g".to_string())
            .or_default()
            .push("probe:ga".to_string());
        to_from
            .entry("probe:ga".to_string())
            .or_default()
            .push("probe:g".to_string());

        let (result, stats) = project_atoms(&atoms, &from_to, &to_from, 0, 1);

        assert_eq!(stats.seeds_found, 1, "only ga exists among the endpoints");
        assert!(result.contains_key("probe:ga"));
        assert!(
            !result.contains_key("probe:f"),
            "f is reachable only via records; records are never traversed"
        );
        assert_eq!(result.len(), 1);
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
