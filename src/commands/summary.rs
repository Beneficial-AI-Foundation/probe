// @kb: kb/engineering/properties.md#p1-envelope-completeness
// @kb: kb/engineering/properties.md#p3-stub-detection-is-structural
// @kb: kb/engineering/properties.md#p14-deterministic-output
// @kb: kb/engineering/schema.md#language-assignment-for-verus-atoms
// @kb: kb/tools/probe-summary.md#imported-evidence-imported-verified
// @kb: kb/tools/probe-summary.md#scope-code-atoms-only

use crate::authority::{load_validated_atom_file, AuthorityScope};
use crate::types::{Atom, InputProvenance, Tool};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Schema 3.0 envelope for summary output.
#[derive(serde::Serialize)]
struct SummaryEnvelope {
    schema: &'static str,
    #[serde(rename = "schema-version")]
    schema_version: &'static str,
    tool: Tool,
    inputs: Vec<InputProvenance>,
    timestamp: String,
    data: SummaryResult,
}

/// Payload of a summary result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SummaryResult {
    pub verified_entrypoints: Vec<String>,
    pub verified_functions: Vec<String>,
    pub verified_lemmas: Vec<String>,
    pub imported_verified: Vec<String>,
}

fn is_test(atom: &Atom) -> bool {
    atom.code_module.contains("test") || atom.display_name.contains("test")
}

fn is_verified(atom: &Atom) -> bool {
    atom.extensions
        .get("verification-status")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "verified" || s == "transitively-verified")
}

/// Imported evidence (ADR-006): the status was copied from a corresponding
/// atom in another language. `status-origin: "kernel-taint"` is local
/// evidence and stays in the ordinary partitions.
fn is_translation_origin(atom: &Atom) -> bool {
    atom.extensions
        .get("status-origin")
        .and_then(|v| v.as_str())
        == Some("translation")
}

/// Presentation aggregates (probe-leanblueprint synthetic node atoms) sit
/// outside the code-atom assurance contract: they appear in no partition and
/// their binding edges must not alter code entrypoints (ADR-006).
fn is_blueprint(atom: &Atom) -> bool {
    atom.language == "blueprint"
}

// @kb: kb/engineering/schema.md#language-assignment-for-verus-atoms
fn is_rust_exec(atom: &Atom) -> bool {
    atom.language == "rust" && atom.kind == "exec"
}

/// Partition verified atoms into entrypoints, verified functions, lemmas,
/// and imported evidence (ADR-006 Decision 9).
///
/// All partitions are computed over **code atoms**: `language: "blueprint"`
/// atoms appear in no list and are excluded from the `depended_upon` set, so
/// presentation-layer binding edges cannot alter code entrypoints.
///
/// **Imported verified**: atoms with `status-origin: "translation"` and a
/// verified status. Imported evidence never appears in the three local
/// partitions; a translated non-verified status appears in no list at all.
///
/// **Entrypoints**: locally verified, non-stub, non-test Rust `exec` atoms
/// whose code-name never appears in any non-test code atom's `dependencies`
/// array.
///
/// **Verified functions**: remaining locally verified Rust `exec` atoms
/// (depended-upon helpers, stubs, test functions).
///
/// **Verified lemmas**: locally verified non-(Rust `exec`) code atoms.
///
/// The four lists partition all verified code atoms.
///
/// Errors when any atom carries a `status-origin` outside the two-value enum
/// (ADR-006 Decision 2): an unknown origin proves neither local nor imported
/// evidence, so this boundary is checked here rather than trusting callers to
/// validate — the CLI adds its input path to the error.
pub fn summarize_atoms(atoms: &BTreeMap<String, Atom>) -> Result<SummaryResult, String> {
    crate::types::validate_status_origins(atoms)?;

    let depended_upon: BTreeSet<&str> = atoms
        .values()
        .filter(|atom| !is_test(atom) && !is_blueprint(atom))
        .flat_map(|atom| atom.dependencies.iter())
        .map(String::as_str)
        .collect();

    let mut verified_entrypoints: Vec<String> = Vec::new();
    let mut verified_functions: Vec<String> = Vec::new();
    let mut verified_lemmas: Vec<String> = Vec::new();
    let mut imported_verified: Vec<String> = Vec::new();

    for (code_name, atom) in atoms {
        if is_blueprint(atom) || !is_verified(atom) {
            continue;
        }
        if is_translation_origin(atom) {
            imported_verified.push(code_name.clone());
            continue;
        }
        let is_entrypoint = !atom.is_stub()
            && !is_test(atom)
            && is_rust_exec(atom)
            && !depended_upon.contains(code_name.as_str());

        if is_entrypoint {
            verified_entrypoints.push(code_name.clone());
        } else if is_rust_exec(atom) {
            verified_functions.push(code_name.clone());
        } else {
            verified_lemmas.push(code_name.clone());
        }
    }

    Ok(SummaryResult {
        verified_entrypoints,
        verified_functions,
        verified_lemmas,
        imported_verified,
    })
}

