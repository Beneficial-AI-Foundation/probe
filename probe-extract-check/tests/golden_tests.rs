//! Golden file tests: validate that expected.json files are self-consistent
//! with their source projects (Layer 2), plus property checks (Layer 3)
//! and live tool comparison tests.
//!
//! These tests verify:
//! 1. The golden JSON passes all structural checks
//! 2. The golden JSON passes source-grounded checks against the micro-project
//! 3. The golden JSON passes dependency checks against the micro-project
//! 4. Property checks (completeness, overlap) pass
//! 5. (ignored) Live extract tools produce output matching golden files

use probe_extract_check::structural::Level;
use std::path::Path;

const FIXTURES: &str = "tests/fixtures";

/// Load a golden file and run all checks against the source project.
fn check_golden(fixture_name: &str, project_subdir: Option<&str>) {
    let fixture_dir = Path::new(FIXTURES).join(fixture_name);
    let json_path = fixture_dir.join("expected.json");
    let project_path = match project_subdir {
        Some(sub) => fixture_dir.join(sub),
        None => fixture_dir.clone(),
    };

    let envelope = probe_extract_check::load_extract_json(&json_path)
        .unwrap_or_else(|e| panic!("failed to load {}: {e}", json_path.display()));

    let report = probe_extract_check::check_all(&envelope, Some(&project_path));

    if !report.diagnostics.is_empty() {
        eprintln!("\n=== Diagnostics for {fixture_name} ===");
        report.print_summary();
    }

    let errors: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.level == Level::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "{fixture_name}: expected no errors, got {} error(s):\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!("  {d}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// =========================================================================
// rust_micro
// =========================================================================

#[test]
fn golden_rust_micro_structural() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let diags = probe_extract_check::structural::check_structural(&envelope);
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "structural errors: {errors:?}");
}

#[test]
fn golden_rust_micro_source_and_deps() {
    check_golden("rust_micro", None);
}

#[test]
fn golden_rust_micro_atom_count() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    assert_eq!(
        envelope.data.len(),
        15,
        "rust_micro should have 15 atoms: greet, capitalize, add, double, is_even, is_odd, \
         standalone, Circle::area, Rect::area, total_area, apply_transform, scale_areas, \
         to_u64, to_i64, convert_both (types_only contributes 0)"
    );
}

#[test]
fn golden_rust_micro_mutual_recursion() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    let is_even = &envelope.data["probe:rust-micro/0.1.0/math/is_even()"];
    assert!(
        is_even
            .dependencies
            .contains("probe:rust-micro/0.1.0/math/is_odd()"),
        "is_even should depend on is_odd"
    );

    let is_odd = &envelope.data["probe:rust-micro/0.1.0/math/is_odd()"];
    assert!(
        is_odd
            .dependencies
            .contains("probe:rust-micro/0.1.0/math/is_even()"),
        "is_odd should depend on is_even"
    );
}

#[test]
fn golden_rust_micro_no_deps_atom() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let standalone = &envelope.data["probe:rust-micro/0.1.0/math/standalone()"];
    assert!(
        standalone.dependencies.is_empty(),
        "standalone should have no dependencies"
    );
}

#[test]
fn golden_rust_micro_trait_impls() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    // Two separate trait impl atoms for area() with different code-names
    let circle_area =
        &envelope.data["probe:rust-micro/0.1.0/shapes/&Circle#impl#[Circle][Area]area()"];
    assert_eq!(circle_area.display_name, "area");
    assert_eq!(circle_area.code_path, "src/shapes.rs");

    let rect_area = &envelope.data["probe:rust-micro/0.1.0/shapes/&Rect#impl#[Rect][Area]area()"];
    assert_eq!(rect_area.display_name, "area");
    assert_eq!(rect_area.code_path, "src/shapes.rs");

    // Different line ranges despite same display-name
    assert_ne!(
        circle_area.code_text.lines_start, rect_area.code_text.lines_start,
        "trait impls should have different line ranges"
    );
}

