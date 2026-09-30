//! Generated-input law and boundary-parity tests for the merge operator.
//!
//! The unit suites in src/commands/merge.rs pin the P4/P5 laws and the P27
//! record semantics on hand-picked fixtures. This suite closes the
//! example-based gap (test-report I1) by sweeping generated inputs instead:
//! both PR #81 review counterexamples were law violations on shapes outside
//! the fixture family, so the laws are exercised here over every combination
//! of a small adversarial vocabulary (stubs vs distinct reals, dotted-alias
//! keys, tied identity triples, empty-method twins, empty and exotic record
//! fields), on the enriched and raw paths alike.
//!
//! Adapted from the cross-model (codex) verification pass over commit
//! 989b7f4; all four tests passed against that commit unchanged.

use jsonschema::Validator;
use probe::commands::merge::{merge_atom_maps, merge_atom_maps_raw};
use probe::commands::propagate::prepare_atoms;
use probe::types::{load_mappings, Atom, Mapping, MAPPING_CONFIDENCE_VALUES};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// `kind` 0 = stub (empty code-path, lines 0,0 — P3); 1 and 2 = two
/// *distinct* real atoms, so equal-key pairs exercise real-vs-real
/// first-wins as well as stub replacement.
fn atom(kind: usize) -> Atom {
    serde_json::from_value(json!({
        "display-name": "f", "dependencies": [], "code-module": "",
        "code-path": if kind == 0 { "" } else if kind == 1 { "one.rs" } else { "two.rs" },
        "code-text": {"lines-start": 0, "lines-end": 0},
        "kind": "exec", "language": "rust"
    }))
    .unwrap()
}

fn single(value: Atom, key: &str) -> BTreeMap<String, Atom> {
    BTreeMap::from([(key.to_owned(), value)])
}

fn merge(
    inputs: Vec<BTreeMap<String, Atom>>,
    raw: bool,
    mappings: Option<&[Mapping]>,
) -> BTreeMap<String, Atom> {
    if raw {
        merge_atom_maps_raw(inputs, mappings).unwrap().0
    } else {
        merge_atom_maps(inputs, mappings).unwrap().0
    }
}

/// P4 (associativity, mapping compatibility) and P5/P27 (self-merge
/// idempotence, record-set union) over every triple of generated variants:
/// 3 atom kinds x 6 record shapes x 2 keys (plain and dotted alias), with
/// `mapped-from` seeded in *reversed* order so canonical sorting is load-
/// bearing, on both the enriched and raw paths — 18^3 x 2 associativity
/// cases. The record shapes include the review counterexamples: tied
/// identity triples via an empty-method twin, targets differing only by
/// trailing dots, and exotic-but-valid method strings.
#[test]
fn laws_hold_over_generated_record_variants() {
    let record_sets = [
        None,
        Some(json!([])),
        Some(json!([{"target":"f", "confidence":"exact"}])),
        Some(json!([
            {"target":"f..", "confidence":"exact", "method":""},
            {"target":"f", "confidence":"exact"},
            {"target":"f.", "confidence":"exact", "method":""}
        ])),
        Some(json!([
            {"target":"", "confidence":"manual", "method":"\u{0}"},
            {"target":"f", "confidence":"exact", "method":"é"}
        ])),
        Some(json!([
            {"target":"f.", "confidence":"heuristic", "method":" "},
            {"target":"f", "confidence":"exact"}
        ])),
    ];
    let mut variants = Vec::new();
    for kind in 0..3 {
        for (index, records) in record_sets.iter().enumerate() {
            let mut value = atom(kind);
            if let Some(records) = records {
                value.extensions.insert("maps-to".into(), records.clone());
                let mut reverse = records.as_array().unwrap().clone();
                reverse.reverse();
                value
                    .extensions
                    .insert("mapped-from".into(), json!(reverse));
            }
            variants.push(single(value, if index % 2 == 0 { "f" } else { "f.." }));
        }
    }
    // The mapping's endpoints normalize onto the shared key, and its empty
    // method canonicalizes to absent (P8/P27).
    let mappings = vec![Mapping {
        from: "f..".into(),
        to: "f.".into(),
        confidence: "exact".into(),
        method: Some("".into()),
    }];
    for raw in [false, true] {
        for first in &variants {
            // Self-merge idempotence: the record union is a set (P27).
            let canonical = merge(vec![first.clone()], raw, None);
            assert_eq!(
                merge(vec![first.clone(), first.clone()], raw, None),
                canonical
            );
            for second in &variants {
                // Mapping compatibility: F_M(mu(A, B)) = mu(F_M(A), F_M(B)).
                let flat = merge(vec![first.clone(), second.clone()], raw, Some(&mappings));
                let mapped_first = merge(vec![first.clone()], raw, Some(&mappings));
                let mapped_second = merge(vec![second.clone()], raw, Some(&mappings));
                assert_eq!(flat, merge(vec![mapped_first, mapped_second], raw, None));
                for third in &variants {
                    // Associativity: both nestings equal the flat 3-way merge.
                    let flat = merge(
                        vec![first.clone(), second.clone(), third.clone()],
                        raw,
                        None,
                    );
                    let left_pair = merge(vec![first.clone(), second.clone()], raw, None);
                    let right_pair = merge(vec![second.clone(), third.clone()], raw, None);
                    assert_eq!(flat, merge(vec![left_pair, third.clone()], raw, None));
                    assert_eq!(flat, merge(vec![first.clone(), right_pair], raw, None));
                }
            }
        }
    }
}

