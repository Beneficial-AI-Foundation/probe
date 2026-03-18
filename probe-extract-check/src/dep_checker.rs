//! Dependency checks: verify that dependency relationships match the source.
//!
//! For each atom's dependencies, checks that the callee's display-name appears
//! as an identifier within the caller's source span.

use crate::structural::{Diagnostic, Level};
use probe::types::Atom;
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;

/// Run dependency checks for all non-stub atoms.
pub fn check_deps(data: &BTreeMap<String, Atom>, project_path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    // Cache file contents to avoid re-reading
    let mut file_cache: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (key, atom) in data {
        if atom.is_stub() || atom.dependencies.is_empty() {
            continue;
        }

        let lines = match get_file_lines(&atom.code_path, project_path, &mut file_cache) {
            Some(l) => l,
            None => continue, // source_checker already flags missing files
        };

        let start = atom.code_text.lines_start;
        let end = atom.code_text.lines_end;

        if start == 0 || end == 0 || end > lines.len() {
            continue; // Already flagged by other checkers
        }

        let span_text: String = lines[start - 1..end].join("\n");

        for dep_key in &atom.dependencies {
            // Look up the target atom to get its display-name
            let dep_name = match data.get(dep_key) {
                Some(dep_atom) => &dep_atom.display_name,
                None => continue, // Dangling ref already caught by structural check
            };

            check_dep_in_span(key, dep_key, dep_name, &span_text, atom, &mut diags);
        }

        // Check dependencies_with_locations if present
        check_located_deps(key, atom, lines, data, &mut diags);
    }

    diags
}

/// Check that a dependency's display-name appears in the caller's span.
fn check_dep_in_span(
    caller_key: &str,
    dep_key: &str,
    dep_name: &str,
    span_text: &str,
    caller: &Atom,
    diags: &mut Vec<Diagnostic>,
) {
    // Skip self-references (recursive calls)
    if caller.display_name == dep_name {
        return;
    }

    let pattern = format!(r"\b{}\b", regex::escape(dep_name));
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return,
    };

    if !re.is_match(span_text) {
        diags.push(Diagnostic {
            level: Level::Warning,
            atom_key: Some(caller_key.into()),
            message: format!(
                "dependency '{}' (display-name '{}') not found as identifier in source span",
                dep_key, dep_name
            ),
        });
    }
}

/// Check dependencies_with_locations: verify line numbers and name presence.
fn check_located_deps(
    caller_key: &str,
    atom: &Atom,
    file_lines: &[String],
    data: &BTreeMap<String, Atom>,
    diags: &mut Vec<Diagnostic>,
) {
    // dependencies-with-locations is in extensions as a JSON array
    let deps_with_loc = match atom.extensions.get("dependencies-with-locations") {
        Some(v) => v,
        None => return,
    };

    let entries = match deps_with_loc.as_array() {
        Some(a) => a,
        None => return,
    };

    let start = atom.code_text.lines_start;
    let end = atom.code_text.lines_end;

    for entry in entries {
        let line = match entry.get("line").and_then(|v| v.as_u64()) {
            Some(l) => l as usize,
            None => continue,
        };

        let code_name = match entry.get("code-name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };

        // Check line is within the caller's span
        if line < start || line > end {
            diags.push(Diagnostic {
                level: Level::Error,
                atom_key: Some(caller_key.into()),
                message: format!(
                    "dependency-with-location for '{}' at line {line} is outside span [{start}, {end}]",
                    code_name
                ),
            });
            continue;
        }

        // Check the target's display-name appears on that specific line
        if let Some(dep_atom) = data.get(code_name) {
            if line > 0 && line <= file_lines.len() {
                let line_text = &file_lines[line - 1];
                let pattern = format!(r"\b{}\b", regex::escape(&dep_atom.display_name));
                if let Ok(re) = Regex::new(&pattern) {
                    if !re.is_match(line_text) {
                        diags.push(Diagnostic {
                            level: Level::Warning,
                            atom_key: Some(caller_key.into()),
                            message: format!(
                                "dependency-with-location '{}' (display-name '{}') not found on line {line}",
                                code_name, dep_atom.display_name
                            ),
                        });
                    }
                }
            }
        }
    }
}

fn get_file_lines<'a>(
    code_path: &str,
    project_path: &Path,
    cache: &'a mut BTreeMap<String, Vec<String>>,
) -> Option<&'a Vec<String>> {
    if !cache.contains_key(code_path) {
        let full_path = project_path.join(code_path);
        let content = std::fs::read_to_string(&full_path).ok()?;
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        cache.insert(code_path.to_string(), lines);
    }
    cache.get(code_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use probe::types::CodeText;
    use tempfile::TempDir;

    fn make_atom(name: &str, path: &str, start: usize, end: usize, deps: &[&str]) -> Atom {
        Atom {
            display_name: name.into(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            code_module: "mod".into(),
            code_path: path.into(),
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
    fn test_dep_found_in_span() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn foo() {\n    bar();\n    baz();\n}\nfn bar() {}\nfn baz() {}\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/foo()".into(),
            make_atom(
                "foo",
                "src/lib.rs",
                1,
                4,
                &["probe:t/1/m/bar()", "probe:t/1/m/baz()"],
            ),
        );
        data.insert(
            "probe:t/1/m/bar()".into(),
            make_atom("bar", "src/lib.rs", 5, 5, &[]),
        );
        data.insert(
            "probe:t/1/m/baz()".into(),
            make_atom("baz", "src/lib.rs", 6, 6, &[]),
        );

        let diags = check_deps(&data, tmp.path());
        assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");
    }

    #[test]
    fn test_dep_not_found_in_span() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn foo() {\n    println!(\"hello\");\n}\nfn bar() {}\n",
        )
        .unwrap();

        let mut data = BTreeMap::new();
        data.insert(
            "probe:t/1/m/foo()".into(),
            make_atom("foo", "src/lib.rs", 1, 3, &["probe:t/1/m/bar()"]),
        );
        data.insert(
            "probe:t/1/m/bar()".into(),
            make_atom("bar", "src/lib.rs", 4, 4, &[]),
        );

        let diags = check_deps(&data, tmp.path());
        assert!(diags
            .iter()
            .any(|d| d.message.contains("not found as identifier")));
    }
}
