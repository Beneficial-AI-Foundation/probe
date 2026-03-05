use jsonschema::Validator;
use serde_json::json;

fn load_schema() -> serde_json::Value {
    let schema_str =
        std::fs::read_to_string("schemas/atom-envelope.schema.json").expect("schema file exists");
    serde_json::from_str(&schema_str).expect("valid JSON")
}

#[test]
fn single_tool_verus_envelope_is_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-verus/atoms",
        "schema-version": "2.0",
        "tool": { "name": "probe-verus", "version": "2.0.0", "command": "atomize" },
        "source": {
            "repo": "https://github.com/org/project",
            "commit": "abc123",
            "language": "rust",
            "package": "curve25519-dalek",
            "package-version": "4.1.3"
        },
        "timestamp": "2026-03-05T14:30:00Z",
        "data": {
            "probe:curve25519-dalek/4.1.3/scalar/add()": {
                "display-name": "add",
                "dependencies": [],
                "code-module": "scalar",
                "code-path": "src/scalar.rs",
                "code-text": { "lines-start": 10, "lines-end": 20 },
                "kind": "exec",
                "language": "rust"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(result.is_ok(), "Verus envelope should validate: {result:?}");
}

#[test]
fn single_tool_lean_envelope_is_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-lean/atoms",
        "schema-version": "2.0",
        "tool": { "name": "probe-lean", "version": "1.0.0", "command": "atomize" },
        "source": {
            "repo": "https://github.com/org/arklib",
            "commit": "f6e5d4c",
            "language": "lean",
            "package": "Arklib",
            "package-version": "f6e5d4c"
        },
        "timestamp": "2026-03-05T14:30:00Z",
        "data": {
            "probe:ArkLib.SumCheck.Protocol.Prover.prove": {
                "display-name": "prove",
                "dependencies": ["probe:ArkLib.SumCheck.Protocol.Verifier.verify"],
                "code-module": "ArkLib.SumCheck.Protocol",
                "code-path": "ArkLib/SumCheck/Protocol.lean",
                "code-text": { "lines-start": 42, "lines-end": 67 },
                "kind": "def",
                "language": "lean"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(result.is_ok(), "Lean envelope should validate: {result:?}");
}

#[test]
fn merged_atoms_envelope_is_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe/merged-atoms",
        "schema-version": "2.0",
        "tool": { "name": "probe", "version": "0.1.0", "command": "merge-atoms" },
        "inputs": [
            {
                "schema": "probe-verus/atoms",
                "source": {
                    "repo": "https://github.com/org/project",
                    "commit": "abc123",
                    "language": "rust",
                    "package": "curve25519-dalek",
                    "package-version": "4.1.3"
                }
            },
            {
                "schema": "probe-lean/atoms",
                "source": {
                    "repo": "https://github.com/org/lean-project",
                    "commit": "def456",
                    "language": "lean",
                    "package": "DalekLean",
                    "package-version": "0.1.0"
                }
            }
        ],
        "timestamp": "2026-03-05T15:00:00Z",
        "data": {
            "probe:curve25519-dalek/4.1.3/scalar/add()": {
                "display-name": "add",
                "dependencies": [],
                "code-module": "scalar",
                "code-path": "src/scalar.rs",
                "code-text": { "lines-start": 10, "lines-end": 20 },
                "kind": "exec",
                "language": "rust"
            },
            "probe:DalekLean.Scalar.add": {
                "display-name": "add",
                "dependencies": [],
                "code-module": "DalekLean.Scalar",
                "code-path": "DalekLean/Scalar.lean",
                "code-text": { "lines-start": 5, "lines-end": 15 },
                "kind": "def",
                "language": "lean"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "Merged envelope should validate: {result:?}"
    );
}

#[test]
fn atom_with_extensions_is_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-verus/atoms",
        "schema-version": "2.0",
        "tool": { "name": "probe-verus", "version": "2.0.0", "command": "atomize" },
        "source": {
            "repo": "https://github.com/org/project",
            "commit": "abc123",
            "language": "rust",
            "package": "my-crate",
            "package-version": "1.0.0"
        },
        "timestamp": "2026-03-05T14:30:00Z",
        "data": {
            "probe:my-crate/1.0.0/mod/f()": {
                "display-name": "f",
                "dependencies": ["probe:my-crate/1.0.0/mod/g()"],
                "code-module": "mod",
                "code-path": "src/lib.rs",
                "code-text": { "lines-start": 10, "lines-end": 20 },
                "kind": "exec",
                "language": "rust",
                "dependencies-with-locations": [
                    { "code-name": "probe:my-crate/1.0.0/mod/g()", "location": "inner", "line": 15 }
                ]
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "Atom with extensions should validate: {result:?}"
    );
}

#[test]
fn missing_required_field_is_rejected() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-verus/atoms",
        "schema-version": "2.0",
        "tool": { "name": "probe-verus", "version": "2.0.0", "command": "atomize" },
        "source": {
            "repo": "https://github.com/org/project",
            "commit": "abc123",
            "language": "rust",
            "package": "my-crate",
            "package-version": "1.0.0"
        },
        "timestamp": "2026-03-05T14:30:00Z",
        "data": {
            "probe:my-crate/1.0.0/mod/f()": {
                "display-name": "f",
                "dependencies": [],
                "code-module": "mod",
                "code-path": "src/lib.rs",
                "code-text": { "lines-start": 10, "lines-end": 20 },
                "kind": "exec"
                // "language" is missing
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(result.is_err(), "Missing 'language' should be rejected");
}
