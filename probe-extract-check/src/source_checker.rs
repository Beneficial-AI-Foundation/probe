//! Source-grounded checks: verify atoms against actual source files.
//!
//! For each non-stub atom, reads the source file and verifies:
//! - The file at `code-path` exists
//! - The line range is within the file's bounds
//! - A declaration matching `display-name` exists within the line range
//! - The `kind` matches the declaration keyword found at that location

use crate::structural::{Diagnostic, Level};
use probe::types::Atom;
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;

/// Language-specific declaration patterns.
struct DeclPatterns {
    /// Maps kind values to regex patterns that match declarations of that kind.
    kind_patterns: Vec<(String, Regex)>,
    /// Generic pattern to find any function/def declaration by name.
    name_pattern_template: String,
}

fn rust_patterns() -> DeclPatterns {
    DeclPatterns {
        kind_patterns: vec![
            ("exec".into(), Regex::new(r"\b(pub\s+)?(async\s+)?fn\b").unwrap()),
            ("proof".into(), Regex::new(r"\bproof\s+fn\b").unwrap()),
            ("spec".into(), Regex::new(r"\bspec\s+fn\b").unwrap()),
        ],
        name_pattern_template: r"\b{NAME}\b".into(),
    }
}

fn lean_patterns() -> DeclPatterns {
    DeclPatterns {
        kind_patterns: vec![
            ("def".into(), Regex::new(r"\bdef\b").unwrap()),
            ("theorem".into(), Regex::new(r"\btheorem\b").unwrap()),
            ("abbrev".into(), Regex::new(r"\babbrev\b").unwrap()),
            ("class".into(), Regex::new(r"\bclass\b").unwrap()),
            ("structure".into(), Regex::new(r"\bstructure\b").unwrap()),
            ("inductive".into(), Regex::new(r"\binductive\b").unwrap()),
            ("instance".into(), Regex::new(r"\binstance\b").unwrap()),
            ("axiom".into(), Regex::new(r"\baxiom\b").unwrap()),
            ("opaque".into(), Regex::new(r"\bopaque\b").unwrap()),
        ],
        name_pattern_template: r"\b{NAME}\b".into(),
    }
}

fn patterns_for_language(lang: &str) -> DeclPatterns {
    match lang {
        "lean" => lean_patterns(),
        // Rust is default — covers both probe-rust and probe-verus
        _ => rust_patterns(),
    }
}

/// Run source-grounded checks for all non-stub atoms.
pub fn check_source(
    data: &BTreeMap<String, Atom>,
    project_path: &Path,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (key, atom) in data {
        if atom.is_stub() {
            continue;
        }

        check_atom_source(key, atom, project_path, &mut diags);
    }

    diags
}

fn check_atom_source(
    key: &str,
    atom: &Atom,
    project_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let file_path = project_path.join(&atom.code_path);

    // 1. File exists
    if !file_path.is_file() {
        diags.push(Diagnostic {
            level: Level::Error,
            atom_key: Some(key.into()),
            message: format!("code-path not found: {}", atom.code_path),
        });
        return;
    }

    // 2. Read file and check line bounds
    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            diags.push(Diagnostic {
                level: Level::Error,
                atom_key: Some(key.into()),
                message: format!("failed to read {}: {e}", file_path.display()),
            });
            return;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let start = atom.code_text.lines_start;
    let end = atom.code_text.lines_end;

    if start == 0 || end == 0 {
        // Already caught by structural checks
        return;
    }

    if end > total_lines {
        diags.push(Diagnostic {
            level: Level::Error,
            atom_key: Some(key.into()),
            message: format!(
                "lines-end ({end}) exceeds file length ({total_lines} lines) in {}",
                atom.code_path
            ),
        });
        return;
    }

    // Extract the span text (1-based to 0-based)
    let span_text: String = lines[start - 1..end].join("\n");

    // 3. Check display-name appears in span
    let name = &atom.display_name;
    let patterns = patterns_for_language(&atom.language);
    let name_re_str = patterns.name_pattern_template.replace("{NAME}", &regex::escape(name));
    let name_re = match Regex::new(&name_re_str) {
        Ok(r) => r,
        Err(_) => {
            diags.push(Diagnostic {
                level: Level::Warning,
                atom_key: Some(key.into()),
                message: format!("could not build regex for display-name '{name}'"),
            });
            return;
        }
    };

    if !name_re.is_match(&span_text) {
        // Lean instances often have auto-generated names (e.g., instHasSizePoint)
        // that don't appear in source. Skip name check if the span has `instance`.
        let is_auto_named_instance =
            atom.kind == "instance" && span_text.contains("instance");
        if !is_auto_named_instance {
            diags.push(Diagnostic {
                level: Level::Error,
                atom_key: Some(key.into()),
                message: format!(
                    "display-name '{}' not found in {}:{}-{}",
                    name, atom.code_path, start, end
                ),
            });
        }
    }

    // 4. Check kind matches declaration keyword
    check_kind_match(key, atom, &span_text, &patterns, diags);
}

