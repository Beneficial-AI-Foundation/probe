//! Integration tests for the `probe enrich` command.

use probe::types::Atom;
use std::collections::BTreeMap;
use std::process::Command;
use tempfile::TempDir;

const FIXTURE: &str = "tests/fixtures/propagate_test/atoms.json";

fn run_enrich(input: &str, output_path: &std::path::Path) {
    let binary = env!("CARGO_BIN_EXE_probe");
    let status = Command::new(binary)
        .args(["enrich", input, "-o", output_path.to_str().unwrap()])
        .status()
        .expect("Failed to run probe");
    assert!(status.success(), "enrich command failed for {input}");
}

fn load_atoms(path: &std::path::Path) -> BTreeMap<String, Atom> {
    let content = std::fs::read_to_string(path).expect("Failed to read output");
    let raw: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse output");
    let data = raw.get("data").expect("missing data field");
    serde_json::from_value(data.clone()).expect("failed to deserialize atoms")
}

fn get_vs(atom: &Atom) -> Option<&str> {
    atom.extensions
        .get("verification-status")
        .and_then(|v| v.as_str())
}

#[test]
fn test_transitive_chain_gets_transitively_verified() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let atoms = load_atoms(&out);

    // entry -> helper -> leaf: all verified, no bad deps → transitively-verified
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/entry()").unwrap()),
        Some("transitively-verified")
    );
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/helper()").unwrap()),
        Some("transitively-verified")
    );
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/leaf()").unwrap()),
        Some("transitively-verified")
    );
}

#[test]
fn test_caller_of_unverified_stays_verified() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let atoms = load_atoms(&out);

    // caller -> broken (unverified) → stays "verified" (locally verified only)
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/caller()").unwrap()),
        Some("verified")
    );
    // broken itself stays unverified
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/broken()").unwrap()),
        Some("unverified")
    );
}

#[test]
fn test_trusted_dep_does_not_block() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let atoms = load_atoms(&out);

    // uses_trusted -> axiom (trusted) → transitively-verified
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/uses_trusted()").unwrap()),
        Some("transitively-verified")
    );
}

#[test]
fn test_missing_dep_does_not_block() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let atoms = load_atoms(&out);

    // uses_external -> probe:std/alloc() (not in map, treated as trusted)
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/uses_external()").unwrap()),
        Some("transitively-verified")
    );
}

#[test]
fn test_cycle_with_unverified_dep_stays_verified() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let atoms = load_atoms(&out);

    // cycle_a -> cycle_b -> cycle_a (cycle), cycle_b -> broken (unverified)
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/cycle_a()").unwrap()),
        Some("verified")
    );
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/cycle_b()").unwrap()),
        Some("verified")
    );
}

#[test]
fn test_missing_status_does_not_contaminate() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let atoms = load_atoms(&out);

    // calls_untracked -> plain_rust (no verification-status at all)
    // plain_rust is untracked/Grey — should NOT contaminate
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/calls_untracked()").unwrap()),
        Some("transitively-verified")
    );
    assert_eq!(
        get_vs(atoms.get("probe:test/1.0/plain_rust()").unwrap()),
        None
    );
}

#[test]
fn test_envelope_structure_preserved() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("output.json");
    run_enrich(FIXTURE, &out);

    let content = std::fs::read_to_string(&out).expect("Failed to read output");
    let raw: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse output");

    assert_eq!(
        raw.get("schema").unwrap().as_str().unwrap(),
        "probe-verus/atoms"
    );
    assert_eq!(raw.get("schema-version").unwrap().as_str().unwrap(), "2.0");
    assert!(
        raw.get("source").is_some(),
        "source field should be preserved"
    );
    assert!(raw.get("tool").is_some(), "tool field should be preserved");
    assert!(
        raw.get("timestamp").is_some(),
        "timestamp should be preserved"
    );
}
