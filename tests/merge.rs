//! Integration tests for the `probe merge` command.
//!
//! Tests all three schema categories (atoms, specs, proofs) using fixture files
//! in tests/fixtures/merge_test/.

use probe::types::{Atom, MergedAtomEnvelope, MergedGenericEnvelope};
use std::collections::BTreeMap;
use std::process::Command;
use tempfile::TempDir;

const FIXTURES: &str = "tests/fixtures/merge_test";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn run_probe_merge(file_a: &str, file_b: &str, output_path: &std::path::Path) {
    let binary = env!("CARGO_BIN_EXE_probe");
    let status = Command::new(binary)
        .args([
            "merge",
            &format!("{FIXTURES}/{file_a}"),
            &format!("{FIXTURES}/{file_b}"),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run probe");
    assert!(
        status.success(),
        "merge command failed for {file_a} + {file_b}"
    );
}

fn load_merged_atom_envelope(path: &str) -> MergedAtomEnvelope {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"))
}

fn load_merged_generic_envelope(path: &str) -> MergedGenericEnvelope {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"))
}

fn load_input_atoms(path: &str) -> BTreeMap<String, Atom> {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    let raw: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));
    let data = raw.get("data").expect("missing data field");
    serde_json::from_value(data.clone()).expect("failed to deserialize atoms")
}

// ===========================================================================
// Atoms tests
// ===========================================================================

/// Merged atom file has the expected entries after stub replacement.
///
/// - main() and process() from crate-a are kept (real atoms in base)
/// - compute() stub in atoms_a is replaced by real atom from atoms_b
/// - validate() stub in atoms_a is replaced by real atom from atoms_b
/// - internal() from atoms_b is added (new atom not in base)
#[test]
fn test_atoms_merge_fixtures_match_expected() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_probe_merge("atoms_a.json", "atoms_b.json", &output_path);

    let envelope = load_merged_atom_envelope(output_path.to_str().unwrap());
    let merged = &envelope.data;

    assert_eq!(envelope.schema, "probe/merged-atoms");
    assert_eq!(envelope.schema_version, "2.0");
    assert_eq!(envelope.tool.name, "probe");
    assert_eq!(envelope.tool.command, "merge");
    assert_eq!(envelope.inputs.len(), 2);

    assert_eq!(merged.len(), 5, "Should have 5 atoms after merge");

    let main_fn = &merged["probe:crate-a/1.0/lib/main()"];
    assert_eq!(main_fn.display_name, "main");
    assert_eq!(main_fn.code_path, "crate-a/src/lib.rs");
    assert_eq!(main_fn.code_text.lines_start, 1);
    assert_eq!(main_fn.code_text.lines_end, 10);
    assert_eq!(main_fn.kind, "exec");
    assert_eq!(main_fn.language, "rust");
    assert!(main_fn
        .dependencies
        .contains("probe:crate-a/1.0/lib/process()"));
    assert!(main_fn
        .dependencies
        .contains("probe:crate-b/1.0/helpers/compute()"));

    let process_fn = &merged["probe:crate-a/1.0/lib/process()"];
    assert_eq!(process_fn.display_name, "process");
    assert_eq!(process_fn.code_path, "crate-a/src/lib.rs");
    assert_eq!(process_fn.kind, "exec");
    assert!(process_fn
        .dependencies
        .contains("probe:crate-b/1.0/helpers/validate()"));

    let compute_fn = &merged["probe:crate-b/1.0/helpers/compute()"];
    assert_eq!(compute_fn.display_name, "compute");
    assert_eq!(compute_fn.code_path, "crate-b/src/helpers.rs");
    assert_eq!(compute_fn.code_text.lines_start, 5);
    assert_eq!(compute_fn.code_text.lines_end, 15);
    assert_eq!(compute_fn.kind, "spec");
    assert_eq!(compute_fn.code_module, "helpers");
    assert!(compute_fn
        .dependencies
        .contains("probe:crate-b/1.0/helpers/validate()"));

    let validate_fn = &merged["probe:crate-b/1.0/helpers/validate()"];
    assert_eq!(validate_fn.display_name, "validate");
    assert_eq!(validate_fn.code_path, "crate-b/src/helpers.rs");
    assert_eq!(validate_fn.kind, "proof");
    assert_eq!(validate_fn.code_module, "helpers");

    let internal_fn = &merged["probe:crate-b/1.0/helpers/internal()"];
    assert_eq!(internal_fn.display_name, "internal");
    assert_eq!(internal_fn.code_path, "crate-b/src/helpers.rs");
    assert_eq!(internal_fn.kind, "exec");
}

