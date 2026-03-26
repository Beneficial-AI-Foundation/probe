//! Property-based checks (Layer 3): checks that can run against any extract
//! output without golden files.
//!
//! - Completeness: count declarations in source vs atoms
//! - Location overlap: no two non-stub atoms share identical spans
//! - Module consistency: code-path and code-module are consistent

use crate::structural::{Diagnostic, Level};
use probe::types::Atom;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

/// Run all property checks.
pub fn check_properties(
    data: &BTreeMap<String, Atom>,
    project_path: Option<&Path>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    check_location_overlap(data, &mut diags);
    check_array_ordering(data, &mut diags);

    if let Some(path) = project_path {
        check_completeness(data, path, &mut diags);
    }

    diags
}

// ---------------------------------------------------------------------------
// Location overlap
// ---------------------------------------------------------------------------

/// No two non-stub atoms should have identical (code-path, lines-start, lines-end).
fn check_location_overlap(data: &BTreeMap<String, Atom>, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<(String, usize, usize), Vec<String>> = HashMap::new();

    for (key, atom) in data {
        if atom.is_stub() {
            continue;
        }
        let loc = (
            atom.code_path.clone(),
            atom.code_text.lines_start,
            atom.code_text.lines_end,
        );
        seen.entry(loc).or_default().push(key.clone());
    }

    for ((path, start, end), keys) in &seen {
        if keys.len() > 1 {
            diags.push(Diagnostic {
                level: Level::Warning,
                atom_key: None,
                message: format!(
                    "overlapping location {path}:{start}-{end} shared by {} atoms: {}",
                    keys.len(),
                    keys.join(", ")
                ),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Array ordering (P14)
// ---------------------------------------------------------------------------

/// `dependencies-with-locations` must be sorted by (line, code-name) per P14.
fn check_array_ordering(data: &BTreeMap<String, Atom>, diags: &mut Vec<Diagnostic>) {
    for (key, atom) in data {
        if let Some(dwl) = atom.extensions.get("dependencies-with-locations") {
            if let Some(arr) = dwl.as_array() {
                let entries: Vec<(u64, &str)> = arr
                    .iter()
                    .filter_map(|entry| {
                        let line = entry.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                        let cn = entry
                            .get("code-name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        Some((line, cn))
                    })
                    .collect();

                for window in entries.windows(2) {
                    let (l1, cn1) = window[0];
                    let (l2, cn2) = window[1];
                    if (l1, cn1) > (l2, cn2) {
                        diags.push(Diagnostic {
                            level: Level::Warning,
                            atom_key: Some(key.clone()),
                            message: format!(
                                "dependencies-with-locations not sorted by (line, code-name): \
                                 ({l1}, {cn1}) > ({l2}, {cn2}) — violates P14 (deterministic output)"
                            ),
                        });
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Completeness
// ---------------------------------------------------------------------------

/// Declaration patterns we look for in source files to count expected atoms.
struct LangCounter {
    /// Regex that matches one declaration per match.
    decl_re: Regex,
    /// File extension.
    extension: &'static str,
}

fn rust_counter() -> LangCounter {
    // Match fn declarations (including pub, async, proof, spec, exec, const, unsafe)
    LangCounter {
        decl_re: Regex::new(
            r"(?m)^\s*(?:pub(?:\(crate\))?\s+)?(?:const\s+|async\s+|unsafe\s+)?(?:exec\s+|proof\s+|spec\s+)?fn\s+\w+"
        ).unwrap(),
        extension: "rs",
    }
}

fn lean_counter() -> LangCounter {
    // Match def, theorem, structure, inductive, class, instance, abbrev, axiom, opaque
    LangCounter {
        decl_re: Regex::new(
            r"(?m)^\s*(?:private\s+|protected\s+|noncomputable\s+|partial\s+|unsafe\s+)*(?:def|theorem|structure|inductive|class|instance|abbrev|axiom|opaque)\s+\w+"
        ).unwrap(),
        extension: "lean",
    }
}

/// Count declarations in source files and compare to atom count.
///
/// This is an approximate check — it may over-count (e.g., commented-out code,
/// test functions) or under-count (e.g., macro-generated functions). Large
/// discrepancies indicate potential issues.
fn check_completeness(
    data: &BTreeMap<String, Atom>,
    project_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    // Determine language from atoms
    let languages: BTreeSet<&str> = data.values().map(|a| a.language.as_str()).collect();

    for lang in &languages {
        let counter = match *lang {
            "rust" => rust_counter(),
            "lean" => lean_counter(),
            _ => continue,
        };

        let source_count = count_declarations_in_project(project_path, &counter);
        let atom_count = data
            .values()
            .filter(|a| a.language == *lang && !a.is_stub())
            .count();

        if source_count == 0 && atom_count == 0 {
            continue;
        }

        // Allow some tolerance — SCIP/tree-sitter may find slightly different counts
        let ratio = if source_count > 0 {
            atom_count as f64 / source_count as f64
        } else {
            0.0
        };

        if ratio < 0.5 {
            diags.push(Diagnostic {
                level: Level::Warning,
                atom_key: None,
                message: format!(
                    "completeness: found {atom_count} {lang} atoms but ~{source_count} \
                     declarations in source (ratio {ratio:.2}) — significant gap may indicate \
                     missing atoms"
                ),
            });
        } else if ratio > 2.0 {
            diags.push(Diagnostic {
                level: Level::Warning,
                atom_key: None,
                message: format!(
                    "completeness: found {atom_count} {lang} atoms but only ~{source_count} \
                     declarations in source (ratio {ratio:.2}) — may indicate phantom atoms"
                ),
            });
        }
    }
}

/// Walk project source files and count declaration matches.
fn count_declarations_in_project(project_path: &Path, counter: &LangCounter) -> usize {
    let mut total = 0;
    walk_source_files(project_path, counter.extension, &mut |content| {
        total += counter.decl_re.find_iter(content).count();
    });
    total
}

/// Walk directory tree for files with the given extension, calling `f` with file contents.
fn walk_source_files(dir: &Path, extension: &str, f: &mut dyn FnMut(&str)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let ft = entry.file_type();
        if ft.is_ok_and(|t| t.is_symlink()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" && name != ".lake" {
                walk_source_files(&path, extension, f);
            }
        } else if path.extension().is_some_and(|e| e == extension) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                f(&content);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use probe::types::CodeText;
    use serde_json::json;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    fn make_atom(name: &str, path: &str, start: usize, end: usize, lang: &str) -> Atom {
        Atom {
            display_name: name.into(),
            dependencies: BTreeSet::new(),
            code_module: "mod".into(),
            code_path: path.into(),
            code_text: CodeText {
                lines_start: start,
                lines_end: end,
            },
            kind: "exec".into(),
            language: lang.into(),
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn test_no_overlap() {
        let mut data = BTreeMap::new();
        data.insert("a".into(), make_atom("a", "src/lib.rs", 1, 5, "rust"));
        data.insert("b".into(), make_atom("b", "src/lib.rs", 6, 10, "rust"));
        let mut diags = Vec::new();
        check_location_overlap(&data, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_overlap_detected() {
        let mut data = BTreeMap::new();
        data.insert("a".into(), make_atom("a", "src/lib.rs", 1, 5, "rust"));
        data.insert("b".into(), make_atom("b", "src/lib.rs", 1, 5, "rust"));
        let mut diags = Vec::new();
        check_location_overlap(&data, &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("overlapping location"));
    }

    #[test]
    fn test_completeness_good_ratio() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn foo() {}\nfn bar() {}\npub fn baz() {}\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert("a".into(), make_atom("foo", "src/lib.rs", 1, 1, "rust"));
        data.insert("b".into(), make_atom("bar", "src/lib.rs", 2, 2, "rust"));
        data.insert("c".into(), make_atom("baz", "src/lib.rs", 3, 3, "rust"));

        let mut diags = Vec::new();
        check_completeness(&data, tmp.path(), &mut diags);
        assert!(
            diags.is_empty(),
            "good ratio should produce no warnings: {diags:?}"
        );
    }

    #[test]
    fn test_completeness_low_ratio() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\nfn e() {}\nfn f() {}\nfn g() {}\nfn h() {}\nfn i() {}\nfn j() {}\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        // Only 2 atoms for 10 functions
        data.insert("a".into(), make_atom("a", "src/lib.rs", 1, 1, "rust"));
        data.insert("b".into(), make_atom("b", "src/lib.rs", 2, 2, "rust"));

        let mut diags = Vec::new();
        check_completeness(&data, tmp.path(), &mut diags);
        assert!(diags.iter().any(|d| d.message.contains("completeness")));
    }

    #[test]
    fn test_lean_completeness() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Test.lean"),
            "def foo : Nat := 0\ntheorem bar : 1 = 1 := rfl\nstructure Baz where\n  x : Nat\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert("a".into(), make_atom("foo", "Test.lean", 1, 1, "lean"));
        data.insert("b".into(), make_atom("bar", "Test.lean", 2, 2, "lean"));
        data.insert("c".into(), make_atom("Baz", "Test.lean", 3, 4, "lean"));

        let mut diags = Vec::new();
        check_completeness(&data, tmp.path(), &mut diags);
        assert!(
            diags.is_empty(),
            "good lean ratio should produce no warnings: {diags:?}"
        );
    }

    fn make_atom_with_dwl(name: &str, dwl: serde_json::Value) -> Atom {
        let mut extensions = BTreeMap::new();
        extensions.insert("dependencies-with-locations".to_string(), dwl);
        Atom {
            display_name: name.into(),
            dependencies: BTreeSet::new(),
            code_module: "mod".into(),
            code_path: "src/lib.rs".into(),
            code_text: CodeText {
                lines_start: 1,
                lines_end: 10,
            },
            kind: "exec".into(),
            language: "rust".into(),
            extensions,
        }
    }

    #[test]
    fn test_sorted_dwl_passes() {
        let atom = make_atom_with_dwl(
            "foo",
            json!([
                {"code-name": "a", "line": 5, "location": "inner"},
                {"code-name": "a", "line": 10, "location": "inner"},
                {"code-name": "b", "line": 10, "location": "inner"},
            ]),
        );
        let mut data = BTreeMap::new();
        data.insert("probe:test/foo()".into(), atom);
        let mut diags = Vec::new();
        check_array_ordering(&data, &mut diags);
        assert!(diags.is_empty(), "sorted DWL should pass: {diags:?}");
    }

    #[test]
    fn test_unsorted_dwl_warns() {
        let atom = make_atom_with_dwl(
            "foo",
            json!([
                {"code-name": "b", "line": 10, "location": "inner"},
                {"code-name": "a", "line": 5, "location": "inner"},
            ]),
        );
        let mut data = BTreeMap::new();
        data.insert("probe:test/foo()".into(), atom);
        let mut diags = Vec::new();
        check_array_ordering(&data, &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("not sorted"));
    }

    #[test]
    fn test_no_dwl_passes() {
        let mut data = BTreeMap::new();
        data.insert("a".into(), make_atom("foo", "src/lib.rs", 1, 5, "rust"));
        let mut diags = Vec::new();
        check_array_ordering(&data, &mut diags);
        assert!(diags.is_empty());
    }
}
