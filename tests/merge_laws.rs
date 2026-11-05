//! Generated-input law and boundary-parity tests for the merge operator.
//!
//! The unit suites in src/commands/merge.rs pin the P4/P5 laws and the P27
//! record semantics on hand-picked fixtures. This suite sweeps generated
//! inputs instead: both PR #81 review counterexamples were law violations on
//! shapes outside the fixture family, so the laws are exercised here over
//! every combination of a small adversarial vocabulary (stubs vs distinct
//! reals, dotted-alias keys, tied identity triples, empty-method twins, empty
//! and exotic record fields, categorized dependency arrays spelled as the
//! same or different sets), on the enriched and raw paths alike.
//!
//! The record sweeps were adapted from the cross-model (codex) verification
//! pass over commit 989b7f4.

use jsonschema::Validator;
use probe::commands::merge::{merge_atom_maps, merge_atom_maps_raw};
use probe::commands::propagate::prepare_atoms;
use probe::types::{
    load_mappings, Atom, Mapping, CATEGORIZED_DEPENDENCY_ARRAYS, MAPPING_CONFIDENCE_VALUES,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

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

/// The same input with every key's `f` prefix replaced by `prefix`, so it
/// shares no normalized key with an input keyed on `f`.
fn rekey(input: &BTreeMap<String, Atom>, prefix: &str) -> BTreeMap<String, Atom> {
    input
        .iter()
        .map(|(key, value)| (key.replacen('f', prefix, 1), value.clone()))
        .collect()
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
            // Mapping re-application is a no-op: F_M(F_M(A)) = F_M(A).
            let mapped = merge(vec![first.clone()], raw, Some(&mappings));
            assert_eq!(merge(vec![mapped.clone()], raw, Some(&mappings)), mapped);
            assert_eq!(
                merge(vec![first.clone(), first.clone()], raw, None),
                canonical
            );
            for second in &variants {
                // Mapping compatibility: F_M(mu(A, B)) = mu(F_M(A), F_M(B)).
                // Commutativity on disjoint keys: renaming every key of
                // `second` makes the inputs disjoint, so order cannot matter.
                let disjoint = rekey(second, "g");
                assert_eq!(
                    merge(vec![first.clone(), disjoint.clone()], raw, None),
                    merge(vec![disjoint, first.clone()], raw, None)
                );
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

/// Binary boundary for the record rejections: `probe merge`, `probe enrich`
/// and `probe project` exit non-zero on every malformed shape in either
/// field, write no output, and name the offending field (merge also names
/// the input ordinal) on stderr.
#[test]
fn cli_boundaries_reject_malformed_records_before_writing() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.json");
    let output = directory.path().join("output.json");
    let mappings = directory.path().join("mappings.json");
    std::fs::write(
        &mappings,
        json!({"schema": "probe/mappings", "schema-version": "1.0", "mappings": []}).to_string(),
    )
    .unwrap();
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
            for command in ["merge", "enrich", "project"] {
                let mut process = std::process::Command::new(env!("CARGO_BIN_EXE_probe"));
                process.arg(command);
                if command == "merge" {
                    process.arg(fixture);
                }
                process.arg(&input);
                if command == "project" {
                    process.arg("--mappings").arg(&mappings);
                }
                let result = process.arg("-o").arg(&output).output().unwrap();
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

/// P4 associativity and the enrichment fixed point with copied translated
/// labels (#92): a translated atom keeps the label it arrives with, stays a
/// seed, and is never relabeled. Two keys, `r` (Rust, depends on `s`) and
/// `s` (Lean source), each absent or drawn from a vocabulary of stubs,
/// distinct reals, local/translated/kernel-taint statuses, so stub
/// resolution and first-wins both decide which label survives.
#[test]
fn laws_hold_with_translated_labels() {
    fn variant(key: &str, code_path: &str, status: Option<&str>, origin: Option<&str>) -> Atom {
        let mut v = json!({
            "display-name": key, "dependencies": if key == "r" { json!(["s"]) } else { json!([]) },
            "code-module": "", "code-path": code_path,
            "code-text": {"lines-start": 0, "lines-end": 0},
            "kind": "exec", "language": if key == "r" { "rust" } else { "lean" }
        });
        if let Some(s) = status {
            v["verification-status"] = json!(s);
        }
        if let Some(o) = origin {
            v["status-origin"] = json!(o);
        }
        serde_json::from_value(v).unwrap()
    }
    let vocab = |key: &str| -> Vec<Option<Atom>> {
        let stub = Atom {
            dependencies: Default::default(),
            ..variant(key, "", None, None)
        };
        vec![
            None,
            Some(stub),
            Some(variant(key, "a", Some("unverified"), None)),
            Some(variant(key, "b", Some("transitively-verified"), None)),
            Some(variant(
                key,
                "c",
                Some("transitively-verified"),
                Some("translation"),
            )),
            Some(variant(key, "d", Some("verified"), Some("translation"))),
            Some(variant(
                key,
                "e",
                Some("transitively-verified"),
                Some("kernel-taint"),
            )),
        ]
    };
    let mut maps = Vec::new();
    for r in vocab("r") {
        for s in vocab("s") {
            let mut m = BTreeMap::new();
            if let Some(a) = r.clone() {
                m.insert("r".to_owned(), a);
            }
            if let Some(a) = s {
                m.insert("s".to_owned(), a);
            }
            maps.push(m);
        }
    }
    let as_json = |m: &BTreeMap<String, Atom>| serde_json::to_value(m).unwrap();
    for a in &maps {
        let once = merge(vec![a.clone()], false, None);
        assert_eq!(
            as_json(&merge(vec![once.clone()], false, None)),
            as_json(&once),
            "enrichment fixed point"
        );
        for b in &maps {
            let ab = merge(vec![a.clone(), b.clone()], false, None);
            for c in &maps {
                let left = merge(vec![ab.clone(), c.clone()], false, None);
                let bc = merge(vec![b.clone(), c.clone()], false, None);
                let right = merge(vec![a.clone(), bc], false, None);
                assert_eq!(as_json(&left), as_json(&right), "associativity");
            }
        }
    }
}

/// P4 associativity and the enrichment fixed point for copied labels next to
/// trusted boundaries (#92): copied `trusted` (a seed, never a boundary),
/// plain `trusted` (a boundary) and unmarked `verified` (a candidate) around
/// a translated `r` that depends on `s`, with an unmarked caller `c -> r`
/// that a translated or copied-trusted `r` must block.
#[test]
fn laws_hold_with_translated_labels_and_trust() {
    fn variant(key: &str, code_path: &str, status: Option<&str>, origin: Option<&str>) -> Atom {
        let mut v = json!({
            "display-name": key,
            "dependencies": match key { "r" => json!(["s"]), "c" => json!(["r"]), _ => json!([]) },
            "code-module": "", "code-path": code_path,
            "code-text": {"lines-start": 0, "lines-end": 0},
            "kind": "exec", "language": if key == "s" { "lean" } else { "rust" }
        });
        if let Some(s) = status {
            v["verification-status"] = json!(s);
        }
        if let Some(o) = origin {
            v["status-origin"] = json!(o);
        }
        serde_json::from_value(v).unwrap()
    }
    let stub = |key: &str| Atom {
        dependencies: Default::default(),
        ..variant(key, "", None, None)
    };
    let r_vocab: Vec<Option<Atom>> = vec![
        None,
        Some(stub("r")),
        Some(variant(
            "r",
            "a",
            Some("transitively-verified"),
            Some("translation"),
        )),
        Some(variant("r", "b", Some("verified"), Some("translation"))),
        Some(variant("r", "c", Some("trusted"), Some("translation"))),
        Some(variant("r", "d", Some("verified"), None)),
    ];
    let s_vocab: Vec<Option<Atom>> = vec![
        None,
        Some(stub("s")),
        Some(variant("s", "e", Some("unverified"), None)),
        Some(variant("s", "f", Some("trusted"), None)),
        Some(variant("s", "g", Some("transitively-verified"), None)),
    ];
    let c_vocab: Vec<Option<Atom>> = vec![None, Some(variant("c", "h", Some("verified"), None))];
    let mut maps = Vec::new();
    for r in &r_vocab {
        for s in &s_vocab {
            for c in &c_vocab {
                let mut m = BTreeMap::new();
                for (key, value) in [("r", r), ("s", s), ("c", c)] {
                    if let Some(a) = value.clone() {
                        m.insert(key.to_owned(), a);
                    }
                }
                maps.push(m);
            }
        }
    }
    let as_json = |m: &BTreeMap<String, Atom>| serde_json::to_value(m).unwrap();
    for a in &maps {
        let once = merge(vec![a.clone()], false, None);
        assert_eq!(
            as_json(&merge(vec![once.clone()], false, None)),
            as_json(&once),
            "enrichment fixed point"
        );
        for b in &maps {
            let ab = merge(vec![a.clone(), b.clone()], false, None);
            for c in &maps {
                let left = merge(vec![ab.clone(), c.clone()], false, None);
                let bc = merge(vec![b.clone(), c.clone()], false, None);
                let right = merge(vec![a.clone(), bc], false, None);
                assert_eq!(as_json(&left), as_json(&right), "associativity");
            }
        }
    }
}

/// Independent oracle for one categorized array (P8/P15): normalize every
/// string entry, then collect into a set. Strings and non-strings are kept
/// apart by a tag so `"42"` and `42` stay distinct. Merge must collapse two
/// aliases exactly when their oracle sets are equal.
fn oracle_set(array: &Value) -> BTreeSet<(bool, String)> {
    array
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| match entry.as_str() {
            Some(name) => (false, name.trim_end_matches('.').to_owned()),
            None => (true, entry.to_string()),
        })
        .collect()
}

/// The canonical array the oracle predicts: strings sorted, then non-strings
/// by their JSON text, no duplicates.
fn oracle_canonical(array: &Value) -> Value {
    Value::Array(
        oracle_set(array)
            .into_iter()
            .map(|(non_string, text)| {
                if non_string {
                    serde_json::from_str(&text).unwrap()
                } else {
                    Value::String(text)
                }
            })
            .collect(),
    )
}

/// The categorized-array vocabulary: the same sets spelled with different
/// order, duplicates and trailing dots, plus sets that differ by one entry
/// mixed string/non-string arrays, and a name that JSON escapes.
fn categorized_arrays() -> Vec<Value> {
    vec![
        json!([]),
        json!(["g()"]),
        json!(["g()."]),
        json!(["g()", "g().."]),
        json!(["h()"]),
        json!(["g()..", "h()"]),
        json!(["h()", "g()", "h()."]),
        json!(["g()", 42]),
        json!([42, "g().", 42]),
        json!(["42"]),
        json!([42]),
        // `"` sorts before `#` by value but after it by JSON text (`\"`),
        // so this pins the sort to the string value.
        json!(["k#", "k\"."]),
    ]
}

/// P8/P15 set semantics, checked against the oracle: two atoms under aliased
/// keys in one input, equal except for one categorized array, collapse iff
/// the normalized arrays are equal as sets, and the survivor carries the
/// canonical array; any set difference between two real atoms rejects the
/// input. With a stub on either side the input is never rejected and the
/// real atom survives (P3/P6). Swept over every ordered pair of arrays, all
/// five `CATEGORIZED_DEPENDENCY_ARRAYS` fields, every stub/real combination,
/// and both the raw and the enriched path. `dependencies` stays empty so the
/// categorized field is the only difference between the aliases.
#[test]
fn categorized_arrays_collapse_iff_equal_as_sets() {
    let arrays = categorized_arrays();
    for field in CATEGORIZED_DEPENDENCY_ARRAYS {
        for plain in &arrays {
            for dotted in &arrays {
                for (plain_kind, dotted_kind) in [(1, 1), (0, 1), (1, 0), (0, 0)] {
                    let with = |kind: usize, array: &Value| {
                        let mut value = atom(kind);
                        value.extensions.insert(field.into(), array.clone());
                        value
                    };
                    let input = BTreeMap::from([
                        ("f()".to_owned(), with(plain_kind, plain)),
                        ("f()..".to_owned(), with(dotted_kind, dotted)),
                    ]);
                    let same_set = oracle_set(plain) == oracle_set(dotted);
                    let both_real = plain_kind == 1 && dotted_kind == 1;
                    for raw in [false, true] {
                        let result = if raw {
                            merge_atom_maps_raw(vec![input.clone()], None)
                        } else {
                            merge_atom_maps(vec![input.clone()], None)
                        };
                        let case = format!(
                            "{field}: {plain} (kind {plain_kind}) vs {dotted} \
                             (kind {dotted_kind}), raw {raw}"
                        );
                        if both_real && !same_set {
                            let err = result.expect_err(&case);
                            assert!(err.contains("distinct real atom"), "{case}: {err}");
                            assert!(err.starts_with("input #1 of 1: "), "{case}: {err}");
                            continue;
                        }
                        let (out, _) = result.expect(&case);
                        assert_eq!(out.len(), 1, "{case}");
                        let survivor = &out["f()"];
                        // The real side wins a stub collision; otherwise
                        // the first key in order ("f()") survives.
                        let expected = if plain_kind == 0 && dotted_kind == 1 {
                            dotted
                        } else {
                            plain
                        };
                        assert_eq!(
                            survivor.is_stub(),
                            plain_kind == 0 && dotted_kind == 0,
                            "{case}"
                        );
                        assert_eq!(
                            survivor.extensions[field],
                            oracle_canonical(expected),
                            "{case}"
                        );
                    }
                }
            }
        }
    }
}

/// P4 (associativity, mapping compatibility) and P5 (self-merge idempotence)
/// on atoms that carry categorized arrays, on both paths. The variants are
/// one-key inputs (plain or dotted key) and two-key inputs whose aliases hold
/// the same set spelled differently, so the intra-input collapse runs inside
/// every nesting. Each array is placed on a different categorized field so
/// all five fields take part, and one record shape rides along so the record
/// union and the set comparison interact. 3 kinds x 3 arrays x 2 records,
/// plus 4 alias-pair inputs: 22 variants, 22^3 x 2 associativity cases.
#[test]
fn laws_hold_over_generated_categorized_variants() {
    let spellings = [
        (json!(["g()", "h()"]), json!(["h().", "g()..", "g()"])),
        (json!(["g()", 42]), json!([42, "g().", 42])),
        (json!(["h()"]), json!(["h()...", "h()"])),
    ];
    let records = [None, Some(json!([{"target":"f.", "confidence":"exact"}]))];
    let with = |kind: usize, field: &str, array: &Value, record: &Option<Value>| {
        let mut value = atom(kind);
        value.extensions.insert(field.into(), array.clone());
        if let Some(record) = record {
            value.extensions.insert("maps-to".into(), record.clone());
        }
        value
    };
    let mut variants = Vec::new();
    for kind in 0..3 {
        for (index, (array, _)) in spellings.iter().enumerate() {
            for record in &records {
                let field = CATEGORIZED_DEPENDENCY_ARRAYS[index];
                let key = if record.is_some() { "f.." } else { "f" };
                variants.push(single(with(kind, field, array, record), key));
            }
        }
    }
    for (index, (array, respelled)) in spellings.iter().enumerate() {
        let field = CATEGORIZED_DEPENDENCY_ARRAYS[(index + 3) % 5];
        variants.push(BTreeMap::from([
            ("f".to_owned(), with(1, field, array, &None)),
            ("f..".to_owned(), with(1, field, respelled, &records[1])),
        ]));
    }
    // A stub alias next to a real one also collapses without error.
    variants.push(BTreeMap::from([
        (
            "f".to_owned(),
            with(0, "type-dependencies", &json!(["k()"]), &None),
        ),
        (
            "f.".to_owned(),
            with(2, "term-dependencies", &json!(["g()."]), &None),
        ),
    ]));
    assert_eq!(variants.len(), 22);
    let mappings = vec![Mapping {
        from: "f..".into(),
        to: "f.".into(),
        confidence: "exact".into(),
        method: None,
    }];
    for raw in [false, true] {
        for first in &variants {
            let canonical = merge(vec![first.clone()], raw, None);
            // Mapping re-application is a no-op: F_M(F_M(A)) = F_M(A).
            let mapped = merge(vec![first.clone()], raw, Some(&mappings));
            assert_eq!(merge(vec![mapped.clone()], raw, Some(&mappings)), mapped);
            assert_eq!(
                merge(vec![first.clone(), first.clone()], raw, None),
                canonical
            );
            assert_eq!(merge(vec![canonical.clone()], raw, None), canonical);
            for second in &variants {
                // Commutativity on disjoint keys: renaming every key of
                // `second` makes the inputs disjoint, so order cannot matter.
                let disjoint = rekey(second, "g");
                assert_eq!(
                    merge(vec![first.clone(), disjoint.clone()], raw, None),
                    merge(vec![disjoint, first.clone()], raw, None)
                );
                let flat = merge(vec![first.clone(), second.clone()], raw, Some(&mappings));
                let mapped_first = merge(vec![first.clone()], raw, Some(&mappings));
                let mapped_second = merge(vec![second.clone()], raw, Some(&mappings));
                assert_eq!(flat, merge(vec![mapped_first, mapped_second], raw, None));
                for third in &variants {
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