/// Stubs in atoms_a should be replaced by real atoms from atoms_b.
#[test]
fn test_atoms_stubs_replaced_with_real() {
    let atoms_a = load_input_atoms(&format!("{FIXTURES}/atoms_a.json"));
    let atoms_b = load_input_atoms(&format!("{FIXTURES}/atoms_b.json"));

    let compute_a = &atoms_a["probe:crate-b/1.0/helpers/compute()"];
    assert!(compute_a.is_stub(), "compute in atoms_a should be a stub");

    let compute_b = &atoms_b["probe:crate-b/1.0/helpers/compute()"];
    assert!(!compute_b.is_stub(), "compute in atoms_b should be real");

    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_probe_merge("atoms_a.json", "atoms_b.json", &output_path);

    let envelope = load_merged_atom_envelope(output_path.to_str().unwrap());
    let compute_merged = &envelope.data["probe:crate-b/1.0/helpers/compute()"];
    assert_eq!(compute_merged.code_path, "crate-b/src/helpers.rs");
    assert_eq!(compute_merged.kind, "spec");
}

/// Cross-project dependency edges should be preserved after atom merge.
#[test]
fn test_atoms_cross_project_edges_preserved() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_probe_merge("atoms_a.json", "atoms_b.json", &output_path);

    let envelope = load_merged_atom_envelope(output_path.to_str().unwrap());
    let merged = &envelope.data;

    assert!(merged["probe:crate-a/1.0/lib/main()"]
        .dependencies
        .contains("probe:crate-b/1.0/helpers/compute()"));
    assert!(merged["probe:crate-a/1.0/lib/process()"]
        .dependencies
        .contains("probe:crate-b/1.0/helpers/validate()"));
}

/// Provenance in atom merge output references both input files.
#[test]
fn test_atoms_provenance_recorded() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_probe_merge("atoms_a.json", "atoms_b.json", &output_path);

    let envelope = load_merged_atom_envelope(output_path.to_str().unwrap());

    assert_eq!(envelope.inputs.len(), 2);
    assert_eq!(envelope.inputs[0].schema, "probe-verus/atoms");
    assert_eq!(envelope.inputs[0].source.package, "crate-a");
    assert_eq!(envelope.inputs[1].schema, "probe-verus/atoms");
    assert_eq!(envelope.inputs[1].source.package, "crate-b");
}

// ===========================================================================
// Specs tests
// ===========================================================================

/// Merged specs file has the expected entries with last-wins on conflict.
///
/// specs_a has: main() (specified), process() (specified), compute() (not specified)
/// specs_b has: compute() (specified), validate() (specified), internal() (not specified)
///
/// Overlap: compute() appears in both. specs_b (last) should win, so merged
/// compute() should be specified=true.
#[test]
fn test_specs_merge_last_wins_on_conflict() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged_specs.json");
    run_probe_merge("specs_a.json", "specs_b.json", &output_path);

    let envelope = load_merged_generic_envelope(output_path.to_str().unwrap());

    assert_eq!(envelope.schema, "probe/merged-specs");
    assert_eq!(envelope.schema_version, "2.0");
    assert_eq!(envelope.tool.name, "probe");
    assert_eq!(envelope.tool.command, "merge");

    assert_eq!(
        envelope.data.len(),
        5,
        "Should have 5 spec entries after merge"
    );

    let compute = &envelope.data["probe:crate-b/1.0/helpers/compute()"];
    assert_eq!(
        compute["specified"], true,
        "compute() should be specified=true from specs_b (last wins)"
    );
    assert_eq!(compute["has_requires"], true);
    assert_eq!(compute["has_ensures"], true);

    let main_fn = &envelope.data["probe:crate-a/1.0/lib/main()"];
    assert_eq!(main_fn["specified"], true);

    let validate = &envelope.data["probe:crate-b/1.0/helpers/validate()"];
    assert_eq!(validate["specified"], true);
    assert_eq!(validate["context"], "impl");
}

/// Provenance in spec merge output references both input files.
#[test]
fn test_specs_provenance_recorded() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged_specs.json");
    run_probe_merge("specs_a.json", "specs_b.json", &output_path);

    let envelope = load_merged_generic_envelope(output_path.to_str().unwrap());

    assert_eq!(envelope.inputs.len(), 2);
    assert_eq!(envelope.inputs[0].schema, "probe-verus/specs");
    assert_eq!(envelope.inputs[0].source.package, "crate-a");
    assert_eq!(envelope.inputs[1].schema, "probe-verus/specs");
    assert_eq!(envelope.inputs[1].source.package, "crate-b");
}