/// Verify the atom's `kind` field matches the declaration keyword in the source span.
fn check_kind_match(
    key: &str,
    atom: &Atom,
    span_text: &str,
    patterns: &DeclPatterns,
    diags: &mut Vec<Diagnostic>,
) {
    let kind = &atom.kind;

    // Find which kind pattern matches the span
    let matched_kind = patterns
        .kind_patterns
        .iter()
        .find(|(_, re)| re.is_match(span_text))
        .map(|(k, _)| k.as_str());

    match matched_kind {
        Some(found_kind) => {
            // For Rust/Verus: "exec" is the generic fn kind, proof/spec are specific.
            // If the atom says "exec" and we find a plain `fn`, that's fine.
            // If the atom says "proof" but we find a plain `fn` (not `proof fn`), that's a mismatch.
            if atom.language == "rust" || atom.language == "verus" {
                // proof fn and spec fn are more specific — check those first.
                // If atom.kind is "proof" or "spec", we must find the corresponding pattern.
                if (kind == "proof" || kind == "spec") && found_kind == "exec" {
                    // The most specific matching pattern was plain `fn` but atom claims proof/spec.
                    // Re-check: does the specific pattern actually match?
                    let specific_match = patterns
                        .kind_patterns
                        .iter()
                        .find(|(k, re)| k == kind && re.is_match(span_text));
                    if specific_match.is_none() {
                        diags.push(Diagnostic {
                            level: Level::Warning,
                            atom_key: Some(key.into()),
                            message: format!(
                                "kind is '{kind}' but source span looks like plain 'fn' (exec)"
                            ),
                        });
                    }
                }
            } else if found_kind != kind {
                diags.push(Diagnostic {
                    level: Level::Warning,
                    atom_key: Some(key.into()),
                    message: format!(
                        "kind is '{kind}' but source span matches '{found_kind}'"
                    ),
                });
            }
        }
        None => {
            // No declaration keyword found at all — might be a macro-generated function
            diags.push(Diagnostic {
                level: Level::Warning,
                atom_key: Some(key.into()),
                message: format!(
                    "no declaration keyword found in source span for kind '{kind}'"
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use probe::types::CodeText;
    use std::collections::{BTreeMap, BTreeSet};
    use tempfile::TempDir;

    fn make_atom(name: &str, path: &str, start: usize, end: usize, kind: &str, lang: &str) -> Atom {
        Atom {
            display_name: name.into(),
            dependencies: BTreeSet::new(),
            code_module: "mod".into(),
            code_path: path.into(),
            code_text: CodeText {
                lines_start: start,
                lines_end: end,
            },
            kind: kind.into(),
            language: lang.into(),
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn test_valid_rust_atom() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("lib.rs"),
            "// preamble\nfn foo() {\n    bar();\n}\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/foo()".into(),
            make_atom("foo", "src/lib.rs", 2, 4, "exec", "rust"),
        );

        let diags = check_source(&data, tmp.path());
        let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_missing_file() {
        let tmp = TempDir::new().unwrap();
        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/foo()".into(),
            make_atom("foo", "src/nonexistent.rs", 1, 5, "exec", "rust"),
        );

        let diags = check_source(&data, tmp.path());
        assert!(diags.iter().any(|d| d.message.contains("code-path not found")));
    }

    #[test]
    fn test_line_range_exceeds_file() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "fn foo() {}\n").unwrap();

        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/foo()".into(),
            make_atom("foo", "src/lib.rs", 1, 100, "exec", "rust"),
        );

        let diags = check_source(&data, tmp.path());
        assert!(diags.iter().any(|d| d.message.contains("exceeds file length")));
    }

    #[test]
    fn test_name_not_in_span() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("lib.rs"),
            "fn bar() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/foo()".into(),
            make_atom("foo", "src/lib.rs", 1, 3, "exec", "rust"),
        );

        let diags = check_source(&data, tmp.path());
        assert!(diags.iter().any(|d| d.message.contains("display-name 'foo' not found")));
    }

    #[test]
    fn test_lean_theorem_kind() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Test.lean"),
            "theorem my_thm : 1 + 1 = 2 := by\n  rfl\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/my_thm".into(),
            make_atom("my_thm", "Test.lean", 1, 2, "theorem", "lean"),
        );

        let diags = check_source(&data, tmp.path());
        let errors: Vec<_> = diags.iter().filter(|d| d.level == Level::Error).collect();
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }
}