/// P27 fail-closed validation is *exactly* schema parity: for every generated
/// record-field shape (7 non-array values + 5 targets x 6 confidences x
/// 6 methods x extra-field on/off, per field), the runtime boundaries accept
/// iff every record — after the documented `method: ""` canonicalization —
/// validates against the executable schema's `correspondenceRecord`.
/// Checked at every boundary: `prepare_atoms` (enrich), `merge_atom_maps` /
/// `merge_atom_maps_raw` with the bad input in each of three positions
/// (rejections carry the right `input #N of M` ordinal, accepted outputs
/// carry only schema-valid records), and intra-input alias collisions with
/// the malformed side as survivor and as loser.
#[test]
fn runtime_acceptance_matches_schema_validity_at_every_boundary() {
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/atom-envelope.schema.json")).unwrap();
    let validator = Validator::new(&schema["$defs"]["correspondenceRecord"]).unwrap();
    let targets = [
        None,
        Some(Value::Null),
        Some(json!(3)),
        Some(json!("")),
        Some(json!("f...")),
    ];
    let confidences = [
        None,
        Some(Value::Null),
        Some(json!(false)),
        Some(json!("high")),
        Some(json!("exact")),
        Some(json!("manual")),
    ];
    let methods = [
        None,
        Some(Value::Null),
        Some(json!(3)),
        Some(json!("")),
        Some(json!(" ")),
        Some(json!("é")),
    ];
    let mut cases = vec![
        Value::Null,
        json!(false),
        json!("bad"),
        json!({}),
        json!([null]),
        json!([3]),
        json!([]),
    ];
    for target in &targets {
        for confidence in &confidences {
            for method in &methods {
                for extra in [false, true] {
                    let mut record = serde_json::Map::new();
                    if let Some(target) = target {
                        record.insert("target".into(), target.clone());
                    }
                    if let Some(confidence) = confidence {
                        record.insert("confidence".into(), confidence.clone());
                    }
                    if let Some(method) = method {
                        record.insert("method".into(), method.clone());
                    }
                    if extra {
                        record.insert("extra".into(), json!(0));
                    }
                    cases.push(json!([record]));
                }
            }
        }
    }
    for field in ["maps-to", "mapped-from"] {
        for records in &cases {
            let accepted = records.as_array().is_some_and(|records| {
                records.iter().all(|record| {
                    let mut record = record.clone();
                    if record.get("method") == Some(&json!("")) {
                        record.as_object_mut().unwrap().remove("method");
                    }
                    validator.is_valid(&record)
                })
            });
            let mut value = atom(0);
            value.extensions.insert(field.into(), records.clone());
            let input = single(value.clone(), "f.");
            assert_eq!(
                prepare_atoms(input.clone()).is_ok(),
                accepted,
                "prepare {field}: {records}"
            );
            for raw in [false, true] {
                for position in 0..3 {
                    let mut inputs = vec![single(atom(1), "f"); 3];
                    inputs[position] = input.clone();
                    let result = if raw {
                        merge_atom_maps_raw(inputs, None)
                    } else {
                        merge_atom_maps(inputs, None)
                    };
                    assert_eq!(
                        result.is_ok(),
                        accepted,
                        "{field}: {records}, raw={raw}, position={position}"
                    );
                    match result {
                        Err(error) => {
                            assert!(error.starts_with(&format!("input #{} of 3:", position + 1)))
                        }
                        Ok((output, _)) => {
                            for record in output["f"].extensions[field].as_array().unwrap() {
                                assert!(validator.is_valid(record));
                            }
                        }
                    }
                }
            }
            let aliases = BTreeMap::from([("f".into(), atom(1)), ("f.".into(), value.clone())]);
            assert_eq!(
                merge_atom_maps_raw(vec![aliases.clone()], None).is_ok(),
                accepted
            );
            assert_eq!(prepare_atoms(aliases).is_ok(), accepted);
            let aliases = BTreeMap::from([("f".into(), value), ("f.".into(), atom(1))]);
            assert_eq!(
                merge_atom_maps_raw(vec![aliases.clone()], None).is_ok(),
                accepted
            );
            assert_eq!(prepare_atoms(aliases).is_ok(), accepted);
        }
    }
}