/// Derive a default output filename from provenance: `summary_<package>_<version>.json`.
fn default_output_name(provenance: &[InputProvenance]) -> String {
    if let Some(first) = provenance.first() {
        let pkg = &first.source.package;
        let ver = &first.source.package_version;
        if !pkg.is_empty() && !ver.is_empty() {
            return format!("summary_{pkg}_{ver}.json");
        }
    }
    "summary.json".to_string()
}

/// CLI entry point: load atom file, compute summary, emit envelope.
///
/// The input passes the version-gate component of the authority validator
/// (ADR-006 Decision 7): a pre-contract `verified` is indistinguishable from
/// local evidence, so summary must not describe it. Projections stay
/// readable — views with inherited labels are legitimate to read.
// @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
pub fn cmd_summary(input: &Path, output: Option<&Path>) {
    let (atoms, provenance) = match load_validated_atom_file(input, AuthorityScope::ReadOnly) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // summarize_atoms fails closed on out-of-enum status-origin markers
    // (ADR-006 Decision 2): an unknown origin proves neither local nor
    // imported evidence, so it must not be classified into the local
    // partitions.
    let result = summarize_atoms(&atoms).unwrap_or_else(|e| {
        eprintln!("Error: {}: {e}", input.display());
        std::process::exit(1);
    });

    let total = result.verified_entrypoints.len()
        + result.verified_functions.len()
        + result.verified_lemmas.len()
        + result.imported_verified.len();
    eprintln!(
        "Verified: {total}  |  Entrypoints: {}  |  Functions: {}  |  Lemmas: {}  |  Imported: {}",
        result.verified_entrypoints.len(),
        result.verified_functions.len(),
        result.verified_lemmas.len(),
        result.imported_verified.len()
    );

    let envelope = SummaryEnvelope {
        schema: "probe/summary",
        schema_version: "3.0",
        tool: Tool {
            name: "probe".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: "summary".to_string(),
        },
        inputs: provenance.clone(),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        data: result,
    };

    let json = serde_json::to_string_pretty(&envelope).expect("failed to serialize output");

    let default_name;
    let out_path = match output {
        Some(p) => p,
        None => {
            default_name = default_output_name(&provenance);
            Path::new(&default_name)
        }
    };

    std::fs::write(out_path, &json).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {e}", out_path.display());
        std::process::exit(1);
    });
    eprintln!("Wrote {}", out_path.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Atom, CodeText};
    use std::collections::BTreeMap;

    fn make_atom(language: &str, kind: &str, code_path: &str, display_name: &str) -> Atom {
        Atom {
            display_name: display_name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: code_path.to_string(),
            code_text: CodeText {
                lines_start: if code_path.is_empty() { 0 } else { 1 },
                lines_end: if code_path.is_empty() { 0 } else { 10 },
            },
            kind: kind.to_string(),
            language: language.to_string(),
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

    fn set_origin(atom: &mut Atom, origin: &str) {
        atom.extensions.insert(
            "status-origin".to_string(),
            serde_json::Value::String(origin.to_string()),
        );
    }

    fn add_dep(atom: &mut Atom, dep: &str) {
        atom.dependencies.insert(dep.to_string());
    }

    fn all_lists(result: &SummaryResult) -> Vec<&str> {
        result
            .verified_entrypoints
            .iter()
            .chain(&result.verified_functions)
            .chain(&result.verified_lemmas)
            .chain(&result.imported_verified)
            .map(String::as_str)
            .collect()
    }

    // The public library boundary is itself fail-closed (ADR-006 Decision 2):
    // a caller that skips CLI validation still cannot get an invalid marker
    // classified into the local partitions.
    #[test]
    fn test_summarize_atoms_rejects_invalid_status_origin() {
        for bad in [serde_json::json!("future-import"), serde_json::json!(null)] {
            let mut atoms = BTreeMap::new();
            let mut atom = make_atom("rust", "exec", "src/lib.rs", "f");
            set_verified(&mut atom);
            atom.extensions.insert("status-origin".to_string(), bad);
            atoms.insert("probe:pkg/1.0/f()".to_string(), atom);

            let err = summarize_atoms(&atoms).unwrap_err();
            assert!(err.contains("invalid status-origin"), "{err}");
        }
    }

    // Wire contract (kb/tools/probe-summary.md § Output format): the four
    // lists serialize under exactly these snake_case field names.
    #[test]
    fn test_summary_result_wire_field_names() {
        let result = SummaryResult {
            verified_entrypoints: vec![],
            verified_functions: vec![],
            verified_lemmas: vec![],
            imported_verified: vec![],
        };
        let value = serde_json::to_value(&result).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            vec![
                "imported_verified",
                "verified_entrypoints",
                "verified_functions",
                "verified_lemmas"
            ]
        );
    }

    #[test]
    fn test_partition_is_exact() {
        let mut atoms = BTreeMap::new();

        let mut ep = make_atom("rust", "exec", "src/lib.rs", "compress");
        set_verified(&mut ep);
        atoms.insert("probe:pkg/1.0/compress()".to_string(), ep);

        let mut dep = make_atom("rust", "exec", "src/field.rs", "reduce");
        set_verified(&mut dep);
        add_dep(&mut dep, "probe:pkg/1.0/helper()");
        atoms.insert("probe:pkg/1.0/reduce()".to_string(), dep);

        let mut caller = make_atom("rust", "exec", "src/lib.rs", "caller");
        set_verified(&mut caller);
        add_dep(&mut caller, "probe:pkg/1.0/reduce()");
        atoms.insert("probe:pkg/1.0/caller()".to_string(), caller);

        let result = summarize_atoms(&atoms).unwrap();
        assert_eq!(
            result.verified_entrypoints.len()
                + result.verified_functions.len()
                + result.verified_lemmas.len(),
            3
        );
    }

    #[test]
    fn test_stubs_are_not_entrypoints() {
        let mut atoms = BTreeMap::new();

        let mut stub = make_atom("rust", "exec", "", "alloc_fn");
        set_verified(&mut stub);
        atoms.insert("probe:alloc/1.0/alloc_fn()".to_string(), stub);

        let result = summarize_atoms(&atoms).unwrap();
        assert!(result.verified_entrypoints.is_empty());
        assert_eq!(result.verified_functions.len(), 1);
        assert!(result.verified_lemmas.is_empty());
    }

    #[test]
    fn test_tests_excluded_from_entrypoints() {
        let mut atoms = BTreeMap::new();

        let mut test_atom = make_atom("rust", "exec", "src/tests.rs", "test_foo");
        test_atom.code_module = "test_module".to_string();
        set_verified(&mut test_atom);
        atoms.insert(
            "probe:pkg/1.0/test_module/test_foo()".to_string(),
            test_atom,
        );

        let result = summarize_atoms(&atoms).unwrap();
        assert!(result.verified_entrypoints.is_empty());
        assert_eq!(result.verified_functions.len(), 1);
        assert!(result.verified_lemmas.is_empty());
    }

    #[test]
    fn test_verus_spec_proof_not_entrypoints() {
        let mut atoms = BTreeMap::new();

        let mut spec = make_atom("verus", "spec", "src/specs.rs", "my_spec");
        set_verified(&mut spec);
        atoms.insert("probe:pkg/1.0/specs/my_spec()".to_string(), spec);

        let mut proof = make_atom("verus", "proof", "src/lemmas.rs", "my_lemma");
        set_verified(&mut proof);
        atoms.insert("probe:pkg/1.0/lemmas/my_lemma()".to_string(), proof);

        let result = summarize_atoms(&atoms).unwrap();
        assert!(result.verified_entrypoints.is_empty());
        assert!(result.verified_functions.is_empty());
        assert_eq!(result.verified_lemmas.len(), 2);
    }

    #[test]
    fn test_unverified_atoms_excluded_from_both_lists() {
        let mut atoms = BTreeMap::new();

        let unverified = make_atom("rust", "exec", "src/lib.rs", "foo");
        atoms.insert("probe:pkg/1.0/foo()".to_string(), unverified);

        let result = summarize_atoms(&atoms).unwrap();
        assert!(result.verified_entrypoints.is_empty());
        assert!(result.verified_functions.is_empty());
        assert!(result.verified_lemmas.is_empty());
    }

    #[test]
    fn test_test_deps_dont_disqualify_entrypoints() {
        let mut atoms = BTreeMap::new();

        let mut func = make_atom("rust", "exec", "src/lib.rs", "compress");
        set_verified(&mut func);
        atoms.insert("probe:pkg/1.0/compress()".to_string(), func);

        let mut test_fn = make_atom("rust", "exec", "src/tests.rs", "test_compress");
        test_fn.code_module = "tests".to_string();
        set_verified(&mut test_fn);
        add_dep(&mut test_fn, "probe:pkg/1.0/compress()");
        atoms.insert("probe:pkg/1.0/tests/test_compress()".to_string(), test_fn);

        let result = summarize_atoms(&atoms).unwrap();
        assert_eq!(
            result.verified_entrypoints,
            vec!["probe:pkg/1.0/compress()"]
        );
    }

    // Status-matrix test (ADR-006 Decision 9): both status-origin values ×
    // the five statuses. Translation × verified statuses → imported_verified
    // only; translation × non-verified → no list; kernel-taint stays in the
    // ordinary partitions; kernel-taint × non-verified → no list.
    #[test]
    fn test_status_origin_matrix() {
        let statuses = [
            "verified",
            "transitively-verified",
            "unverified",
            "failed",
            "trusted",
        ];
        for origin in ["translation", "kernel-taint"] {
            for status in statuses {
                let mut atoms = BTreeMap::new();
                let mut atom = make_atom("rust", "exec", "src/lib.rs", "f");
                set_status(&mut atom, status);
                set_origin(&mut atom, origin);
                atoms.insert("probe:pkg/1.0/f()".to_string(), atom);

                let result = summarize_atoms(&atoms).unwrap();
                let verified_status = status == "verified" || status == "transitively-verified";
                match (origin, verified_status) {
                    ("translation", true) => {
                        assert_eq!(
                            result.imported_verified,
                            vec!["probe:pkg/1.0/f()"],
                            "translation × {status}"
                        );
                        assert!(result.verified_entrypoints.is_empty());
                        assert!(result.verified_functions.is_empty());
                        assert!(result.verified_lemmas.is_empty());
                    }
                    (_, false) => {
                        assert!(
                            all_lists(&result).is_empty(),
                            "{origin} × {status} must appear in no list"
                        );
                    }
                    ("kernel-taint", true) => {
                        assert_eq!(
                            result.verified_entrypoints,
                            vec!["probe:pkg/1.0/f()"],
                            "kernel-taint × {status} stays in the local partitions"
                        );
                        assert!(result.imported_verified.is_empty());
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    // A translated verified atom never counts as a local result, whatever its
    // shape (here a lemma-shaped Lean atom, not just Rust exec).
    #[test]
    fn test_translation_origin_excluded_from_lemmas() {
        let mut atoms = BTreeMap::new();

        let mut lean = make_atom("lean", "theorem", "Src/Thm.lean", "thm");
        set_verified(&mut lean);
        set_origin(&mut lean, "translation");
        atoms.insert("probe:Pkg.thm".to_string(), lean);

        let result = summarize_atoms(&atoms).unwrap();
        assert!(result.verified_lemmas.is_empty());
        assert_eq!(result.imported_verified, vec!["probe:Pkg.thm"]);
    }

    // Plan §3 test (o): a verified Lean theorem plus its bound blueprint node
    // reports exactly one verified lemma — blueprint atoms appear in no list.
    #[test]
    fn test_blueprint_atoms_excluded_from_partitions() {
        let mut atoms = BTreeMap::new();

        let mut theorem = make_atom("lean", "theorem", "Src/Thm.lean", "main_theorem");
        set_verified(&mut theorem);
        atoms.insert("probe:Pkg.main_theorem".to_string(), theorem);

        let mut node = make_atom("blueprint", "node", "blueprint/src/content.tex", "thm:main");
        set_verified(&mut node);
        add_dep(&mut node, "probe:Pkg.main_theorem");
        atoms.insert("probe:blueprint/thm:main".to_string(), node);

        let result = summarize_atoms(&atoms).unwrap();
        assert_eq!(result.verified_lemmas, vec!["probe:Pkg.main_theorem"]);
        assert!(result.verified_entrypoints.is_empty());
        assert!(result.verified_functions.is_empty());
        assert!(result.imported_verified.is_empty());
    }

    // Blueprint binding edges are excluded from the depended_upon set, so a
    // Rust entrypoint referenced only by a blueprint node stays an entrypoint.
    #[test]
    fn test_blueprint_deps_do_not_alter_entrypoints() {
        let mut atoms = BTreeMap::new();

        let mut ep = make_atom("rust", "exec", "src/lib.rs", "compress");
        set_verified(&mut ep);
        atoms.insert("probe:pkg/1.0/compress()".to_string(), ep);

        let mut node = make_atom(
            "blueprint",
            "node",
            "blueprint/src/content.tex",
            "def:compress",
        );
        set_verified(&mut node);
        add_dep(&mut node, "probe:pkg/1.0/compress()");
        atoms.insert("probe:blueprint/def:compress".to_string(), node);

        let result = summarize_atoms(&atoms).unwrap();
        assert_eq!(
            result.verified_entrypoints,
            vec!["probe:pkg/1.0/compress()"]
        );
    }

    #[test]
    fn test_depended_upon_is_not_entrypoint() {
        let mut atoms = BTreeMap::new();

        let mut inner = make_atom("rust", "exec", "src/field.rs", "reduce");
        set_verified(&mut inner);
        atoms.insert("probe:pkg/1.0/reduce()".to_string(), inner);

        let mut outer = make_atom("rust", "exec", "src/lib.rs", "compress");
        set_verified(&mut outer);
        add_dep(&mut outer, "probe:pkg/1.0/reduce()");
        atoms.insert("probe:pkg/1.0/compress()".to_string(), outer);

        let result = summarize_atoms(&atoms).unwrap();
        assert_eq!(
            result.verified_entrypoints,
            vec!["probe:pkg/1.0/compress()"]
        );
        assert_eq!(result.verified_functions, vec!["probe:pkg/1.0/reduce()"]);
        assert!(result.verified_lemmas.is_empty());
    }
}
