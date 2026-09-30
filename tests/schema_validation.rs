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
        "schema-version": "3.0",
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
        "schema-version": "3.0",
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
        "schema-version": "3.0",
        "tool": { "name": "probe", "version": "0.1.0", "command": "merge" },
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
        "schema-version": "3.0",
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
fn single_tool_rust_extract_envelope_is_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-rust/extract",
        "schema-version": "3.0",
        "tool": { "name": "probe-rust", "version": "0.1.0", "command": "extract" },
        "source": {
            "repo": "https://github.com/org/my-crate",
            "commit": "abc123",
            "language": "rust",
            "package": "my-crate",
            "package-version": "1.0.0"
        },
        "timestamp": "2026-03-17T12:00:00Z",
        "data": {
            "probe:my-crate/1.0.0/lib/main()": {
                "display-name": "main",
                "dependencies": [],
                "code-module": "lib",
                "code-path": "src/lib.rs",
                "code-text": { "lines-start": 1, "lines-end": 10 },
                "kind": "exec",
                "language": "rust"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "probe-rust/extract envelope should validate: {result:?}"
    );
}

#[test]
fn composed_aeneas_extract_envelope_is_valid() {
    // probe-aeneas emits a *composed* envelope under its own schema string:
    // `inputs: [Rust, Lean]` and no `source`. The branch split keys on
    // provenance shape (ADR-006), so this must validate as-is — no synthetic
    // `source` masking the real shape.
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-aeneas/extract",
        "schema-version": "3.0",
        "tool": { "name": "probe-aeneas", "version": "0.21.0", "command": "extract" },
        "inputs": [
            {
                "schema": "probe-rust/extract",
                "source": {
                    "repo": "https://github.com/org/my-project",
                    "commit": "def456",
                    "language": "rust",
                    "package": "my-project",
                    "package-version": "1.0.0"
                }
            },
            {
                "schema": "probe-lean/extract",
                "source": {
                    "repo": "https://github.com/org/my-project-lean",
                    "commit": "a1b2c3",
                    "language": "lean",
                    "package": "MyProjectLean",
                    "package-version": "0.1.0"
                }
            }
        ],
        "timestamp": "2026-03-17T12:00:00Z",
        "data": {
            "probe:my-project/1.0.0/lib/f()": {
                "display-name": "f",
                "dependencies": [],
                "code-module": "lib",
                "code-path": "src/lib.rs",
                "code-text": { "lines-start": 1, "lines-end": 5 },
                "kind": "exec",
                "language": "rust"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "composed probe-aeneas/extract envelope should validate: {result:?}"
    );
}

#[test]
fn single_tool_vcvio_extract_with_status_origin_is_valid() {
    // probe-vcvio re-emits probe-lean atoms under its own identity; its
    // schema string joins the single-tool branch (ADR-006). The atom carries
    // a status-origin marker — a flattened extension field.
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-vcvio/extract",
        "schema-version": "3.0",
        "tool": { "name": "probe-vcvio", "version": "0.2.0", "command": "extract" },
        "source": {
            "repo": "https://github.com/org/my-protocol",
            "commit": "fedcba",
            "language": "lean",
            "package": "MyProtocol",
            "package-version": "0.1.0"
        },
        "timestamp": "2026-03-17T12:00:00Z",
        "data": {
            "probe:MyProtocol.encrypt": {
                "display-name": "encrypt",
                "dependencies": [],
                "code-module": "MyProtocol",
                "code-path": "MyProtocol/Basic.lean",
                "code-text": { "lines-start": 3, "lines-end": 9 },
                "kind": "def",
                "language": "lean",
                "verification-status": "verified",
                "status-origin": "translation"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "probe-vcvio/extract envelope with status-origin should validate: {result:?}"
    );
}

#[test]
fn status_origin_outside_the_enum_is_rejected() {
    // status-origin is an enum over exactly "translation" and "kernel-taint"
    // (ADR-006 Decision 2); any other value must fail validation.
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-lean/extract",
        "schema-version": "3.0",
        "tool": { "name": "probe-lean", "version": "0.16.0", "command": "extract" },
        "source": {
            "repo": "https://github.com/org/proj",
            "commit": "abcdef",
            "language": "lean",
            "package": "Proj",
            "package-version": "0.1.0"
        },
        "timestamp": "2026-03-17T12:00:00Z",
        "data": {
            "probe:Proj.thm": {
                "display-name": "thm",
                "dependencies": [],
                "code-module": "Proj",
                "code-path": "Proj/Basic.lean",
                "code-text": { "lines-start": 3, "lines-end": 9 },
                "kind": "theorem",
                "language": "lean",
                "verification-status": "verified",
                "status-origin": "graph-taint"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_err(),
        "a status-origin value outside the two-value enum must be rejected"
    );
}

#[test]
fn projected_atoms_envelope_is_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe/projected-atoms",
        "schema-version": "3.1",
        "tool": { "name": "probe", "version": "0.5.0", "command": "project" },
        "inputs": [
            {
                "schema": "probe-verus/atoms",
                "source": {
                    "repo": "https://github.com/org/project",
                    "commit": "abc123",
                    "language": "rust",
                    "package": "my-crate",
                    "package-version": "1.0.0"
                }
            }
        ],
        "timestamp": "2026-03-17T12:00:00Z",
        "projection": {
            "mappings-file": "mappings.json",
            "seeds": 1,
            "forward-depth": 2,
            "reverse-depth": 0,
            "atoms-in": 10,
            "atoms-out": 1,
            "deps-trimmed": 3
        },
        "data": {
            "probe:my-crate/1.0.0/mod/f()": {
                "display-name": "f",
                "dependencies": [],
                "code-module": "mod",
                "code-path": "src/lib.rs",
                "code-text": { "lines-start": 1, "lines-end": 5 },
                "kind": "exec",
                "language": "rust"
            }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "probe/projected-atoms envelope should validate: {result:?}"
    );

    // Without its projection block, a projected envelope is invalid.
    let mut without_block = doc.clone();
    without_block.as_object_mut().unwrap().remove("projection");
    assert!(
        validator.validate(&without_block).is_err(),
        "probe/projected-atoms without a projection block should be rejected"
    );
}

#[test]
fn envelope_with_both_source_and_inputs_is_rejected() {
    // The two atoms branches are keyed on provenance shape; an envelope
    // carrying both `source` and `inputs` matches neither.
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let source = json!({
        "repo": "https://github.com/org/project",
        "commit": "abc123",
        "language": "rust",
        "package": "my-crate",
        "package-version": "1.0.0"
    });
    let doc = json!({
        "schema": "probe-verus/atoms",
        "schema-version": "3.0",
        "tool": { "name": "probe-verus", "version": "2.0.0", "command": "atomize" },
        "source": source,
        "inputs": [ { "schema": "probe-verus/atoms", "source": source } ],
        "timestamp": "2026-03-05T14:30:00Z",
        "data": {}
    });

    assert!(
        validator.validate(&doc).is_err(),
        "an envelope with both source and inputs should be rejected"
    );
}

#[test]
fn single_tool_specs_envelope_is_valid() {
    // Regression: specs/proofs data values are not full atoms (P7). A single-tool
    // specs envelope with entries like `{ "specified": true }` must validate.
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-verus/specs",
        "schema-version": "3.0",
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
            "probe:my-crate/1.0.0/mod/f()": { "specified": true, "has_requires": true, "has_ensures": false }
        }
    });

    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "probe-verus/specs envelope should validate: {result:?}"
    );
}