/// New spec entries from the second file are added.
#[test]
fn test_specs_new_entries_added() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged_specs.json");
    run_probe_merge("specs_a.json", "specs_b.json", &output_path);

    let envelope = load_merged_generic_envelope(output_path.to_str().unwrap());

    assert!(
        envelope
            .data
            .contains_key("probe:crate-b/1.0/helpers/validate()"),
        "validate() from specs_b should be added"
    );
    assert!(
        envelope
            .data
            .contains_key("probe:crate-b/1.0/helpers/internal()"),
        "internal() from specs_b should be added"
    );
}

// ===========================================================================
// Proofs tests
// ===========================================================================

/// Merged proofs file uses last-wins: re-verification results override stale ones.
///
/// proofs_a has: main() (success), process() (failure), compute() (failure)
/// proofs_b has: compute() (success), validate() (success), internal() (success)
///
/// Overlap: compute() appears in both. proofs_b (last) wins, so merged
/// compute() should be verified=true, status="success".
#[test]
fn test_proofs_merge_last_wins_overrides_failure() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged_proofs.json");
    run_probe_merge("proofs_a.json", "proofs_b.json", &output_path);

    let envelope = load_merged_generic_envelope(output_path.to_str().unwrap());

    assert_eq!(envelope.schema, "probe/merged-proofs");
    assert_eq!(envelope.schema_version, "2.0");
    assert_eq!(envelope.tool.name, "probe");
    assert_eq!(envelope.tool.command, "merge");

    assert_eq!(
        envelope.data.len(),
        5,
        "Should have 5 proof entries after merge"
    );

    let compute = &envelope.data["probe:crate-b/1.0/helpers/compute()"];
    assert_eq!(
        compute["verified"], true,
        "compute() should be verified=true from proofs_b (last wins, was failure in proofs_a)"
    );
    assert_eq!(compute["status"], "success");

    let main_fn = &envelope.data["probe:crate-a/1.0/lib/main()"];
    assert_eq!(main_fn["verified"], true);
    assert_eq!(main_fn["status"], "success");

    let process_fn = &envelope.data["probe:crate-a/1.0/lib/process()"];
    assert_eq!(
        process_fn["verified"], false,
        "process() only in proofs_a, should keep its failure status"
    );
    assert_eq!(process_fn["status"], "failure");
}

/// Provenance in proof merge output references both input files.
#[test]
fn test_proofs_provenance_recorded() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged_proofs.json");
    run_probe_merge("proofs_a.json", "proofs_b.json", &output_path);

    let envelope = load_merged_generic_envelope(output_path.to_str().unwrap());

    assert_eq!(envelope.inputs.len(), 2);
    assert_eq!(envelope.inputs[0].schema, "probe-verus/proofs");
    assert_eq!(envelope.inputs[0].source.package, "crate-a");
    assert_eq!(envelope.inputs[1].schema, "probe-verus/proofs");
    assert_eq!(envelope.inputs[1].source.package, "crate-b");
}

/// New proof entries from the second file are added.
#[test]
fn test_proofs_new_entries_added() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged_proofs.json");
    run_probe_merge("proofs_a.json", "proofs_b.json", &output_path);

    let envelope = load_merged_generic_envelope(output_path.to_str().unwrap());

    assert!(
        envelope
            .data
            .contains_key("probe:crate-b/1.0/helpers/validate()"),
        "validate() from proofs_b should be added"
    );
    assert!(
        envelope
            .data
            .contains_key("probe:crate-b/1.0/helpers/internal()"),
        "internal() from proofs_b should be added"
    );
}

// ===========================================================================
// Cross-category rejection
// ===========================================================================

/// Mixing atoms and specs should fail.
#[test]
fn test_category_mismatch_rejected() {
    let binary = env!("CARGO_BIN_EXE_probe");
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("should_not_exist.json");

    let status = Command::new(binary)
        .args([
            "merge",
            &format!("{FIXTURES}/atoms_a.json"),
            &format!("{FIXTURES}/specs_b.json"),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run probe");

    assert!(
        !status.success(),
        "merge should fail when mixing atoms and specs"
    );
    assert!(
        !output_path.exists(),
        "output file should not be created on failure"
    );
}