#[test]
fn golden_rust_micro_generics() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    let total_area = &envelope.data["probe:rust-micro/0.1.0/shapes/total_area()"];
    assert_eq!(total_area.display_name, "total_area");

    let scale_areas = &envelope.data["probe:rust-micro/0.1.0/shapes/scale_areas()"];
    assert!(
        scale_areas
            .dependencies
            .contains("probe:rust-micro/0.1.0/shapes/apply_transform()"),
        "scale_areas should depend on apply_transform"
    );
}

#[test]
fn golden_rust_micro_closure_param() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    // apply_transform takes an impl Fn — it should exist as an atom
    let apply = &envelope.data["probe:rust-micro/0.1.0/shapes/apply_transform()"];
    assert_eq!(apply.display_name, "apply_transform");
    assert!(apply.dependencies.is_empty());
}

#[test]
fn golden_rust_micro_types_only_no_atoms() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    // types_only.rs has only structs, enums, type aliases, and consts — no functions.
    // Verify no atoms reference it.
    let types_only_atoms: Vec<_> = envelope
        .data
        .values()
        .filter(|a| a.code_path == "src/types_only.rs")
        .collect();
    assert!(
        types_only_atoms.is_empty(),
        "types_only.rs should contribute 0 atoms, found {}",
        types_only_atoms.len()
    );
}

#[test]
fn golden_rust_micro_macro_generated() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    // Macro-generated functions should be present as atoms
    assert!(
        envelope
            .data
            .contains_key("probe:rust-micro/0.1.0/macros/to_u64()"),
        "macro-generated to_u64 should be an atom"
    );
    assert!(
        envelope
            .data
            .contains_key("probe:rust-micro/0.1.0/macros/to_i64()"),
        "macro-generated to_i64 should be an atom"
    );

    // convert_both should depend on both macro-generated functions
    let convert = &envelope.data["probe:rust-micro/0.1.0/macros/convert_both()"];
    assert!(
        convert
            .dependencies
            .contains("probe:rust-micro/0.1.0/macros/to_u64()"),
        "convert_both should depend on to_u64"
    );
    assert!(
        convert
            .dependencies
            .contains("probe:rust-micro/0.1.0/macros/to_i64()"),
        "convert_both should depend on to_i64"
    );
}

// =========================================================================
// verus_micro
// =========================================================================

#[test]
fn golden_verus_micro_structural() {
    let json_path = Path::new(FIXTURES).join("verus_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let diags = probe_extract_check::structural::check_structural(&envelope);
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "structural errors: {errors:?}");
}

#[test]
fn golden_verus_micro_source_and_deps() {
    check_golden("verus_micro", None);
}

#[test]
fn golden_verus_micro_kinds() {
    let json_path = Path::new(FIXTURES).join("verus_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    assert_eq!(
        envelope.data["probe:verus-micro/0.1.0/lib/is_positive()"].kind,
        "spec"
    );
    assert_eq!(
        envelope.data["probe:verus-micro/0.1.0/lib/positive_sum()"].kind,
        "proof"
    );
    assert_eq!(
        envelope.data["probe:verus-micro/0.1.0/lib/checked_add()"].kind,
        "exec"
    );
    assert_eq!(
        envelope.data["probe:verus-micro/0.1.0/lib/double_checked()"].kind,
        "exec"
    );
}

