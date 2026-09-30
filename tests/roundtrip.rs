//! Contract-release round-trip tests over **real binary outputs** (plan §9
//! row 3c, ADR-006 Decision 7).
//!
//! The authority tests construct hypothetical inputs; these run the actual
//! pipeline: what `probe merge`/`probe project` write today must pass the
//! hub's own gate in `probe enrich`/`probe summary`/`probe project`, and a
//! real projection must be rejected by the recomputation boundaries on the
//! projection predicate specifically (not the version gate). Real outputs
//! can only satisfy the gate at the 0.5.0 contract release — which is
//! exactly what these tests pin.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run_probe(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_probe"))
        .args(args)
        .output()
        .expect("failed to run probe")
}

fn write_json(path: &Path, value: &serde_json::Value) {
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn atom(deps: &[&str], status: Option<&str>) -> serde_json::Value {
    let mut a = serde_json::json!({
        "display-name": "x",
        "dependencies": deps,
        "code-module": "",
        "code-path": "src/lib.rs",
        "code-text": {"lines-start": 1, "lines-end": 10},
        "kind": "exec",
        "language": "rust"
    });
    if let Some(s) = status {
        a["verification-status"] = serde_json::json!(s);
    }
    a
}

fn single_tool_envelope(
    schema: &str,
    tool_name: &str,
    tool_version: &str,
    package: &str,
    data: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "schema": schema,
        "schema-version": "3.0",
        "tool": {"name": tool_name, "version": tool_version, "command": "extract"},
        "source": {"repo": "https://example.org/r", "commit": "c0ffee", "language": "rust",
                   "package": package, "package-version": "1.0"},
        "timestamp": "2026-01-01T00:00:00Z",
        "data": data
    })
}

fn mappings_file(entries: &[(&str, &str)]) -> serde_json::Value {
    let mappings: Vec<serde_json::Value> = entries
        .iter()
        .map(|(from, to)| serde_json::json!({"from": from, "to": to, "confidence": "manual"}))
        .collect();
    serde_json::json!({
        "schema": "probe/mappings",
        "schema-version": "1.0",
        "mappings": mappings
    })
}

/// Two ungated single-tool extracts, ready to merge.
fn write_inputs(dir: &Path) -> (PathBuf, PathBuf) {
    let a = dir.join("a.json");
    let b = dir.join("b.json");
    write_json(
        &a,
        &single_tool_envelope(
            "probe-rust/extract",
            "probe-rust",
            "1.0.0",
            "pkg-a",
            serde_json::json!({
                "probe:app/1.0/f()": atom(&[], Some("verified")),
            }),
        ),
    );
    write_json(
        &b,
        &single_tool_envelope(
            "probe-lean/extract",
            "probe-lean",
            "0.16.0",
            "pkg-b",
            serde_json::json!({
                "probe:Lean.T": atom(&[], Some("verified")),
            }),
        ),
    );
    (a, b)
}