/// Binary boundary for the record rejections: `probe merge` and
/// `probe enrich` exit non-zero on every malformed shape in either field,
/// write no output, and name the offending field (merge also names the
/// input ordinal) on stderr.
#[test]
fn cli_merge_and_enrich_reject_malformed_records_before_writing() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.json");
    let output = directory.path().join("output.json");
    let fixture = "tests/fixtures/merge_test/atoms_a.json";
    let original: Value = serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
    for field in ["maps-to", "mapped-from"] {
        for invalid in [
            json!("bad"),
            json!([null]),
            json!([{"target":"f", "confidence":"exact", "extra":0}]),
            json!([{"target":"f", "confidence":"bad"}]),
            json!([{"target":null, "confidence":"exact"}]),
            json!([{"target":"f", "confidence":"exact", "method":null}]),
        ] {
            let mut document = original.clone();
            document["data"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap()[field] = invalid;
            std::fs::write(&input, document.to_string()).unwrap();
            for command in ["merge", "enrich"] {
                let mut process = std::process::Command::new(env!("CARGO_BIN_EXE_probe"));
                process.arg(command);
                if command == "merge" {
                    process.arg(fixture);
                }
                let result = process.arg(&input).arg("-o").arg(&output).output().unwrap();
                assert!(!result.status.success(), "{command}: {field}");
                assert!(!output.exists());
                let error = String::from_utf8_lossy(&result.stderr);
                assert!(error.contains(field), "{error}");
                if command == "merge" {
                    assert!(error.contains("input #2 of 2"), "{error}");
                }
            }
        }
    }
}

/// Mapping canonicalization end to end: 72 generated mappings (every
/// confidence x method-encoding x trailing-dot suffix) collapse to their
/// canonical record set with the exact `records_attached` count;
/// re-application in reversed order attaches nothing and changes nothing
/// (P13 set-likeness); a dotted-key input serializes byte-identically to the
/// plain-key one (P8/P14). The loader and the in-memory merge boundary agree
/// with the vocabulary — including the case-sensitive and empty-string
/// negatives — and empty input lists are fine on both paths.
#[test]
fn mapping_canonicalization_counts_loader_and_empty_inputs() {
    for raw in [false, true] {
        assert!(merge(vec![], raw, None).is_empty());
        assert!(merge(vec![BTreeMap::new()], raw, Some(&[])).is_empty());
    }
    let mut mappings = Vec::new();
    for confidence in MAPPING_CONFIDENCE_VALUES {
        for method in [None, Some(""), Some("é"), Some(" ")] {
            for suffix in ["", ".", "..."] {
                mappings.push(Mapping {
                    from: format!("f{suffix}"),
                    to: format!("f{suffix}"),
                    confidence: confidence.into(),
                    method: method.map(str::to_owned),
                });
            }
        }
    }
    // 6 confidences x 3 canonical methods (absent==""; "é"; " ") x 2 fields.
    let (first, stats) =
        merge_atom_maps_raw(vec![single(atom(1), "f...")], Some(&mappings)).unwrap();
    assert_eq!(stats.records_attached, 36);
    mappings.reverse();
    let (again, stats) = merge_atom_maps_raw(vec![first.clone()], Some(&mappings)).unwrap();
    assert_eq!(stats.records_attached, 0);
    assert_eq!(first, again);
    let (reordered, stats) =
        merge_atom_maps_raw(vec![single(atom(1), "f")], Some(&mappings)).unwrap();
    assert_eq!(stats.records_attached, 36);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&reordered).unwrap()
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("mappings.json");
    for confidence in ["exact", "manual", "high", "", "EXACT"] {
        std::fs::write(
            &path,
            json!({"schema":"probe/mappings", "schema-version":"3.0",
                   "mappings":[{"from":"f..", "to":"f.", "confidence":confidence, "method":""}]})
            .to_string(),
        )
        .unwrap();
        let loaded = load_mappings(&path);
        let valid = MAPPING_CONFIDENCE_VALUES.contains(&confidence);
        assert_eq!(loaded.is_ok(), valid);
        let direct = [Mapping {
            from: "absent".into(),
            to: "absent".into(),
            confidence: confidence.into(),
            method: None,
        }];
        assert_eq!(merge_atom_maps_raw(vec![], Some(&direct)).is_ok(), valid);
        if let Ok(loaded) = loaded {
            assert_eq!(loaded[0].method, None);
            assert_eq!(loaded[0].from, "f");
            assert_eq!(loaded[0].to, "f");
        }
    }
}
