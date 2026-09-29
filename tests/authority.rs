//! Integration tests for the ADR-006 authority boundary: projection
//! rejection and the per-producer version gate, wired into `probe merge`,
//! `probe enrich`, `probe summary`, and `probe project`.
//!
//! Plan test names (merge-soundness-fix-plan.md §3):
//! - test f: merge and enrich reject both projection formats
//! - test l: legacy gate — pre-threshold envelopes rejected, post-threshold
//!   and ungated accepted, legacy Verus-composer shape rejected
//! - test m: an Aeneas-shaped composed envelope round-trips
//!   load → merge → serialize → validate with its inventory intact
//! - test n: read-only gate — summary/project reject pre-threshold inputs;
//!   projections stay readable

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn atom(deps: &[&str]) -> serde_json::Value {
    json!({
        "display-name": "f",
        "dependencies": deps,
        "code-module": "m",
        "code-path": "src/lib.rs",
        "code-text": { "lines-start": 1, "lines-end": 10 },
        "kind": "exec",
        "language": "rust"
    })
}

fn source(language: &str, package: &str) -> serde_json::Value {
    json!({
        "repo": "https://github.com/org/repo",
        "commit": "abc123",
        "language": language,
        "package": package,
        "package-version": "1.0"
    })
}

/// Single-tool atoms envelope (`source` provenance).
fn single_tool_envelope(
    schema: &str,
    tool_name: &str,
    tool_version: &str,
    key: &str,
) -> serde_json::Value {
    json!({
        "schema": schema,
        "schema-version": "3.0",
        "tool": { "name": tool_name, "version": tool_version, "command": "extract" },
        "source": source("rust", "pkg"),
        "timestamp": "2026-01-01T00:00:00Z",
        "data": { key: atom(&[]) }
    })
}

/// Composed atoms envelope (`inputs` provenance).
fn composed_envelope(schema: &str, tool: serde_json::Value, key: &str) -> serde_json::Value {
    json!({
        "schema": schema,
        "schema-version": "3.0",
        "tool": tool,
        "inputs": [
            { "schema": "probe-rust/extract", "source": source("rust", "pkg-rust") },
            { "schema": "probe-lean/extract", "source": source("lean", "pkg-lean") }
        ],
        "timestamp": "2026-01-01T00:00:00Z",
        "data": { key: atom(&[]) }
    })
}

fn projection_block() -> serde_json::Value {
    json!({
        "mappings-file": "mappings.json",
        "seeds": 1,
        "forward-depth": 2,
        "reverse-depth": 0,
        "atoms-in": 10,
        "atoms-out": 1,
        "deps-trimmed": 3
    })
}

fn write_json(dir: &Path, name: &str, value: &serde_json::Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    path
}

