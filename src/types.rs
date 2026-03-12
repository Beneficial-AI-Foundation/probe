use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

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

/// Schema 2.0 envelope for merged files, generic over the data-entry type.
///
/// For atoms use `MergedEnvelope<Atom>`, for specs/proofs use
/// `MergedEnvelope<serde_json::Value>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MergedEnvelope<D> {
    pub schema: String,
    #[serde(rename = "schema-version")]
    pub schema_version: String,
    pub tool: Tool,
    pub inputs: Vec<InputProvenance>,
    pub timestamp: String,
    pub data: BTreeMap<String, D>,
}

pub type MergedAtomEnvelope = MergedEnvelope<Atom>;
pub type MergedGenericEnvelope = MergedEnvelope<serde_json::Value>;

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

fn deserialize_code_text<'de, D>(deserializer: D) -> Result<CodeText, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<CodeText>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Line range of an atom's definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeText {
    #[serde(rename = "lines-start", default)]
    pub lines_start: usize,
    #[serde(rename = "lines-end", default)]
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
    #[serde(rename = "code-text", default, deserialize_with = "deserialize_code_text")]
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

// ---------------------------------------------------------------------------
// Schema categories
// ---------------------------------------------------------------------------

/// The three categories of data files the merge tool can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCategory {
    Atoms,
    Specs,
    Proofs,
}

impl SchemaCategory {
    /// The `schema` value used in the merged output envelope.
    pub fn merged_schema(&self) -> &'static str {
        match self {
            SchemaCategory::Atoms => "probe/merged-atoms",
            SchemaCategory::Specs => "probe/merged-specs",
            SchemaCategory::Proofs => "probe/merged-proofs",
        }
    }

    /// Human-readable label for log messages.
    pub fn label(&self) -> &'static str {
        match self {
            SchemaCategory::Atoms => "atoms",
            SchemaCategory::Specs => "specs",
            SchemaCategory::Proofs => "proofs",
        }
    }
}

impl std::fmt::Display for SchemaCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Determine the schema category from a `schema` string.
///
/// Returns `None` for unrecognized schemas.
pub fn detect_category(schema: &str) -> Option<SchemaCategory> {
    if schema.ends_with("/atoms")
        || schema.ends_with("/enriched-atoms")
        || schema.ends_with("/extract")
        || schema == "probe/merged-atoms"
    {
        Some(SchemaCategory::Atoms)
    } else if schema.ends_with("/specs") || schema == "probe/merged-specs" {
        Some(SchemaCategory::Specs)
    } else if schema.ends_with("/proofs") || schema == "probe/merged-proofs" {
        Some(SchemaCategory::Proofs)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Envelope loading
// ---------------------------------------------------------------------------

/// Parsed envelope metadata returned by [`load_envelope`].
pub struct EnvelopeMeta {
    pub schema: String,
    pub category: SchemaCategory,
    pub provenance: Vec<InputProvenance>,
    /// The raw `data` value, ready to be deserialized into the appropriate type.
    pub data_value: serde_json::Value,
}

/// Parse a Schema 2.0 envelope, extracting shared metadata.
///
/// Validates the schema-version, detects the [`SchemaCategory`], and extracts
/// provenance (flattening `inputs` for previously merged files).
pub fn load_envelope(path: &std::path::Path) -> Result<EnvelopeMeta, String> {
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

    let category = detect_category(schema).ok_or_else(|| {
        format!(
            "{}: unsupported schema \"{schema}\" (expected */atoms, */enriched-atoms, */specs, */proofs, or probe/merged-*)",
            path.display()
        )
    })?;

    let data_value = raw
        .get("data")
        .ok_or_else(|| format!("{}: missing \"data\" field", path.display()))?
        .clone();

    let is_merged = schema.starts_with("probe/merged-");
    let provenance = if is_merged {
        raw.get("inputs")
            .and_then(|v| serde_json::from_value::<Vec<InputProvenance>>(v.clone()).ok())
            .unwrap_or_default()
    } else {
        let source = raw
            .get("source")
            .and_then(|v| serde_json::from_value::<Source>(v.clone()).ok())
            .unwrap_or_else(|| Source {
                repo: String::new(),
                commit: String::new(),
                language: String::new(),
                package: path.file_stem().map_or_else(
                    || "unknown".to_string(),
                    |s| s.to_string_lossy().to_string(),
                ),
                package_version: String::new(),
            });
        vec![InputProvenance {
            schema: schema.to_string(),
            source,
        }]
    };

    Ok(EnvelopeMeta {
        schema: schema.to_string(),
        category,
        provenance,
        data_value,
    })
}

/// Result of loading an atom file: data dictionary and provenance entries.
pub type LoadResult = (BTreeMap<String, Atom>, Vec<InputProvenance>);

/// Load a Schema 2.0 atom file (convenience wrapper around [`load_envelope`]).
///
/// Returns typed `Atom` entries. Errors if the file is not an atoms-category schema.
pub fn load_atom_file(path: &std::path::Path) -> Result<LoadResult, String> {
    let meta = load_envelope(path)?;
    if meta.category != SchemaCategory::Atoms {
        return Err(format!(
            "{}: expected atoms schema, got {} (\"{}\")",
            path.display(),
            meta.category,
            meta.schema
        ));
    }
    let data: BTreeMap<String, Atom> = serde_json::from_value(meta.data_value)
        .map_err(|e| format!("{}: failed to deserialize atoms: {e}", path.display()))?;
    Ok((data, meta.provenance))
}

/// Result of loading a generic data file.
pub type GenericLoadResult = (
    BTreeMap<String, serde_json::Value>,
    Vec<InputProvenance>,
    SchemaCategory,
);

// ---------------------------------------------------------------------------
// Translations
// ---------------------------------------------------------------------------

/// A single mapping entry in a translations file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMapping {
    pub from: String,
    pub to: String,
    pub confidence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

/// A translations file mapping code-names between languages.
#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationsFile {
    pub schema: String,
    #[serde(rename = "schema-version")]
    pub schema_version: String,
    pub mappings: Vec<TranslationMapping>,
}

/// Load a translations file and build bidirectional lookup maps.
///
/// Returns two maps: `from → to` and `to → from`.
pub fn load_translations(
    path: &std::path::Path,
) -> Result<(HashMap<String, String>, HashMap<String, String>), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read translations {}: {e}", path.display()))?;

    let file: TranslationsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse translations {}: {e}", path.display()))?;

    if file.schema != "probe/translations" {
        return Err(format!(
            "{}: expected schema \"probe/translations\", got \"{}\"",
            path.display(),
            file.schema
        ));
    }

    let mut from_to = HashMap::new();
    let mut to_from = HashMap::new();

    for mapping in &file.mappings {
        from_to.insert(mapping.from.clone(), mapping.to.clone());
        to_from.insert(mapping.to.clone(), mapping.from.clone());
    }

    Ok((from_to, to_from))
}

/// Load any Schema 2.0 data file as opaque JSON entries.
///
/// Works for atoms, specs, and proofs. Returns the data as generic JSON
/// values along with provenance and the detected category.
pub fn load_generic_file(path: &std::path::Path) -> Result<GenericLoadResult, String> {
    let meta = load_envelope(path)?;
    let data: BTreeMap<String, serde_json::Value> = serde_json::from_value(meta.data_value)
        .map_err(|e| format!("{}: failed to deserialize data: {e}", path.display()))?;
    Ok((data, meta.provenance, meta.category))
}
