use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Schema 2.0 envelope for single-tool atom files.
#[derive(Debug, Serialize, Deserialize)]
pub struct AtomEnvelope {
    pub schema: String,
    #[serde(rename = "schema-version")]
    pub schema_version: String,
    pub tool: Tool,
    pub source: Source,
    pub timestamp: String,
    pub data: BTreeMap<String, Atom>,
}

/// Schema 2.0 envelope for merged atom files.
#[derive(Debug, Serialize, Deserialize)]
pub struct MergedAtomEnvelope {
    pub schema: String,
    #[serde(rename = "schema-version")]
    pub schema_version: String,
    pub tool: Tool,
    pub inputs: Vec<InputProvenance>,
    pub timestamp: String,
    pub data: BTreeMap<String, Atom>,
}

/// Tool metadata in the envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
    pub command: String,
}

/// Source metadata in the envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub repo: String,
    pub commit: String,
    pub language: String,
    pub package: String,
    #[serde(rename = "package-version")]
    pub package_version: String,
}

/// One entry in the merged envelope's `inputs` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputProvenance {
    pub schema: String,
    pub source: Source,
}

/// Line range of an atom's definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeText {
    #[serde(rename = "lines-start")]
    pub lines_start: usize,
    #[serde(rename = "lines-end")]
    pub lines_end: usize,
}

/// A single atom with typed core fields and passthrough extensions.
///
/// The `extensions` field captures any language-specific optional fields
/// (e.g., `dependencies-with-locations`, `is-hidden`, `rust-source`) without
/// the merge tool needing to know about them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    #[serde(rename = "display-name")]
    pub display_name: String,
    pub dependencies: BTreeSet<String>,
    #[serde(rename = "code-module")]
    pub code_module: String,
    #[serde(rename = "code-path")]
    pub code_path: String,
    #[serde(rename = "code-text")]
    pub code_text: CodeText,
    pub kind: String,
    pub language: String,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Atom {
    /// An atom is a stub when it has no source location.
    pub fn is_stub(&self) -> bool {
        self.code_path.is_empty()
            && self.code_text.lines_start == 0
            && self.code_text.lines_end == 0
    }
}

/// Result of loading an atom file: data dictionary, schema string, and optional source.
pub type LoadResult = (BTreeMap<String, Atom>, String, Option<Source>);

/// Load a Schema 2.0 atom file, extracting the data dictionary from the envelope.
///
/// Accepts both single-tool and merged-atoms envelopes. Returns the atom data,
/// the `schema` string, and the `source` object (None for merged files).
pub fn load_atom_file(path: &std::path::Path) -> Result<LoadResult, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let raw: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON in {}: {e}", path.display()))?;

    let schema_version = raw
        .get("schema-version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !schema_version.starts_with("2.") {
        return Err(format!(
            "{}: incompatible schema-version \"{schema_version}\" (expected 2.x)",
            path.display()
        ));
    }

    let schema = raw
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{}: missing \"schema\" field", path.display()))?;

    if !schema.ends_with("/atoms")
        && !schema.ends_with("/enriched-atoms")
        && schema != "probe/merged-atoms"
    {
        return Err(format!(
            "{}: unsupported schema \"{schema}\" (expected */atoms, */enriched-atoms, or probe/merged-atoms)",
            path.display()
        ));
    }

    let data_value = raw
        .get("data")
        .ok_or_else(|| format!("{}: missing \"data\" field", path.display()))?;

    let data: BTreeMap<String, Atom> = serde_json::from_value(data_value.clone())
        .map_err(|e| format!("{}: failed to deserialize atoms: {e}", path.display()))?;

    let source = raw
        .get("source")
        .and_then(|v| serde_json::from_value::<Source>(v.clone()).ok());

    Ok((data, schema.to_string(), source))
}