#[test]
fn single_tool_proofs_stubs_and_report_envelopes_are_valid() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");
    let source = json!({
        "repo": "https://github.com/org/project",
        "commit": "abc123",
        "language": "rust",
        "package": "my-crate",
        "package-version": "1.0.0"
    });

    for schema_value in [
        "probe-verus/proofs",
        "probe-verus/stubs",
        "probe-verus/verification-report",
    ] {
        let doc = json!({
            "schema": schema_value,
            "schema-version": "3.0",
            "tool": { "name": "probe-verus", "version": "2.0.0", "command": "atomize" },
            "source": source,
            "timestamp": "2026-03-05T14:30:00Z",
            "data": {
                "probe:my-crate/1.0.0/mod/f()": { "verified": true }
            }
        });

        let result = validator.validate(&doc);
        assert!(
            result.is_ok(),
            "{schema_value} envelope should validate: {result:?}"
        );
    }
}

#[test]
fn missing_required_field_is_rejected() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let doc = json!({
        "schema": "probe-verus/atoms",
        "schema-version": "3.0",
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

/// PR 3b (ADR-006): a merged envelope produced by the real merge operation —
/// correspondence records attached, labels re-enriched — validates against the
/// executable schema (envelope-level, not only map-level algebra).
#[test]
fn merged_envelope_with_correspondence_records_is_valid() {
    use probe::commands::merge::merge_atom_maps;
    use probe::types::{Atom, InputProvenance, Mapping, MergedAtomEnvelope, Source, Tool};
    use std::collections::BTreeMap;

    let atom = |code_path: &str, language: &str, kind: &str| -> Atom {
        serde_json::from_value(json!({
            "display-name": "add",
            "dependencies": [],
            "code-module": "m",
            "code-path": code_path,
            "code-text": { "lines-start": 1, "lines-end": 5 },
            "kind": kind,
            "language": language,
            "verification-status": "verified"
        }))
        .unwrap()
    };

    let mut rust_atoms = BTreeMap::new();
    rust_atoms.insert(
        "probe:dalek/4.1.3/scalar/add()".to_string(),
        atom("src/scalar.rs", "rust", "exec"),
    );
    let mut lean_atoms = BTreeMap::new();
    lean_atoms.insert(
        "probe:DalekLean.Scalar.add".to_string(),
        atom("DalekLean/Scalar.lean", "lean", "def"),
    );

    let mappings = vec![Mapping {
        from: "probe:dalek/4.1.3/scalar/add()".to_string(),
        to: "probe:DalekLean.Scalar.add".to_string(),
        confidence: "manual".to_string(),
        method: Some("hand-written".to_string()),
    }];

    let (merged, stats) =
        merge_atom_maps(vec![rust_atoms, lean_atoms], Some(&mappings)).expect("merge succeeds");
    assert_eq!(stats.records_attached, 2);

    let source = |language: &str, package: &str| Source {
        repo: "https://github.com/org/project".to_string(),
        commit: "abc123".to_string(),
        language: language.to_string(),
        package: package.to_string(),
        package_version: "1.0".to_string(),
        extensions: BTreeMap::new(),
    };
    let envelope = MergedAtomEnvelope {
        schema: "probe/merged-atoms".to_string(),
        schema_version: "3.1".to_string(),
        tool: Tool {
            name: "probe".to_string(),
            version: "0.5.0".to_string(),
            command: "merge".to_string(),
        },
        inputs: vec![
            InputProvenance {
                schema: "probe-verus/atoms".to_string(),
                source: source("rust", "dalek"),
            },
            InputProvenance {
                schema: "probe-lean/extract".to_string(),
                source: source("lean", "DalekLean"),
            },
        ],
        timestamp: "2026-09-29T12:00:00Z".to_string(),
        data: merged,
    };

    let doc = serde_json::to_value(&envelope).expect("serializes");
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");
    let result = validator.validate(&doc);
    assert!(
        result.is_ok(),
        "merged envelope with records should validate: {result:?}"
    );

    // The records really are in the serialized output.
    let rust_atom = &doc["data"]["probe:dalek/4.1.3/scalar/add()"];
    assert_eq!(
        rust_atom["maps-to"],
        json!([{ "target": "probe:DalekLean.Scalar.add",
                 "confidence": "manual", "method": "hand-written" }])
    );
    let lean_atom = &doc["data"]["probe:DalekLean.Scalar.add"];
    assert_eq!(
        lean_atom["mapped-from"],
        json!([{ "target": "probe:dalek/4.1.3/scalar/add()",
                 "confidence": "manual", "method": "hand-written" }])
    );
}

/// The executable schema constrains correspondence records: an out-of-enum
/// confidence, a missing target, and extra fields are all rejected.
#[test]
fn malformed_correspondence_records_are_rejected() {
    let schema = load_schema();
    let validator = Validator::new(&schema).expect("valid schema");

    let envelope_with_record = |record: serde_json::Value| {
        json!({
            "schema": "probe-verus/atoms",
            "schema-version": "3.0",
            "tool": { "name": "probe-verus", "version": "2.0.0", "command": "atomize" },
            "source": {
                "repo": "r", "commit": "c", "language": "rust",
                "package": "p", "package-version": "1.0"
            },
            "timestamp": "2026-03-05T14:30:00Z",
            "data": {
                "probe:a/1.0/f()": {
                    "display-name": "f",
                    "dependencies": [],
                    "code-module": "",
                    "code-path": "src/lib.rs",
                    "code-text": { "lines-start": 1, "lines-end": 2 },
                    "kind": "exec",
                    "language": "rust",
                    "maps-to": [record]
                }
            }
        })
    };

    let good = envelope_with_record(json!({"target": "probe:Pkg.f", "confidence": "exact"}));
    assert!(validator.validate(&good).is_ok());

    for bad in [
        json!({"target": "probe:Pkg.f", "confidence": "high"}),
        json!({"confidence": "exact"}),
        json!({"target": "probe:Pkg.f", "confidence": "exact", "extra": 1}),
        // Empty method: the canonical encoding of "no method" is omission
        // (P27); merge canonicalizes, the schema forbids.
        json!({"target": "probe:Pkg.f", "confidence": "exact", "method": ""}),
    ] {
        let doc = envelope_with_record(bad.clone());
        assert!(
            validator.validate(&doc).is_err(),
            "record {bad} should be rejected"
        );
    }
}

/// Drift guard: the confidence vocabulary the code validates against
/// (`MAPPING_CONFIDENCE_VALUES`, used by `load_mappings` and the merge
/// boundaries) is exactly the executable schema's `correspondenceRecord`
/// confidence enum.
#[test]
fn confidence_vocabulary_matches_executable_schema() {
    let schema = load_schema();
    let enum_values: Vec<&str> = schema["$defs"]["correspondenceRecord"]["properties"]
        ["confidence"]["enum"]
        .as_array()
        .expect("confidence enum present")
        .iter()
        .map(|v| v.as_str().expect("enum values are strings"))
        .collect();
    assert_eq!(enum_values, probe::types::MAPPING_CONFIDENCE_VALUES);
}