/// Run the probe binary; return (success, stderr).
fn run_probe(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_probe"))
        .args(args)
        .output()
        .expect("failed to run probe");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// A known-good merge partner (ungated probe-verus extract).
fn good_partner(dir: &Path) -> PathBuf {
    write_json(
        dir,
        "partner.json",
        &single_tool_envelope(
            "probe-verus/atoms",
            "probe-verus",
            "2.0.0",
            "probe:partner/1.0/p()",
        ),
    )
}

fn assert_merge_rejects(dir: &Path, input: &Path, needle: &str) {
    let partner = good_partner(dir);
    let out = dir.join("out.json");
    let (ok, stderr) = run_probe(&[
        "merge",
        input.to_str().unwrap(),
        partner.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(!ok, "merge should reject {}", input.display());
    assert!(
        stderr.contains(needle),
        "stderr should mention {needle:?}: {stderr}"
    );
}

fn assert_enrich_rejects(dir: &Path, input: &Path, needle: &str) {
    let out = dir.join("enriched.json");
    let (ok, stderr) = run_probe(&[
        "enrich",
        input.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(!ok, "enrich should reject {}", input.display());
    assert!(
        stderr.contains(needle),
        "stderr should mention {needle:?}: {stderr}"
    );
    assert!(
        !out.exists(),
        "enrich must not write output when rejecting {}",
        input.display()
    );
}

// ---------------------------------------------------------------------------
// Test f: merge and enrich reject both projection formats
// ---------------------------------------------------------------------------

#[test]
fn merge_and_enrich_reject_projected_atoms_schema() {
    let tmp = TempDir::new().unwrap();
    let mut projected = composed_envelope(
        "probe/projected-atoms",
        json!({ "name": "probe", "version": "0.5.0", "command": "project" }),
        "probe:a/1.0/f()",
    );
    projected["projection"] = projection_block();
    let path = write_json(tmp.path(), "projected.json", &projected);

    // Message-specific needles: the fixture path itself contains "projected",
    // so a generic substring could not attribute the rejection to the
    // projection predicate.
    assert_merge_rejects(tmp.path(), &path, "projected input (schema");
    assert_enrich_rejects(tmp.path(), &path, "projected input (schema");
}

#[test]
fn merge_and_enrich_reject_legacy_projection_format() {
    let tmp = TempDir::new().unwrap();
    // The actual legacy shape: probe/merged-atoms plus a projection field.
    // tool.version is in the accepted probe interval, so the rejection is
    // attributable to the projection predicate alone.
    let mut legacy = composed_envelope(
        "probe/merged-atoms",
        json!({ "name": "probe", "version": "0.5.0", "command": "project" }),
        "probe:a/1.0/f()",
    );
    legacy["projection"] = projection_block();
    let path = write_json(tmp.path(), "legacy_projection.json", &legacy);

    // Message-specific needles: the fixture path itself contains
    // "projection", so a generic substring could not attribute the rejection
    // to the legacy-projection predicate.
    assert_merge_rejects(tmp.path(), &path, "legacy projection rejected");
    assert_enrich_rejects(tmp.path(), &path, "legacy projection rejected");
}

// ---------------------------------------------------------------------------
// Test l: the per-producer version gate
// ---------------------------------------------------------------------------

#[test]
fn gate_rejects_pre_threshold_probe_lean_extract() {
    let tmp = TempDir::new().unwrap();
    let path = write_json(
        tmp.path(),
        "lean_old.json",
        &single_tool_envelope("probe-lean/extract", "probe-lean", "0.15.0", "probe:L.f"),
    );
    assert_merge_rejects(tmp.path(), &path, "pre-contract");
    assert_enrich_rejects(tmp.path(), &path, "pre-contract");
}

#[test]
fn gate_rejects_pre_threshold_hub_merged_artifact() {
    let tmp = TempDir::new().unwrap();
    let path = write_json(
        tmp.path(),
        "old_merge.json",
        &composed_envelope(
            "probe/merged-atoms",
            json!({ "name": "probe", "version": "0.4.0", "command": "merge" }),
            "probe:a/1.0/f()",
        ),
    );
    assert_merge_rejects(tmp.path(), &path, "pre-contract");
    assert_enrich_rejects(tmp.path(), &path, "pre-contract");
}

#[test]
fn gate_rejects_pre_threshold_aeneas_composed_extract() {
    let tmp = TempDir::new().unwrap();
    let path = write_json(
        tmp.path(),
        "aeneas_old.json",
        &composed_envelope(
            "probe-aeneas/extract",
            json!({ "name": "probe-aeneas", "version": "0.20.0", "command": "extract" }),
            "probe:a/1.0/f()",
        ),
    );
    assert_merge_rejects(tmp.path(), &path, "pre-contract");
}

#[test]
fn gate_rejects_pre_threshold_vcvio_reemitted_extract() {
    let tmp = TempDir::new().unwrap();
    let path = write_json(
        tmp.path(),
        "vcvio_old.json",
        &single_tool_envelope("probe-vcvio/extract", "probe-vcvio", "0.1.0", "probe:V.f"),
    );
    assert_merge_rejects(tmp.path(), &path, "pre-contract");
}

#[test]
fn gate_rejects_legacy_verus_composer_shape() {
    let tmp = TempDir::new().unwrap();
    // The real legacy shape: tool.name "probe" at probe-verus's own 8.x
    // package version with the merge-atoms command the hub never shipped.
    let path = write_json(
        tmp.path(),
        "verus_composer.json",
        &composed_envelope(
            "probe/merged-atoms",
            json!({ "name": "probe", "version": "8.0.1", "command": "merge-atoms" }),
            "probe:a/1.0/f()",
        ),
    );
    assert_merge_rejects(tmp.path(), &path, "merge-atoms");
    assert_enrich_rejects(tmp.path(), &path, "merge-atoms");

    // The command check also catches an in-interval version on its own.
    let in_interval = write_json(
        tmp.path(),
        "verus_composer_in_interval.json",
        &composed_envelope(
            "probe/merged-atoms",
            json!({ "name": "probe", "version": "0.6.0", "command": "merge-atoms" }),
            "probe:a/1.0/f()",
        ),
    );
    assert_merge_rejects(tmp.path(), &in_interval, "merge-atoms");
}

#[test]
fn gate_accepts_post_threshold_and_ungated_envelopes() {
    let tmp = TempDir::new().unwrap();
    // Post-threshold gated producers plus an ungated probe-verus extract all
    // merge cleanly (pairwise with the good partner).
    let inputs = [
        single_tool_envelope("probe-lean/extract", "probe-lean", "0.16.0", "probe:L.f"),
        single_tool_envelope("probe-vcvio/extract", "probe-vcvio", "0.2.0", "probe:V.f"),
        composed_envelope(
            "probe-aeneas/extract",
            json!({ "name": "probe-aeneas", "version": "0.21.0", "command": "extract" }),
            "probe:a/1.0/f()",
        ),
        composed_envelope(
            "probe/merged-atoms",
            json!({ "name": "probe", "version": "0.5.0", "command": "merge" }),
            "probe:b/1.0/g()",
        ),
        single_tool_envelope(
            "probe-verus/atoms",
            "probe-verus",
            "8.0.1",
            "probe:x/1.0/v()",
        ),
    ];
    for (i, envelope) in inputs.iter().enumerate() {
        let input = write_json(tmp.path(), &format!("in_{i}.json"), envelope);
        let partner = good_partner(tmp.path());
        let out = tmp.path().join(format!("out_{i}.json"));
        let (ok, stderr) = run_probe(&[
            "merge",
            input.to_str().unwrap(),
            partner.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ]);
        assert!(ok, "merge should accept input {i}: {stderr}");
    }
}

#[test]
fn enrich_accepts_post_threshold_probe_lean_extract() {
    let tmp = TempDir::new().unwrap();
    let path = write_json(
        tmp.path(),
        "lean_new.json",
        &single_tool_envelope("probe-lean/extract", "probe-lean", "0.16.0", "probe:L.f"),
    );
    let out = tmp.path().join("enriched.json");
    let (ok, stderr) = run_probe(&[
        "enrich",
        path.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        ok,
        "enrich should accept a post-threshold extract: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Test m: Aeneas-shaped composed envelope round-trip
// ---------------------------------------------------------------------------

#[test]
fn aeneas_composed_envelope_inventory_survives_merge() {
    let tmp = TempDir::new().unwrap();
    let aeneas = write_json(
        tmp.path(),
        "aeneas.json",
        &composed_envelope(
            "probe-aeneas/extract",
            json!({ "name": "probe-aeneas", "version": "0.21.0", "command": "extract" }),
            "probe:a/1.0/f()",
        ),
    );
    let lean = write_json(
        tmp.path(),
        "lean.json",
        &single_tool_envelope("probe-lean/extract", "probe-lean", "0.16.0", "probe:L.g"),
    );
    let out = tmp.path().join("merged.json");
    let (ok, stderr) = run_probe(&[
        "merge",
        aeneas.to_str().unwrap(),
        lean.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "merge should succeed: {stderr}");

    let merged: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();

    // Both Aeneas source entries survive into the merged inventory (P9) —
    // the old probe/merged- prefix check silently discarded them.
    let inputs = merged["inputs"].as_array().unwrap();
    let packages: Vec<&str> = inputs
        .iter()
        .map(|p| p["source"]["package"].as_str().unwrap())
        .collect();
    assert!(
        packages.contains(&"pkg-rust"),
        "Rust inventory entry survives: {packages:?}"
    );
    assert!(
        packages.contains(&"pkg-lean"),
        "Lean inventory entry survives: {packages:?}"
    );
    assert_eq!(inputs.len(), 3, "two Aeneas entries + one Lean entry");

    // The merged envelope validates against the executable schema.
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("schemas/atom-envelope.schema.json").unwrap(),
    )
    .unwrap();
    let validator = jsonschema::Validator::new(&schema).unwrap();
    assert!(
        validator.validate(&merged).is_ok(),
        "merged envelope should validate: {:?}",
        validator.validate(&merged)
    );
}

// ---------------------------------------------------------------------------
// Test n: read-only consumers run the version gate; projections stay readable
// ---------------------------------------------------------------------------

fn mappings_file(dir: &Path) -> PathBuf {
    write_json(
        dir,
        "mappings.json",
        &json!({
            "schema": "probe/mappings",
            "schema-version": "3.0",
            "mappings": [
                { "from": "probe:a/1.0/f()", "to": "probe:A.f", "confidence": "exact" }
            ]
        }),
    )
}

#[test]
fn summary_and_project_reject_pre_threshold_aeneas_envelope() {
    let tmp = TempDir::new().unwrap();
    let path = write_json(
        tmp.path(),
        "aeneas_old.json",
        &composed_envelope(
            "probe-aeneas/extract",
            json!({ "name": "probe-aeneas", "version": "0.20.0", "command": "extract" }),
            "probe:a/1.0/f()",
        ),
    );

    let out = tmp.path().join("summary.json");
    let (ok, stderr) = run_probe(&[
        "summary",
        path.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(!ok, "summary should reject a pre-threshold envelope");
    assert!(stderr.contains("pre-contract"), "{stderr}");

    let mappings = mappings_file(tmp.path());
    let out = tmp.path().join("projected.json");
    let (ok, stderr) = run_probe(&[
        "project",
        path.to_str().unwrap(),
        "-m",
        mappings.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(!ok, "project should reject a pre-threshold envelope");
    assert!(stderr.contains("pre-contract"), "{stderr}");
}

#[test]
fn summary_reads_a_modern_projection() {
    let tmp = TempDir::new().unwrap();
    // A projection as the contract release will produce it. Its atoms carry
    // status-origin: "translation" — the imported-verified listing itself
    // lands with the PR 2 summary contract; here we pin readability.
    let mut projected = composed_envelope(
        "probe/projected-atoms",
        json!({ "name": "probe", "version": "0.5.0", "command": "project" }),
        "probe:a/1.0/f()",
    );
    projected["projection"] = projection_block();
    projected["data"]["probe:a/1.0/f()"]["verification-status"] = json!("verified");
    projected["data"]["probe:a/1.0/f()"]["status-origin"] = json!("translation");
    let path = write_json(tmp.path(), "projected.json", &projected);

    let out = tmp.path().join("summary.json");
    let (ok, stderr) = run_probe(&[
        "summary",
        path.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "summary should read a modern projection: {stderr}");
}

#[test]
fn project_reads_an_already_projected_input() {
    let tmp = TempDir::new().unwrap();
    let mut projected = composed_envelope(
        "probe/projected-atoms",
        json!({ "name": "probe", "version": "0.5.0", "command": "project" }),
        "probe:a/1.0/f()",
    );
    projected["projection"] = projection_block();
    let path = write_json(tmp.path(), "projected.json", &projected);

    let mappings = mappings_file(tmp.path());
    let out = tmp.path().join("reprojected.json");
    let (ok, stderr) = run_probe(&[
        "project",
        path.to_str().unwrap(),
        "-m",
        mappings.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        ok,
        "project should read an already-projected input: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Loader provenance shape: ambiguous and empty inventories are rejected
// ---------------------------------------------------------------------------

#[test]
fn merge_rejects_envelope_with_both_source_and_inputs() {
    let tmp = TempDir::new().unwrap();
    // Post-threshold tool, so the rejection is attributable to the ambiguous
    // provenance shape alone (the executable schema's branches match neither).
    let mut ambiguous = composed_envelope(
        "probe-aeneas/extract",
        json!({ "name": "probe-aeneas", "version": "0.21.0", "command": "extract" }),
        "probe:a/1.0/f()",
    );
    ambiguous["source"] = source("rust", "pkg");
    let path = write_json(tmp.path(), "ambiguous.json", &ambiguous);

    assert_merge_rejects(tmp.path(), &path, "ambiguous provenance");
    assert_enrich_rejects(tmp.path(), &path, "ambiguous provenance");
}

#[test]
fn project_rejects_empty_inputs_inventory() {
    let tmp = TempDir::new().unwrap();
    // An empty composed inventory must fail at load — previously it passed
    // through and produced output violating the schema's `inputs` minItems.
    let mut empty = composed_envelope(
        "probe/merged-atoms",
        json!({ "name": "probe", "version": "0.5.0", "command": "merge" }),
        "probe:a/1.0/f()",
    );
    empty["inputs"] = json!([]);
    let path = write_json(tmp.path(), "empty_inputs.json", &empty);

    let mappings = mappings_file(tmp.path());
    let out = tmp.path().join("projected.json");
    let (ok, stderr) = run_probe(&[
        "project",
        path.to_str().unwrap(),
        "-m",
        mappings.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(!ok, "project should reject an empty inputs inventory");
    assert!(stderr.contains("empty \"inputs\""), "{stderr}");

    assert_merge_rejects(tmp.path(), &path, "empty \"inputs\"");
}

// ---------------------------------------------------------------------------
// Enrich input validation: unsupported schema strings and non-atoms
// categories are rejected at the binary level (both checks moved into
// parse_envelope / cmd_enrich by ADR-006)
// ---------------------------------------------------------------------------

#[test]
fn enrich_rejects_unsupported_schema_string() {
    let tmp = TempDir::new().unwrap();
    // Post-threshold tool, so the rejection is attributable to the schema
    // string alone.
    let path = write_json(
        tmp.path(),
        "bogus_schema.json",
        &single_tool_envelope(
            "probe-verus/bogus",
            "probe-verus",
            "2.0.0",
            "probe:x/1.0/f()",
        ),
    );
    assert_enrich_rejects(tmp.path(), &path, "unsupported schema");
}

#[test]
fn enrich_rejects_specs_envelope() {
    let tmp = TempDir::new().unwrap();
    // A valid specs-category envelope: parse_envelope accepts it, but enrich
    // only recomputes over atoms.
    let specs = json!({
        "schema": "probe-verus/specs",
        "schema-version": "3.0",
        "tool": { "name": "probe-verus", "version": "2.0.0", "command": "specify" },
        "source": source("rust", "pkg"),
        "timestamp": "2026-01-01T00:00:00Z",
        "data": {
            "probe:pkg/1.0/f()": {
                "specified": true,
                "code-path": "src/lib.rs",
                "spec-text": { "lines-start": 1, "lines-end": 3 },
                "kind": "exec",
                "has_requires": true,
                "has_ensures": true,
                "context": "standalone"
            }
        }
    });
    let path = write_json(tmp.path(), "specs.json", &specs);
    assert_enrich_rejects(tmp.path(), &path, "expected atoms schema, got specs");
}

// ---------------------------------------------------------------------------
// Projected output carries the distinct schema and validates
// ---------------------------------------------------------------------------

#[test]
fn project_output_uses_projected_atoms_schema_and_validates() {
    let tmp = TempDir::new().unwrap();
    let input = write_json(
        tmp.path(),
        "atoms.json",
        &single_tool_envelope(
            "probe-verus/atoms",
            "probe-verus",
            "2.0.0",
            "probe:a/1.0/f()",
        ),
    );
    let mappings = mappings_file(tmp.path());
    let out = tmp.path().join("projected.json");
    let (ok, stderr) = run_probe(&[
        "project",
        input.to_str().unwrap(),
        "-m",
        mappings.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "project should succeed: {stderr}");

    let projected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(projected["schema"], "probe/projected-atoms");
    assert!(
        projected.get("projection").is_some(),
        "projection block present"
    );

    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("schemas/atom-envelope.schema.json").unwrap(),
    )
    .unwrap();
    let validator = jsonschema::Validator::new(&schema).unwrap();
    assert!(
        validator.validate(&projected).is_ok(),
        "projected envelope should validate: {:?}",
        validator.validate(&projected)
    );

    // And the projection is rejected if fed back into merge/enrich.
    // Message-specific needles: the output path contains "projected", and
    // this build's output is also gate-rejected (pre-contract tool version),
    // so only the exact projection-predicate message attributes the rejection.
    assert_merge_rejects(tmp.path(), &out, "projected input (schema");
    assert_enrich_rejects(tmp.path(), &out, "projected input (schema");
}