/// merge → enrich / summary / project all accept the real merged output, and
/// the merged envelope carries the hub's own in-interval tool version.
#[test]
fn merged_output_passes_the_hubs_own_gate() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = write_inputs(dir.path());
    let merged = dir.path().join("merged.json");

    let out = run_probe(&[
        "merge",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "-o",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "merge failed: {}", stderr_of(&out));

    let envelope = read_json(&merged);
    assert_eq!(envelope["tool"]["name"], "probe");
    assert_eq!(
        envelope["tool"]["version"],
        env!("CARGO_PKG_VERSION"),
        "real output carries the hub's package version"
    );

    // enrich accepts the real merged output (gate: 0.5.0 ≤ v < 1.0.0).
    let enriched = dir.path().join("enriched.json");
    let out = run_probe(&[
        "enrich",
        merged.to_str().unwrap(),
        "-o",
        enriched.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "enrich rejected the hub's own merge output: {}",
        stderr_of(&out)
    );

    // summary accepts it.
    let summary = dir.path().join("summary.json");
    let out = run_probe(&[
        "summary",
        merged.to_str().unwrap(),
        "-o",
        summary.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "summary rejected the hub's own merge output: {}",
        stderr_of(&out)
    );

    // project accepts it.
    let mappings = dir.path().join("m.json");
    write_json(
        &mappings,
        &mappings_file(&[("probe:app/1.0/f()", "probe:Lean.T")]),
    );
    let projected = dir.path().join("projected.json");
    let out = run_probe(&[
        "project",
        merged.to_str().unwrap(),
        "--mappings",
        mappings.to_str().unwrap(),
        "-o",
        projected.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "project rejected the hub's own merge output: {}",
        stderr_of(&out)
    );
}

/// project output stays readable (summary, project) but is rejected by the
/// recomputation boundaries on the projection predicate specifically — not
/// the version gate.
#[test]
fn projection_readable_by_consumers_rejected_by_recomputation() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = write_inputs(dir.path());
    let merged = dir.path().join("merged.json");
    let out = run_probe(&[
        "merge",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "-o",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));

    let mappings = dir.path().join("m.json");
    write_json(
        &mappings,
        &mappings_file(&[("probe:app/1.0/f()", "probe:Lean.T")]),
    );
    let projected = dir.path().join("projected.json");
    let out = run_probe(&[
        "project",
        merged.to_str().unwrap(),
        "--mappings",
        mappings.to_str().unwrap(),
        "-o",
        projected.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));
    assert_eq!(read_json(&projected)["schema"], "probe/projected-atoms");

    // Readable: summary over the real projection.
    let out = run_probe(&[
        "summary",
        projected.to_str().unwrap(),
        "-o",
        dir.path().join("s.json").to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "summary must read projections: {}",
        stderr_of(&out)
    );

    // Readable: project over the real projection (labels inherited).
    let out = run_probe(&[
        "project",
        projected.to_str().unwrap(),
        "--mappings",
        mappings.to_str().unwrap(),
        "-o",
        dir.path().join("p2.json").to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "project must read projections: {}",
        stderr_of(&out)
    );
    assert!(
        stderr_of(&out).contains("labels inherited"),
        "projected input skips recomputation: {}",
        stderr_of(&out)
    );

    // Rejected by merge — on the projection predicate, not the gate.
    let out = run_probe(&[
        "merge",
        projected.to_str().unwrap(),
        b.to_str().unwrap(),
        "-o",
        dir.path().join("x.json").to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "merge must reject a projection");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("projected input"), "{stderr}");
    assert!(
        !stderr.contains("pre-contract"),
        "rejection must be the projection predicate, not the version gate: {stderr}"
    );

    // Rejected by enrich — same predicate.
    let out = run_probe(&["enrich", projected.to_str().unwrap()]);
    assert!(!out.status.success(), "enrich must reject a projection");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("projected input"), "{stderr}");
    assert!(!stderr.contains("pre-contract"), "{stderr}");
}

/// Envelope idempotence (P4/P9): re-merging the merged output with one of its
/// inputs changes neither the data nor the deduplicated source inventory.
#[test]
fn envelope_idempotence_modulo_meta_after_dedup() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = write_inputs(dir.path());
    let merged = dir.path().join("merged.json");
    let out = run_probe(&[
        "merge",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "-o",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));

    let merged2 = dir.path().join("merged2.json");
    let out = run_probe(&[
        "merge",
        merged.to_str().unwrap(),
        b.to_str().unwrap(),
        "-o",
        merged2.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));

    let e1 = read_json(&merged);
    let e2 = read_json(&merged2);
    assert_eq!(e1["data"], e2["data"], "data idempotent");
    assert_eq!(
        e1["inputs"], e2["inputs"],
        "provenance inventory idempotent after dedup (P9)"
    );
    assert_eq!(
        e2["inputs"].as_array().unwrap().len(),
        2,
        "two distinct sources, each once"
    );
}

/// Merging two specs files from the same source collapses the inventory to
/// one entry, and the resulting envelope still validates against the
/// executable schema (the `minItems` relaxation).
#[test]
fn specs_merge_same_source_dedups_and_validates() {
    let dir = tempfile::tempdir().unwrap();
    let specs = |data: serde_json::Value| {
        serde_json::json!({
            "schema": "probe-verus/specs",
            "schema-version": "3.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "specify"},
            "source": {"repo": "https://example.org/r", "commit": "c0ffee", "language": "rust",
                       "package": "pkg-a", "package-version": "1.0"},
            "timestamp": "2026-01-01T00:00:00Z",
            "data": data
        })
    };
    let s1 = dir.path().join("s1.json");
    let s2 = dir.path().join("s2.json");
    write_json(
        &s1,
        &specs(serde_json::json!({"probe:a/1.0/f()": {"specified": true}})),
    );
    write_json(
        &s2,
        &specs(serde_json::json!({"probe:a/1.0/g()": {"specified": true}})),
    );

    let merged = dir.path().join("merged_specs.json");
    let out = run_probe(&[
        "merge",
        s1.to_str().unwrap(),
        s2.to_str().unwrap(),
        "-o",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));

    let envelope = read_json(&merged);
    assert_eq!(
        envelope["inputs"].as_array().unwrap().len(),
        1,
        "identical sources collapse to one inventory entry (P9)"
    );

    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("schemas/atom-envelope.schema.json").unwrap(),
    )
    .unwrap();
    let validator = jsonschema::Validator::new(&schema).expect("valid schema");
    assert!(
        validator.validate(&envelope).is_ok(),
        "deduped single-entry inventory must validate (minItems 1)"
    );
}

/// Plan issue 6: an intra-input post-normalization collision in a specs file
/// is warned about and counted in the reported conflicts, while last-wins is
/// kept.
#[test]
fn specs_intra_input_collision_warned_and_counted() {
    let dir = tempfile::tempdir().unwrap();
    let s1 = dir.path().join("s1.json");
    let s2 = dir.path().join("s2.json");
    write_json(
        &s1,
        &serde_json::json!({
            "schema": "probe-verus/specs",
            "schema-version": "3.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "specify"},
            "source": {"repo": "https://example.org/r", "commit": "c0ffee", "language": "rust",
                       "package": "pkg-a", "package-version": "1.0"},
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {
                "probe:a/1.0/s()": {"which": "plain"},
                "probe:a/1.0/s().": {"which": "dotted"}
            }
        }),
    );
    write_json(
        &s2,
        &serde_json::json!({
            "schema": "probe-lean/specs",
            "schema-version": "3.0",
            "tool": {"name": "probe-lean", "version": "0.16.0", "command": "specify"},
            "source": {"repo": "https://example.org/r2", "commit": "c1", "language": "lean",
                       "package": "pkg-b", "package-version": "1.0"},
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {"probe:Lean.spec": {"which": "lean"}}
        }),
    );

    let merged = dir.path().join("merged_specs.json");
    let out = run_probe(&[
        "merge",
        s1.to_str().unwrap(),
        s2.to_str().unwrap(),
        "-o",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));
    assert!(
        stderr_of(&out).contains("normalization collision"),
        "collision warned: {}",
        stderr_of(&out)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains("Conflicts:        1"),
        "collision counted in stats.conflicts: {stdout}"
    );
    assert_eq!(
        read_json(&merged)["data"]["probe:a/1.0/s()"]["which"],
        "dotted",
        "last-wins kept (P7)"
    );
}

/// The Verus-shaped stale-label regression (plan §3d/§8): projecting an
/// authoritative extract carrying a stale embedded enrichment recomputes
/// labels on the full graph *before* trimming — f comes out `verified`, not
/// the frozen `transitively-verified`.
#[test]
fn projection_recomputes_enrichment_before_trimming() {
    let dir = tempfile::tempdir().unwrap();
    let extract = dir.path().join("verus_shaped.json");
    write_json(
        &extract,
        &single_tool_envelope(
            "probe-verus/atoms",
            "probe-verus",
            "2.0.0",
            "pkg-v",
            serde_json::json!({
                "probe:app/1.0/f()": atom(&["probe:app/1.0/helper()"], Some("transitively-verified")),
                "probe:app/1.0/helper()": atom(&["probe:app/1.0/bad()"], None),
                "probe:app/1.0/bad()": atom(&[], Some("failed")),
            }),
        ),
    );
    let mappings = dir.path().join("m.json");
    write_json(
        &mappings,
        &mappings_file(&[("probe:app/1.0/f()", "probe:Lean.f")]),
    );

    let projected = dir.path().join("projected.json");
    let out = run_probe(&[
        "project",
        extract.to_str().unwrap(),
        "--mappings",
        mappings.to_str().unwrap(),
        "--forward-depth",
        "0",
        "-o",
        projected.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));

    let f = &read_json(&projected)["data"]["probe:app/1.0/f()"];
    assert_eq!(
        f["verification-status"], "verified",
        "stale transitively-verified recomputed on the full graph before trimming (P23)"
    );
    assert_eq!(
        f["dependencies"],
        serde_json::json!([]),
        "depth-0 view has the dependency trimmed"
    );
}

/// The dotted-key projection-seed regression (plan §3d): a legacy dotted-key
/// atom is found by its normalized mapping seed — seed matching runs over
/// normalized keys on both sides (P8).
#[test]
fn projection_seeds_match_normalized_keys() {
    let dir = tempfile::tempdir().unwrap();
    let extract = dir.path().join("legacy.json");
    write_json(
        &extract,
        &single_tool_envelope(
            "probe-verus/atoms",
            "probe-verus",
            "2.0.0",
            "pkg-v",
            serde_json::json!({
                "probe:app/1.0/f().": atom(&[], Some("verified")),
            }),
        ),
    );
    let mappings = dir.path().join("m.json");
    write_json(
        &mappings,
        &mappings_file(&[("probe:app/1.0/f().", "probe:Lean.f")]),
    );

    let projected = dir.path().join("projected.json");
    let out = run_probe(&[
        "project",
        extract.to_str().unwrap(),
        "--mappings",
        mappings.to_str().unwrap(),
        "-o",
        projected.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr_of(&out));

    let data = &read_json(&projected)["data"];
    assert!(
        data.get("probe:app/1.0/f()").is_some(),
        "dotted-key atom selected by its normalized seed (P8): {data}"
    );
}