#[test]
fn golden_verus_micro_categorized_deps() {
    let json_path = Path::new(FIXTURES).join("verus_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    // double_checked should have body dep on checked_add and requires dep on is_positive
    let double = &envelope.data["probe:verus-micro/0.1.0/lib/double_checked()"];
    let body_deps = double
        .extensions
        .get("body-dependencies")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    assert!(
        body_deps.contains(&"probe:verus-micro/0.1.0/lib/checked_add()"),
        "double_checked should have body dep on checked_add"
    );
}

// =========================================================================
// lean_micro
// =========================================================================

#[test]
fn golden_lean_micro_structural() {
    let json_path = Path::new(FIXTURES).join("lean_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let diags = probe_extract_check::structural::check_structural(&envelope);
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "structural errors: {errors:?}");
}

#[test]
fn golden_lean_micro_source_and_deps() {
    check_golden("lean_micro", None);
}

#[test]
fn golden_lean_micro_kinds() {
    let json_path = Path::new(FIXTURES).join("lean_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    assert_eq!(
        envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.add"].kind,
        "def"
    );
    assert_eq!(
        envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.add_comm"].kind,
        "theorem"
    );
    assert_eq!(
        envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.Point"].kind,
        "structure"
    );
    assert_eq!(
        envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.HasSize"].kind,
        "class"
    );
    assert_eq!(
        envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.instHasSizePoint"].kind,
        "instance"
    );
}

#[test]
fn golden_lean_micro_theorem_deps() {
    let json_path = Path::new(FIXTURES).join("lean_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    let thm = &envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.double_eq_add_self"];
    assert!(
        thm.dependencies
            .contains("probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.double"),
        "double_eq_add_self should depend on double"
    );
    assert!(
        thm.dependencies
            .contains("probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.add"),
        "double_eq_add_self should depend on add"
    );
}

#[test]
fn golden_lean_micro_instance() {
    let json_path = Path::new(FIXTURES).join("lean_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    let inst = &envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.instHasSizePoint"];
    assert_eq!(inst.kind, "instance");
    assert!(
        inst.dependencies
            .contains("probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.add"),
        "HasSize Point instance should depend on add"
    );
}

#[test]
fn golden_lean_micro_sorry() {
    let json_path = Path::new(FIXTURES).join("lean_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    let sorry_thm = &envelope.data["probe:LeanMicro/0.1.0/LeanMicro.Basic/LeanMicro.sorry_example"];
    assert_eq!(sorry_thm.kind, "theorem");
    let status = sorry_thm
        .extensions
        .get("verification-status")
        .and_then(|v| v.as_str());
    assert_eq!(
        status,
        Some("failed"),
        "sorry_example should have verification-status 'failed'"
    );
}

// =========================================================================
// aeneas_micro
// =========================================================================

#[test]
fn golden_aeneas_micro_structural() {
    let json_path = Path::new(FIXTURES).join("aeneas_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let diags = probe_extract_check::structural::check_structural(&envelope);
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "structural errors: {errors:?}");
}

#[test]
fn golden_aeneas_micro_source_and_deps() {
    check_golden("aeneas_micro", Some("rust_src"));
}

#[test]
fn golden_aeneas_micro_translations() {
    let json_path = Path::new(FIXTURES).join("aeneas_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();

    for atom in envelope.data.values() {
        let translation = atom
            .extensions
            .get("translation-name")
            .and_then(|v| v.as_str());
        assert!(
            translation.is_some(),
            "all aeneas_micro atoms should have translation-name, missing for {}",
            atom.display_name
        );
    }
}

// =========================================================================
// Layer 3: Property checks on golden files
// =========================================================================

#[test]
fn properties_rust_micro() {
    let json_path = Path::new(FIXTURES).join("rust_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let project_path = Path::new(FIXTURES).join("rust_micro");
    let diags =
        probe_extract_check::properties::check_properties(&envelope.data, Some(&project_path));
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "property errors: {errors:?}");
    // Check no overlap warnings either
    let overlaps: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("overlapping"))
        .collect();
    assert!(overlaps.is_empty(), "unexpected overlaps: {overlaps:?}");
}

#[test]
fn properties_lean_micro() {
    let json_path = Path::new(FIXTURES).join("lean_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let project_path = Path::new(FIXTURES).join("lean_micro");
    let diags =
        probe_extract_check::properties::check_properties(&envelope.data, Some(&project_path));
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "property errors: {errors:?}");
}

#[test]
fn properties_verus_micro() {
    let json_path = Path::new(FIXTURES).join("verus_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let project_path = Path::new(FIXTURES).join("verus_micro");
    let diags =
        probe_extract_check::properties::check_properties(&envelope.data, Some(&project_path));
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "property errors: {errors:?}");
}

#[test]
fn properties_aeneas_micro() {
    let json_path = Path::new(FIXTURES).join("aeneas_micro/expected.json");
    let envelope = probe_extract_check::load_extract_json(&json_path).unwrap();
    let project_path = Path::new(FIXTURES).join("aeneas_micro/rust_src");
    let diags =
        probe_extract_check::properties::check_properties(&envelope.data, Some(&project_path));
    let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
    assert!(errors.is_empty(), "property errors: {errors:?}");
}

// =========================================================================
// Live tool comparison (requires tools installed — run with --include-ignored)
// =========================================================================

/// Run an external tool and compare output to golden file using structural diff.
fn run_tool_and_compare(tool_binary: &str, args: &[&str], output_path: &Path, golden_path: &Path) {
    use std::process::Command;

    let status = Command::new(tool_binary)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {tool_binary}: {e}"));
    assert!(status.success(), "{tool_binary} exited with {status}");

    let actual_content = std::fs::read_to_string(output_path)
        .unwrap_or_else(|e| panic!("failed to read output {}: {e}", output_path.display()));
    let actual: serde_json::Value = serde_json::from_str(&actual_content)
        .unwrap_or_else(|e| panic!("failed to parse output JSON: {e}"));

    let golden_content = std::fs::read_to_string(golden_path)
        .unwrap_or_else(|e| panic!("failed to read golden {}: {e}", golden_path.display()));
    let expected: serde_json::Value = serde_json::from_str(&golden_content)
        .unwrap_or_else(|e| panic!("failed to parse golden JSON: {e}"));

    let diffs = probe_extract_check::golden::compare(&expected, &actual);
    if !diffs.is_empty() {
        eprintln!("\n=== Golden diff ({tool_binary}) ===");
        for d in &diffs {
            eprintln!("  {d}");
        }
        panic!(
            "{tool_binary}: golden comparison found {} difference(s)",
            diffs.len()
        );
    }
}

#[test]
#[ignore = "requires probe-rust installed"]
fn live_probe_rust_extract() {
    let tmp = tempfile::TempDir::new().unwrap();
    let output = tmp.path().join("output.json");
    let project = Path::new(FIXTURES).join("rust_micro");
    let golden = project.join("expected.json");

    run_tool_and_compare(
        "probe-rust",
        &[
            "extract",
            project.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--auto-install",
        ],
        &output,
        &golden,
    );
}

#[test]
#[ignore = "requires probe-verus installed"]
fn live_probe_verus_extract() {
    let tmp = tempfile::TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    let project = Path::new(FIXTURES).join("verus_micro");
    let golden = project.join("expected.json");

    // probe-verus extract writes to a directory; the unified output is extract.json
    run_tool_and_compare(
        "probe-verus",
        &[
            "extract",
            project.to_str().unwrap(),
            "-o",
            output_dir.to_str().unwrap(),
            "--auto-install",
        ],
        &output_dir.join("extract.json"),
        &golden,
    );
}

#[test]
#[ignore = "requires probe-lean installed"]
fn live_probe_lean_extract() {
    let tmp = tempfile::TempDir::new().unwrap();
    let output = tmp.path().join("output.json");
    let project = Path::new(FIXTURES).join("lean_micro");
    let golden = project.join("expected.json");

    run_tool_and_compare(
        "probe-lean",
        &[
            "extract",
            project.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ],
        &output,
        &golden,
    );
}

#[test]
#[ignore = "requires probe-aeneas installed"]
fn live_probe_aeneas_extract() {
    let tmp = tempfile::TempDir::new().unwrap();
    let output = tmp.path().join("output.json");
    let project = Path::new(FIXTURES).join("aeneas_micro");
    let golden = project.join("expected.json");

    run_tool_and_compare(
        "probe-aeneas",
        &[
            "extract",
            "--rust-project",
            project.join("rust_src").to_str().unwrap(),
            "--lean-project",
            project.join("lean_src").to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ],
        &output,
        &golden,
    );
}

// =========================================================================
// Idempotency tests (run extract twice, compare outputs)
// =========================================================================

/// Run an extract tool twice and verify the outputs are structurally identical.
fn run_tool_idempotency(tool_binary: &str, args_fn: impl Fn(&Path) -> Vec<String>) {
    use std::process::Command;

    let tmp1 = tempfile::TempDir::new().unwrap();
    let tmp2 = tempfile::TempDir::new().unwrap();
    let out1 = tmp1.path().join("output.json");
    let out2 = tmp2.path().join("output.json");

    let args1 = args_fn(&out1);
    let args1_refs: Vec<&str> = args1.iter().map(|s| s.as_str()).collect();
    let status1 = Command::new(tool_binary)
        .args(&args1_refs)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {tool_binary} (run 1): {e}"));
    assert!(status1.success(), "{tool_binary} run 1 failed: {status1}");

    let args2 = args_fn(&out2);
    let args2_refs: Vec<&str> = args2.iter().map(|s| s.as_str()).collect();
    let status2 = Command::new(tool_binary)
        .args(&args2_refs)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {tool_binary} (run 2): {e}"));
    assert!(status2.success(), "{tool_binary} run 2 failed: {status2}");

    let content1 = std::fs::read_to_string(&out1).unwrap();
    let content2 = std::fs::read_to_string(&out2).unwrap();
    let val1: serde_json::Value = serde_json::from_str(&content1).unwrap();
    let val2: serde_json::Value = serde_json::from_str(&content2).unwrap();

    let diffs = probe_extract_check::golden::compare(&val1, &val2);
    if !diffs.is_empty() {
        eprintln!("\n=== Idempotency diff ({tool_binary}) ===");
        for d in &diffs {
            eprintln!("  {d}");
        }
        panic!(
            "{tool_binary}: idempotency check found {} difference(s) between two runs",
            diffs.len()
        );
    }
}

#[test]
#[ignore = "requires probe-rust installed"]
fn idempotency_probe_rust() {
    let project = std::fs::canonicalize(Path::new(FIXTURES).join("rust_micro")).unwrap();
    run_tool_idempotency("probe-rust", |out| {
        vec![
            "extract".into(),
            project.to_str().unwrap().into(),
            "--output".into(),
            out.to_str().unwrap().into(),
            "--auto-install".into(),
        ]
    });
}

#[test]
#[ignore = "requires probe-verus installed"]
fn idempotency_probe_verus() {
    let project = std::fs::canonicalize(Path::new(FIXTURES).join("verus_micro")).unwrap();
    run_tool_idempotency("probe-verus", |out| {
        let out_dir = out.parent().unwrap();
        vec![
            "extract".into(),
            project.to_str().unwrap().into(),
            "-o".into(),
            out_dir.to_str().unwrap().into(),
            "--auto-install".into(),
        ]
    });
}

#[test]
#[ignore = "requires probe-lean installed"]
fn idempotency_probe_lean() {
    let project = std::fs::canonicalize(Path::new(FIXTURES).join("lean_micro")).unwrap();
    run_tool_idempotency("probe-lean", |out| {
        vec![
            "extract".into(),
            project.to_str().unwrap().into(),
            "--output".into(),
            out.to_str().unwrap().into(),
        ]
    });
}

#[test]
#[ignore = "requires probe-aeneas installed"]
fn idempotency_probe_aeneas() {
    let project = std::fs::canonicalize(Path::new(FIXTURES).join("aeneas_micro")).unwrap();
    run_tool_idempotency("probe-aeneas", |out| {
        vec![
            "extract".into(),
            "--rust-project".into(),
            project.join("rust_src").to_str().unwrap().into(),
            "--lean-project".into(),
            project.join("lean_src").to_str().unwrap().into(),
            "--output".into(),
            out.to_str().unwrap().into(),
        ]
    });
}
