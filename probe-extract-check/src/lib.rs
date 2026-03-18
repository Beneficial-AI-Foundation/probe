pub mod dep_checker;
pub mod golden;
pub mod properties;
pub mod source_checker;
pub mod structural;

use probe::types::AtomEnvelope;
use std::path::Path;
use structural::{Diagnostic, Level};

/// Result of running all checks.
pub struct CheckReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.level == Level::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.level == Level::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.errors().count()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }

    pub fn is_ok(&self) -> bool {
        self.error_count() == 0
    }

    pub fn print_summary(&self) {
        for d in &self.diagnostics {
            eprintln!("{d}");
        }
        eprintln!(
            "\n{} error(s), {} warning(s)",
            self.error_count(),
            self.warning_count()
        );
    }
}

/// Run all checks: structural, source-grounded, dependency, and properties.
///
/// `project_path` is optional — if `None`, only structural checks and
/// data-only property checks run.
pub fn check_all(envelope: &AtomEnvelope, project_path: Option<&Path>) -> CheckReport {
    let mut diagnostics = structural::check_structural(envelope);
    diagnostics.extend(properties::check_properties(&envelope.data, project_path));

    if let Some(path) = project_path {
        diagnostics.extend(source_checker::check_source(&envelope.data, path));
        diagnostics.extend(dep_checker::check_deps(&envelope.data, path));
    }

    CheckReport { diagnostics }
}

/// Load an extract JSON file and parse it as an AtomEnvelope.
pub fn load_extract_json(path: &Path) -> Result<AtomEnvelope, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))
}
