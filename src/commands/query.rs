// @kb: kb/engineering/properties.md#p1-envelope-completeness
// @kb: kb/engineering/properties.md#p3-stub-detection-is-structural
// @kb: kb/engineering/properties.md#p14-deterministic-output
// @kb: kb/engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope

use crate::types::{load_atom_file, Atom, InputProvenance, Tool};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Schema 2.0 envelope for query output.
#[derive(serde::Serialize)]
struct QueryEnvelope {
    schema: &'static str,
    #[serde(rename = "schema-version")]
    schema_version: &'static str,
    tool: Tool,
    inputs: Vec<InputProvenance>,
    timestamp: String,
    data: QueryResult,
}

/// Payload of a query result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryResult {
    pub entrypoints: Vec<String>,
    pub verified_dependencies: Vec<String>,
}

fn is_test(atom: &Atom) -> bool {
    atom.code_module.contains("test") || atom.display_name.contains("test")
}

fn is_verified(atom: &Atom) -> bool {
    atom.extensions
        .get("verification-status")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "verified")
}

// @kb: kb/engineering/schema.md#language-assignment-for-verus-atoms
fn is_rust_exec(atom: &Atom) -> bool {
    atom.language == "rust" && atom.kind == "exec"
}

/// Partition verified atoms into entrypoints and verified dependencies.
///
/// **Entrypoints**: verified, non-stub, non-test Rust `exec` atoms whose
/// code-name never appears in any atom's `dependencies` array.
///
/// **Verified dependencies**: all remaining verified atoms.
///
/// The two lists are a partition: `entrypoints + verified_deps == total verified`.
pub fn query_atoms(atoms: &BTreeMap<String, Atom>) -> QueryResult {
    let depended_upon: BTreeSet<&str> = atoms
        .values()
        .flat_map(|atom| atom.dependencies.iter())
        .map(String::as_str)
        .collect();

    let mut entrypoints: Vec<String> = Vec::new();
    let mut verified_deps: Vec<String> = Vec::new();

    for (code_name, atom) in atoms {
        if !is_verified(atom) {
            continue;
        }
        let is_entrypoint = !atom.is_stub()
            && !is_test(atom)
            && is_rust_exec(atom)
            && !depended_upon.contains(code_name.as_str());

        if is_entrypoint {
            entrypoints.push(code_name.clone());
        } else {
            verified_deps.push(code_name.clone());
        }
    }

    QueryResult {
        entrypoints,
        verified_dependencies: verified_deps,
    }
}

/// CLI entry point: load atom file, compute query, emit envelope.
pub fn cmd_query(input: &Path, output: Option<&Path>) {
    let (atoms, provenance) = match load_atom_file(input) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let result = query_atoms(&atoms);

    eprintln!(
        "Verified: {}  |  Entrypoints: {}  |  Verified deps: {}",
        result.entrypoints.len() + result.verified_dependencies.len(),
        result.entrypoints.len(),
        result.verified_dependencies.len()
    );

    let envelope = QueryEnvelope {
        schema: "probe/query",
        schema_version: "2.0",
        tool: Tool {
            name: "probe".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: "query".to_string(),
        },
        inputs: provenance,
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        data: result,
    };

    let json = serde_json::to_string_pretty(&envelope).expect("failed to serialize output");

    match output {
        Some(path) => {
            std::fs::write(path, &json).unwrap_or_else(|e| {
                eprintln!("Error writing {}: {e}", path.display());
                std::process::exit(1);
            });
            eprintln!("Wrote {}", path.display());
        }
        None => println!("{json}"),
    }
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

    fn set_verified(atom: &mut Atom) {
        atom.extensions.insert(
            "verification-status".to_string(),
            serde_json::Value::String("verified".to_string()),
        );
    }

    fn add_dep(atom: &mut Atom, dep: &str) {
        atom.dependencies.insert(dep.to_string());
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

        let result = query_atoms(&atoms);
        let total_verified = 3;
        assert_eq!(
            result.entrypoints.len() + result.verified_dependencies.len(),
            total_verified
        );
    }

    #[test]
    fn test_stubs_are_not_entrypoints() {
        let mut atoms = BTreeMap::new();

        let mut stub = make_atom("rust", "exec", "", "alloc_fn");
        set_verified(&mut stub);
        atoms.insert("probe:alloc/1.0/alloc_fn()".to_string(), stub);

        let result = query_atoms(&atoms);
        assert!(result.entrypoints.is_empty());
        assert_eq!(result.verified_dependencies.len(), 1);
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

        let result = query_atoms(&atoms);
        assert!(result.entrypoints.is_empty());
        assert_eq!(result.verified_dependencies.len(), 1);
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

        let result = query_atoms(&atoms);
        assert!(result.entrypoints.is_empty());
        assert_eq!(result.verified_dependencies.len(), 2);
    }

    #[test]
    fn test_unverified_atoms_excluded_from_both_lists() {
        let mut atoms = BTreeMap::new();

        let unverified = make_atom("rust", "exec", "src/lib.rs", "foo");
        atoms.insert("probe:pkg/1.0/foo()".to_string(), unverified);

        let result = query_atoms(&atoms);
        assert!(result.entrypoints.is_empty());
        assert!(result.verified_dependencies.is_empty());
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

        let result = query_atoms(&atoms);
        assert_eq!(result.entrypoints, vec!["probe:pkg/1.0/compress()"]);
        assert_eq!(result.verified_dependencies, vec!["probe:pkg/1.0/reduce()"]);
    }
}
