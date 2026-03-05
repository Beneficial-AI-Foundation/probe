//! Integration tests for the merge-atoms command.
//!
//! Ported from probe-verus's tests/merge_atoms.rs, adapted for Schema 2.0 envelopes.
//! Uses the same logical fixture data:
//! - atoms_a.json: crate-a with stubs for crate-b functions
//! - atoms_b.json: crate-b with real function entries

use probe::types::{Atom, MergedAtomEnvelope};
use std::collections::BTreeMap;
use std::process::Command;
use tempfile::TempDir;

const FIXTURES: &str = "tests/fixtures/merge_test";

fn load_merged_envelope(path: &str) -> MergedAtomEnvelope {
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

fn run_merge(output_path: &std::path::Path) {
    let binary = env!("CARGO_BIN_EXE_probe");
    let status = Command::new(binary)
        .args([
            "merge-atoms",
            &format!("{FIXTURES}/atoms_a.json"),
            &format!("{FIXTURES}/atoms_b.json"),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run probe");
    assert!(status.success(), "merge-atoms command failed");
}

/// Expected merged data matches probe-verus's atoms_combined.json content.
///
/// The expected result after merging atoms_a (base) and atoms_b (incoming):
/// - main() and process() from crate-a are kept (real atoms in base)
/// - compute() stub in atoms_a is replaced by real atom from atoms_b
/// - validate() stub in atoms_a is replaced by real atom from atoms_b
/// - internal() from atoms_b is added (new atom not in base)
#[test]
fn test_merge_fixtures_match_expected() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");

    run_merge(&output_path);

    let envelope = load_merged_envelope(output_path.to_str().unwrap());
    let merged = &envelope.data;

    assert_eq!(envelope.schema, "probe/merged-atoms");
    assert_eq!(envelope.schema_version, "2.0");
    assert_eq!(envelope.tool.name, "probe");
    assert_eq!(envelope.tool.command, "merge-atoms");
    assert_eq!(envelope.inputs.len(), 2);

    assert_eq!(merged.len(), 5, "Should have 5 atoms after merge");

    // Verify each atom matches expected values (same as probe-verus atoms_combined.json)
    let main_fn = &merged["probe:crate-a/1.0/lib/main()"];
    assert_eq!(main_fn.display_name, "main");
    assert_eq!(main_fn.code_path, "crate-a/src/lib.rs");
    assert_eq!(main_fn.code_text.lines_start, 1);
    assert_eq!(main_fn.code_text.lines_end, 10);
    assert_eq!(main_fn.kind, "exec");
    assert_eq!(main_fn.language, "rust");
    assert!(main_fn.dependencies.contains("probe:crate-a/1.0/lib/process()"));
    assert!(main_fn.dependencies.contains("probe:crate-b/1.0/helpers/compute()"));

    let process_fn = &merged["probe:crate-a/1.0/lib/process()"];
    assert_eq!(process_fn.display_name, "process");
    assert_eq!(process_fn.code_path, "crate-a/src/lib.rs");
    assert_eq!(process_fn.kind, "exec");
    assert!(process_fn.dependencies.contains("probe:crate-b/1.0/helpers/validate()"));

    let compute_fn = &merged["probe:crate-b/1.0/helpers/compute()"];
    assert_eq!(compute_fn.display_name, "compute");
    assert_eq!(compute_fn.code_path, "crate-b/src/helpers.rs");
    assert_eq!(compute_fn.code_text.lines_start, 5);
    assert_eq!(compute_fn.code_text.lines_end, 15);
    assert_eq!(compute_fn.kind, "spec");
    assert_eq!(compute_fn.code_module, "helpers");
    assert!(compute_fn.dependencies.contains("probe:crate-b/1.0/helpers/validate()"));

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
fn test_merge_stubs_replaced_with_real_atoms() {
    let atoms_a = load_input_atoms(&format!("{FIXTURES}/atoms_a.json"));
    let atoms_b = load_input_atoms(&format!("{FIXTURES}/atoms_b.json"));

    let compute_a = &atoms_a["probe:crate-b/1.0/helpers/compute()"];
    assert!(compute_a.is_stub(), "compute in atoms_a should be a stub");

    let compute_b = &atoms_b["probe:crate-b/1.0/helpers/compute()"];
    assert!(!compute_b.is_stub(), "compute in atoms_b should be real");

    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_merge(&output_path);

    let envelope = load_merged_envelope(output_path.to_str().unwrap());
    let compute_merged = &envelope.data["probe:crate-b/1.0/helpers/compute()"];
    assert_eq!(compute_merged.code_path, "crate-b/src/helpers.rs");
    assert_eq!(compute_merged.kind, "spec");
}

/// Cross-project dependency edges should be preserved after merge.
#[test]
fn test_merge_cross_project_edges_preserved() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_merge(&output_path);

    let envelope = load_merged_envelope(output_path.to_str().unwrap());
    let merged = &envelope.data;

    let main_fn = &merged["probe:crate-a/1.0/lib/main()"];
    assert!(
        main_fn.dependencies.contains("probe:crate-b/1.0/helpers/compute()"),
        "main() should depend on compute()"
    );

    let process_fn = &merged["probe:crate-a/1.0/lib/process()"];
    assert!(
        process_fn.dependencies.contains("probe:crate-b/1.0/helpers/validate()"),
        "process() should depend on validate()"
    );
}

/// Provenance in the output envelope should reference both input files.
#[test]
fn test_merge_provenance_recorded() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("merged.json");
    run_merge(&output_path);

    let envelope = load_merged_envelope(output_path.to_str().unwrap());

    assert_eq!(envelope.inputs.len(), 2);
    assert_eq!(envelope.inputs[0].schema, "probe-verus/atoms");
    assert_eq!(envelope.inputs[0].source.package, "crate-a");
    assert_eq!(envelope.inputs[1].schema, "probe-verus/atoms");
    assert_eq!(envelope.inputs[1].source.package, "crate-b");
}
