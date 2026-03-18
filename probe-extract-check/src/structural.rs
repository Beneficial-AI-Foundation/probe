//! Structural invariant checks on extract JSON.
//!
//! These checks validate the JSON in isolation — no source files needed.

use probe::types::{Atom, AtomEnvelope};
use std::collections::BTreeSet;

/// A single diagnostic from a check.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub atom_key: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.level {
            Level::Error => "ERROR",
            Level::Warning => "WARN",
        };
        if let Some(key) = &self.atom_key {
            write!(f, "[{prefix}] {key}: {}", self.message)
        } else {
            write!(f, "[{prefix}] {}", self.message)
        }
    }
}

/// Run all structural checks on a parsed envelope.
pub fn check_structural(envelope: &AtomEnvelope) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    check_envelope_fields(envelope, &mut diags);
    check_line_ranges(&envelope.data, &mut diags);
    check_referential_integrity(&envelope.data, &mut diags);

    diags
}

/// Verify that required envelope fields are populated.
fn check_envelope_fields(envelope: &AtomEnvelope, diags: &mut Vec<Diagnostic>) {
    if envelope.schema.is_empty() {
        diags.push(Diagnostic {
            level: Level::Error,
            atom_key: None,
            message: "envelope 'schema' field is empty".into(),
        });
    }
    if envelope.schema_version.is_empty() {
        diags.push(Diagnostic {
            level: Level::Error,
            atom_key: None,
            message: "envelope 'schema-version' field is empty".into(),
        });
    }
    if envelope.tool.name.is_empty() {
        diags.push(Diagnostic {
            level: Level::Error,
            atom_key: None,
            message: "envelope 'tool.name' field is empty".into(),
        });
    }
    if envelope.timestamp.is_empty() {
        diags.push(Diagnostic {
            level: Level::Error,
            atom_key: None,
            message: "envelope 'timestamp' field is empty".into(),
        });
    }
    if envelope.data.is_empty() {
        diags.push(Diagnostic {
            level: Level::Warning,
            atom_key: None,
            message: "envelope 'data' is empty (no atoms)".into(),
        });
    }
}

/// Verify that all atoms have valid line ranges.
fn check_line_ranges(
    data: &std::collections::BTreeMap<String, Atom>,
    diags: &mut Vec<Diagnostic>,
) {
    for (key, atom) in data {
        // Stubs have 0/0 ranges — skip them.
        if atom.is_stub() {
            continue;
        }

        let start = atom.code_text.lines_start;
        let end = atom.code_text.lines_end;

        if start == 0 || end == 0 {
            diags.push(Diagnostic {
                level: Level::Error,
                atom_key: Some(key.clone()),
                message: format!("line range has zero value: start={start}, end={end}"),
            });
        } else if start > end {
            diags.push(Diagnostic {
                level: Level::Error,
                atom_key: Some(key.clone()),
                message: format!("lines-start ({start}) > lines-end ({end})"),
            });
        }
    }
}

/// Verify that all dependency targets exist in the data (or are stubs).
fn check_referential_integrity(
    data: &std::collections::BTreeMap<String, Atom>,
    diags: &mut Vec<Diagnostic>,
) {
    let known_keys: BTreeSet<&str> = data.keys().map(|k| k.as_str()).collect();

    for (key, atom) in data {
        for dep in &atom.dependencies {
            if !known_keys.contains(dep.as_str()) {
                diags.push(Diagnostic {
                    level: Level::Warning,
                    atom_key: Some(key.clone()),
                    message: format!("dependency target not found in data: {dep}"),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use probe::types::{CodeText, Source, Tool};
    use std::collections::{BTreeMap, BTreeSet};

    fn make_envelope(data: BTreeMap<String, Atom>) -> AtomEnvelope {
        AtomEnvelope {
            schema: "probe-rust/extract".into(),
            schema_version: "2.0".into(),
            tool: Tool {
                name: "probe-rust".into(),
                version: "1.0.0".into(),
                command: "extract".into(),
            },
            source: Source {
                repo: "test".into(),
                commit: "abc".into(),
                language: "rust".into(),
                package: "test".into(),
                package_version: "0.1.0".into(),
            },
            timestamp: "2026-01-01T00:00:00Z".into(),
            data,
        }
    }

    fn make_atom(name: &str, deps: &[&str], start: usize, end: usize) -> Atom {
        Atom {
            display_name: name.into(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            code_module: "mod".into(),
            code_path: "src/lib.rs".into(),
            code_text: CodeText {
                lines_start: start,
                lines_end: end,
            },
            kind: "exec".into(),
            language: "rust".into(),
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn test_valid_envelope_no_errors() {
        let mut data = BTreeMap::new();
        data.insert("probe:t/1/m/a()".into(), make_atom("a", &["probe:t/1/m/b()"], 1, 10));
        data.insert("probe:t/1/m/b()".into(), make_atom("b", &[], 12, 20));
        let envelope = make_envelope(data);
        let diags = check_structural(&envelope);
        let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_inverted_line_range() {
        let mut data = BTreeMap::new();
        data.insert("probe:t/1/m/a()".into(), make_atom("a", &[], 20, 10));
        let envelope = make_envelope(data);
        let diags = check_structural(&envelope);
        assert!(diags.iter().any(|d| d.message.contains("lines-start (20) > lines-end (10)")));
    }

    #[test]
    fn test_dangling_dependency() {
        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/a()".into(),
            make_atom("a", &["probe:t/1/m/missing()"], 1, 10),
        );
        let envelope = make_envelope(data);
        let diags = check_structural(&envelope);
        assert!(diags.iter().any(|d| d.message.contains("not found in data")));
    }

    #[test]
    fn test_stubs_skip_line_range_check() {
        let mut data = BTreeMap::new();
        data.insert(
            "probe:ext/1/lib/ext()".into(),
            Atom {
                display_name: "ext".into(),
                dependencies: BTreeSet::new(),
                code_module: "lib".into(),
                code_path: "".into(),
                code_text: CodeText {
                    lines_start: 0,
                    lines_end: 0,
                },
                kind: "exec".into(),
                language: "rust".into(),
                extensions: BTreeMap::new(),
            },
        );
        let envelope = make_envelope(data);
        let diags = check_structural(&envelope);
        let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
        assert!(errors.is_empty());
    }
}
