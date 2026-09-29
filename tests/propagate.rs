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

/// Run `probe enrich` without asserting success; returns (success, stderr).
fn run_enrich_raw(input: &str, output_path: &std::path::Path) -> (bool, String) {
    let binary = env!("CARGO_BIN_EXE_probe");
    let out = Command::new(binary)
        .args(["enrich", input, "-o", output_path.to_str().unwrap()])
        .output()
        .expect("Failed to run probe");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
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

/// Build a minimal ungated (probe-verus) atoms envelope around `data`.
fn envelope_with(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema": "probe-verus/atoms",
        "schema-version": "3.0",
        "tool": { "name": "probe-verus", "version": "5.0.0", "command": "extract" },
        "source": {
            "repo": "https://github.com/example/test-crate",
            "commit": "abcdef1234567890abcdef1234567890abcdef12",
            "language": "rust",
            "package": "test-crate",
            "package-version": "1.0.0"
        },
        "timestamp": "2026-05-12T10:00:00Z",
        "data": data
    })
}

fn atom_json(status: Option<&str>, deps: &[&str]) -> serde_json::Value {
    let mut atom = serde_json::json!({
        "display-name": "f",
        "dependencies": deps,
        "code-module": "",
        "code-path": "src/lib.rs",
        "code-text": { "lines-start": 1, "lines-end": 10 },
        "kind": "exec",
        "language": "rust"
    });
    if let Some(s) = status {
        atom["verification-status"] = serde_json::json!(s);
    }
    atom
}

// Recomputation end to end: a stale "transitively-verified" label whose dep
// is unverified is downgraded by `probe enrich` (P23 — enrichment recomputes
// rather than upgrades).
#[test]
fn test_stale_transitive_label_is_downgraded() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("stale.json");
    let out = tmp.path().join("output.json");

    let envelope = envelope_with(serde_json::json!({
        "probe:t/1.0/f()": atom_json(Some("transitively-verified"), &["probe:t/1.0/bad()"]),
        "probe:t/1.0/bad()": atom_json(Some("unverified"), &[]),
    }));
    std::fs::write(&input, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();

    run_enrich(input.to_str().unwrap(), &out);
    let atoms = load_atoms(&out);
    assert_eq!(
        get_vs(atoms.get("probe:t/1.0/f()").unwrap()),
        Some("verified")
    );
}

// status-origin blocker seeds end to end: a kernel-taint "verified" leaf is
// never promoted, and neither is its locally verified caller (ADR-006).
#[test]
fn test_status_origin_blocks_promotion_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("marked.json");
    let out = tmp.path().join("output.json");

    let mut leaf = atom_json(Some("verified"), &[]);
    leaf["status-origin"] = serde_json::json!("kernel-taint");
    let envelope = envelope_with(serde_json::json!({
        "probe:t/1.0/own_sorry()": leaf,
        "probe:t/1.0/caller()": atom_json(Some("verified"), &["probe:t/1.0/own_sorry()"]),
    }));
    std::fs::write(&input, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();

    run_enrich(input.to_str().unwrap(), &out);
    let atoms = load_atoms(&out);
    assert_eq!(
        get_vs(atoms.get("probe:t/1.0/own_sorry()").unwrap()),
        Some("verified")
    );
    assert_eq!(
        get_vs(atoms.get("probe:t/1.0/caller()").unwrap()),
        Some("verified")
    );
}

// Carrier preparation end to end: `probe enrich` normalizes before
// recomputation, so a dotted-alias dependency reaches its contamination
// source instead of dangling (ADR-006 — the dotted-alias regression).
#[test]
fn test_dotted_alias_contamination_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("dotted.json");
    let out = tmp.path().join("output.json");

    let envelope = envelope_with(serde_json::json!({
        "probe:t/1.0/f()": atom_json(Some("verified"), &["probe:t/1.0/g()."]),
        "probe:t/1.0/g()": atom_json(Some("failed"), &[]),
    }));
    std::fs::write(&input, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();

    run_enrich(input.to_str().unwrap(), &out);
    let atoms = load_atoms(&out);
    assert_eq!(
        get_vs(atoms.get("probe:t/1.0/f()").unwrap()),
        Some("verified")
    );
}

