//! Golden file comparison: structural JSON diff ignoring volatile fields.
//!
//! Compares two extract JSON files structurally, ignoring fields that change
//! between runs (timestamp, commit hash, tool version, repo URL).

use serde_json::Value;
use std::collections::BTreeSet;

/// Fields in the envelope that are expected to differ between runs.
const VOLATILE_ENVELOPE_FIELDS: &[&str] = &["timestamp", "commit", "repo", "version"];

/// A single difference found between expected and actual JSON.
#[derive(Debug, Clone)]
pub struct Diff {
    pub path: String,
    pub kind: DiffKind,
}

#[derive(Debug, Clone)]
pub enum DiffKind {
    Missing,
    Extra,
    TypeMismatch { expected: String, actual: String },
    ValueMismatch { expected: String, actual: String },
}

impl std::fmt::Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DiffKind::Missing => write!(f, "MISSING {}", self.path),
            DiffKind::Extra => write!(f, "EXTRA   {}", self.path),
            DiffKind::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "TYPE    {} expected={expected}, actual={actual}",
                    self.path
                )
            }
            DiffKind::ValueMismatch { expected, actual } => {
                write!(
                    f,
                    "VALUE   {} expected={expected}, actual={actual}",
                    self.path
                )
            }
        }
    }
}

/// Compare two extract JSON values structurally.
///
/// Returns an empty vec if they match (ignoring volatile fields).
pub fn compare(expected: &Value, actual: &Value) -> Vec<Diff> {
    let mut diffs = Vec::new();
    compare_values(expected, actual, "$", &mut diffs);
    diffs
}

fn is_volatile_field(path: &str) -> bool {
    // Match paths like $.timestamp, $.tool.version, $.source.commit, $.source.repo
    for field in VOLATILE_ENVELOPE_FIELDS {
        if path.ends_with(&format!(".{field}")) {
            // Only skip if it's a top-level or source/tool field, not inside data
            if !path.contains(".data.") {
                return true;
            }
        }
    }
    false
}

fn compare_values(expected: &Value, actual: &Value, path: &str, diffs: &mut Vec<Diff>) {
    if is_volatile_field(path) {
        return;
    }

    match (expected, actual) {
        (Value::Object(exp_map), Value::Object(act_map)) => {
            let exp_keys: BTreeSet<&String> = exp_map.keys().collect();
            let act_keys: BTreeSet<&String> = act_map.keys().collect();

            for key in &exp_keys {
                let child_path = format!("{path}.{key}");
                if !act_keys.contains(key) {
                    if !is_volatile_field(&child_path) {
                        diffs.push(Diff {
                            path: child_path,
                            kind: DiffKind::Missing,
                        });
                    }
                } else {
                    compare_values(
                        &exp_map[key.as_str()],
                        &act_map[key.as_str()],
                        &child_path,
                        diffs,
                    );
                }
            }

            for key in &act_keys {
                if !exp_keys.contains(key) {
                    let child_path = format!("{path}.{key}");
                    if !is_volatile_field(&child_path) {
                        diffs.push(Diff {
                            path: child_path,
                            kind: DiffKind::Extra,
                        });
                    }
                }
            }
        }
        (Value::Array(exp_arr), Value::Array(act_arr)) => {
            let max_len = exp_arr.len().max(act_arr.len());
            for i in 0..max_len {
                let child_path = format!("{path}[{i}]");
                match (exp_arr.get(i), act_arr.get(i)) {
                    (Some(e), Some(a)) => compare_values(e, a, &child_path, diffs),
                    (Some(_), None) => diffs.push(Diff {
                        path: child_path,
                        kind: DiffKind::Missing,
                    }),
                    (None, Some(_)) => diffs.push(Diff {
                        path: child_path,
                        kind: DiffKind::Extra,
                    }),
                    (None, None) => unreachable!(),
                }
            }
        }
        _ => {
            if std::mem::discriminant(expected) != std::mem::discriminant(actual) {
                diffs.push(Diff {
                    path: path.into(),
                    kind: DiffKind::TypeMismatch {
                        expected: type_label(expected),
                        actual: type_label(actual),
                    },
                });
            } else if expected != actual {
                diffs.push(Diff {
                    path: path.into(),
                    kind: DiffKind::ValueMismatch {
                        expected: expected.to_string(),
                        actual: actual.to_string(),
                    },
                });
            }
        }
    }
}

fn type_label(v: &Value) -> String {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_identical_values() {
        let v = json!({"a": 1, "b": [2, 3]});
        let diffs = compare(&v, &v);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_volatile_fields_ignored() {
        let expected = json!({
            "timestamp": "2026-01-01",
            "tool": { "version": "1.0" },
            "source": { "commit": "abc", "repo": "http://x" },
            "data": {}
        });
        let actual = json!({
            "timestamp": "2026-03-18",
            "tool": { "version": "2.0" },
            "source": { "commit": "def", "repo": "http://y" },
            "data": {}
        });
        let diffs = compare(&expected, &actual);
        assert!(
            diffs.is_empty(),
            "volatile fields should be ignored, got: {diffs:?}"
        );
    }

    #[test]
    fn test_missing_key() {
        let expected = json!({"a": 1, "b": 2});
        let actual = json!({"a": 1});
        let diffs = compare(&expected, &actual);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(diffs[0].kind, DiffKind::Missing));
    }

    #[test]
    fn test_extra_key() {
        let expected = json!({"a": 1});
        let actual = json!({"a": 1, "b": 2});
        let diffs = compare(&expected, &actual);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(diffs[0].kind, DiffKind::Extra));
    }

    #[test]
    fn test_value_mismatch() {
        let expected = json!({"data": {"x": {"kind": "exec"}}});
        let actual = json!({"data": {"x": {"kind": "proof"}}});
        let diffs = compare(&expected, &actual);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(diffs[0].kind, DiffKind::ValueMismatch { .. }));
    }

    #[test]
    fn test_data_timestamp_not_ignored() {
        // A "timestamp" field inside data should NOT be ignored
        let expected = json!({"data": {"atom": {"timestamp": "old"}}});
        let actual = json!({"data": {"atom": {"timestamp": "new"}}});
        let diffs = compare(&expected, &actual);
        assert_eq!(diffs.len(), 1);
    }
}