// P8 fail-closed at the unary boundary: a normalization collision between two
// distinct real atoms would silently discard one atom's evidence, so `probe
// enrich` refuses the input instead of writing output.
#[test]
fn test_enrich_rejects_normalization_collision() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("collision.json");
    let out = tmp.path().join("output.json");

    let envelope = envelope_with(serde_json::json!({
        "probe:t/1.0/g()": atom_json(Some("verified"), &[]),
        "probe:t/1.0/g().": atom_json(Some("failed"), &[]),
        "probe:t/1.0/caller()": atom_json(Some("verified"), &["probe:t/1.0/g()."]),
    }));
    std::fs::write(&input, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();

    // Pre-existing output must survive a rejection untouched.
    std::fs::write(&out, "sentinel").unwrap();

    let (ok, stderr) = run_enrich_raw(input.to_str().unwrap(), &out);
    assert!(
        !ok,
        "enrich must reject a distinct-atom collision: {stderr}"
    );
    assert!(
        stderr.contains("probe:t/1.0/g().") && stderr.contains("probe:t/1.0/g()"),
        "collision message should name both keys: {stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        "sentinel",
        "rejection must not clobber existing output"
    );
}

// The benign halves of the collision rule stay accepted end to end: an
// identical duplicate collapses (dedup) and a stub alias is absorbed —
// neither is a distinct-atom collision (P8).
#[test]
fn test_enrich_accepts_identical_duplicate_and_stub_collisions() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("benign.json");
    let out = tmp.path().join("output.json");

    let mut stub = atom_json(Some("verified"), &[]);
    stub["code-path"] = serde_json::json!("");
    stub["code-text"] = serde_json::json!({ "lines-start": 0, "lines-end": 0 });
    let envelope = envelope_with(serde_json::json!({
        // identical duplicate pair
        "probe:t/1.0/g()": atom_json(Some("verified"), &[]),
        "probe:t/1.0/g().": atom_json(Some("verified"), &[]),
        // stub alias absorbed by the real atom
        "probe:t/1.0/h()": atom_json(Some("verified"), &[]),
        "probe:t/1.0/h().": stub,
    }));
    std::fs::write(&input, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();

    run_enrich(input.to_str().unwrap(), &out);
    let atoms = load_atoms(&out);
    assert_eq!(atoms.len(), 2);
    assert_eq!(
        get_vs(atoms.get("probe:t/1.0/g()").unwrap()),
        Some("transitively-verified")
    );
    assert!(!atoms.get("probe:t/1.0/h()").unwrap().code_path.is_empty());
}

// ADR-006 Decision 2 fail-closed: a status-origin outside the two-value enum
// (here a non-string, which would otherwise silently read as absent) is
// rejected at the load boundary.
#[test]
fn test_enrich_rejects_invalid_status_origin() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("badmarker.json");
    let out = tmp.path().join("output.json");

    let mut leaf = atom_json(Some("verified"), &[]);
    leaf["status-origin"] = serde_json::json!(null);
    let envelope = envelope_with(serde_json::json!({
        "probe:t/1.0/leaf()": leaf,
    }));
    std::fs::write(&input, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();

    let (ok, stderr) = run_enrich_raw(input.to_str().unwrap(), &out);
    assert!(
        !ok,
        "enrich must reject an out-of-enum status-origin: {stderr}"
    );
    assert!(
        stderr.contains("invalid status-origin"),
        "rejection should name the marker: {stderr}"
    );
    assert!(!out.exists(), "no output on rejection");
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
    assert_eq!(raw.get("schema-version").unwrap().as_str().unwrap(), "3.0");
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
