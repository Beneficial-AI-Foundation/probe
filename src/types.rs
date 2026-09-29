use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

// @kb: kb/engineering/schema.md#envelope
/// Schema 3.0 envelope for single-tool atom files.
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

// @kb: kb/engineering/schema.md#envelope — merged variant with provenance
/// Schema 3.0 envelope for merged files, generic over the data-entry type.
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
    /// Any additional `source` fields the producer emitted (e.g. `class`).
    /// Captured with `#[serde(flatten)]` so they round-trip through consumers
    /// that only model the fields above, instead of being silently dropped on
    /// re-emit.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CodeText {
    #[serde(rename = "lines-start", default)]
    pub lines_start: usize,
    #[serde(rename = "lines-end", default)]
    pub lines_end: usize,
}

// @kb: kb/engineering/schema.md#atom — core fields; P10 for extensions
/// A single atom with typed core fields and passthrough extensions.
///
/// The `extensions` field captures any language-specific optional fields
/// (e.g., `dependencies-with-locations`, `is-hidden`, `rust-source`) without
/// the merge tool needing to know about them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atom {
    #[serde(rename = "display-name")]
    pub display_name: String,
    pub dependencies: BTreeSet<String>,
    #[serde(rename = "code-module")]
    pub code_module: String,
    #[serde(rename = "code-path")]
    pub code_path: String,
    #[serde(
        rename = "code-text",
        default,
        deserialize_with = "deserialize_code_text"
    )]
    pub code_text: CodeText,
    pub kind: String,
    pub language: String,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Atom {
    // @kb: kb/engineering/properties.md#p3-stub-detection-is-structural
    /// An atom is a stub when it has no source location.
    pub fn is_stub(&self) -> bool {
        self.code_path.is_empty()
            && self.code_text.lines_start == 0
            && self.code_text.lines_end == 0
    }
}

// @kb: kb/engineering/schema.md#common-optional-fields — status-origin marker
/// Validate every atom's `status-origin` marker against the two-value enum
/// (ADR-006 Decision 2: `"translation"` or `"kernel-taint"`).
///
/// The executable schema constrains the marker, but the runtime load paths do
/// not schema-validate, so `probe enrich` and `probe summary` call this at
/// their boundaries: an out-of-contract marker fails closed here instead of
/// silently reading as absent (non-string values) or being presented as local
/// evidence (unknown strings).
pub fn validate_status_origins(atoms: &BTreeMap<String, Atom>, origin: &str) -> Result<(), String> {
    for (code_name, atom) in atoms {
        if let Some(value) = atom.extensions.get("status-origin") {
            match value.as_str() {
                Some("translation" | "kernel-taint") => {}
                _ => {
                    return Err(format!(
                        "{origin}: atom {code_name:?} carries invalid status-origin {value} \
                         (expected \"translation\" or \"kernel-taint\", ADR-006)"
                    ));
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Schema categories
// ---------------------------------------------------------------------------

// @kb: kb/engineering/schema.md#registered-schema-values
/// Schema string written by `probe project`. A projection is a *view* of an
/// authoritative graph: read-only consumers accept it, recomputation
/// boundaries (merge, enrich) reject it (ADR-006).
pub const PROJECTED_ATOMS_SCHEMA: &str = "probe/projected-atoms";

// @kb: kb/engineering/glossary.md#schema-category
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
        || schema == PROJECTED_ATOMS_SCHEMA
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
#[derive(Debug)]
pub struct EnvelopeMeta {
    pub schema: String,
    pub category: SchemaCategory,
    pub provenance: Vec<InputProvenance>,
    /// The envelope's `tool` metadata; `None` when absent or malformed.
    /// Consumed by the authority validator ([`crate::authority`]), which
    /// rejects atoms envelopes without well-formed tool metadata.
    pub tool: Option<Tool>,
    /// Whether the raw envelope carries a `projection` field (any value,
    /// including `null`). Presence marks a legacy projection (ADR-006).
    pub has_projection_field: bool,
    /// The raw `data` value, ready to be deserialized into the appropriate type.
    pub data_value: serde_json::Value,
}

// @kb: kb/engineering/properties.md#p1-envelope-completeness
// @kb: kb/engineering/properties.md#p9-provenance-is-preserved
/// Parse an already-loaded Schema 3.x envelope value, extracting shared
/// metadata. `origin` names the input in error messages (usually a path).
///
/// Validates the schema-version, detects the [`SchemaCategory`], and extracts
/// provenance. Composed-provenance detection is **structural**: an envelope
/// with an `inputs` array is composed (its inventory is preserved regardless
/// of schema string, e.g. `probe-aeneas/extract`), one without is single-tool
/// and wraps its `source` into one provenance entry. An envelope carrying
/// both `source` and `inputs` is rejected as ambiguous, and a composed
/// envelope's `inputs` must be a non-empty well-formed array.
pub fn parse_envelope(raw: &serde_json::Value, origin: &str) -> Result<EnvelopeMeta, String> {
    let schema_version = raw
        .get("schema-version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !schema_version.starts_with("3.") {
        return Err(format!(
            "{origin}: incompatible schema-version \"{schema_version}\" (expected 3.x)"
        ));
    }

    let schema = raw
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{origin}: missing \"schema\" field"))?;

    let category = detect_category(schema).ok_or_else(|| {
        format!(
            "{origin}: unsupported schema \"{schema}\" (expected */atoms, */enriched-atoms, */extract, */specs, */proofs, probe/merged-*, or probe/projected-atoms)"
        )
    })?;

    let data_value = raw
        .get("data")
        .ok_or_else(|| format!("{origin}: missing \"data\" field"))?
        .clone();

    let provenance = if let Some(inputs) = raw.get("inputs") {
        // Provenance shape is the composed/single-tool discriminator (P9);
        // an envelope carrying both is ambiguous and matches neither branch
        // of the executable schema.
        if raw.get("source").is_some() {
            return Err(format!(
                "{origin}: ambiguous provenance — envelope carries both \"source\" and \"inputs\""
            ));
        }
        // P9: a composed envelope's inventory must survive loading — a
        // malformed or empty `inputs` array is an error, not an empty
        // inventory.
        let inventory = serde_json::from_value::<Vec<InputProvenance>>(inputs.clone())
            .map_err(|e| format!("{origin}: malformed \"inputs\" provenance: {e}"))?;
        if inventory.is_empty() {
            return Err(format!(
                "{origin}: empty \"inputs\" provenance — a composed envelope must list at least one source"
            ));
        }
        inventory
    } else {
        let source = raw
            .get("source")
            .and_then(|v| serde_json::from_value::<Source>(v.clone()).ok())
            .unwrap_or_else(|| Source {
                repo: String::new(),
                commit: String::new(),
                language: String::new(),
                package: std::path::Path::new(origin).file_stem().map_or_else(
                    || "unknown".to_string(),
                    |s| s.to_string_lossy().to_string(),
                ),
                package_version: String::new(),
                extensions: BTreeMap::new(),
            });
        vec![InputProvenance {
            schema: schema.to_string(),
            source,
        }]
    };

    let tool = raw
        .get("tool")
        .and_then(|v| serde_json::from_value::<Tool>(v.clone()).ok());
    let has_projection_field = raw
        .as_object()
        .is_some_and(|o| o.contains_key("projection"));

    Ok(EnvelopeMeta {
        schema: schema.to_string(),
        category,
        provenance,
        tool,
        has_projection_field,
        data_value,
    })
}

/// Parse a Schema 3.x envelope file (read + JSON parse + [`parse_envelope`]).
pub fn load_envelope(path: &std::path::Path) -> Result<EnvelopeMeta, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let raw: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON in {}: {e}", path.display()))?;

    parse_envelope(&raw, &path.display().to_string())
}

/// Result of loading an atom file: data dictionary and provenance entries.
pub type LoadResult = (BTreeMap<String, Atom>, Vec<InputProvenance>);

/// Load a Schema 3.0 atom file (convenience wrapper around [`load_envelope`]).
///
/// Returns typed `Atom` entries. Errors if the file is not an atoms-category schema.
///
/// Performs **no authority validation** (ADR-006): projections and
/// pre-contract envelopes load without error. Library callers at an envelope
/// boundary should use [`crate::authority::load_validated_atom_file`] with
/// the appropriate [`crate::authority::AuthorityScope`] instead.
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
// Cross-language mappings
// ---------------------------------------------------------------------------

// @kb: kb/engineering/schema.md#mappings-file-format
/// A single entry in a cross-language mappings file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub from: String,
    pub to: String,
    pub confidence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

/// A cross-language mappings file linking code-names between languages.
#[derive(Debug, Serialize, Deserialize)]
pub struct MappingsFile {
    pub schema: String,
    #[serde(rename = "schema-version")]
    pub schema_version: String,
    pub mappings: Vec<Mapping>,
}

/// Load a mappings file and build bidirectional lookup maps.
///
/// Returns two maps: `from → [to₁, to₂, …]` and `to → [from₁, from₂, …]`.
/// A single `from` key may map to multiple `to` targets (1-to-many).
#[allow(clippy::type_complexity)]
pub fn load_mappings(
    path: &std::path::Path,
) -> Result<(HashMap<String, Vec<String>>, HashMap<String, Vec<String>>), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read mappings {}: {e}", path.display()))?;

    let file: MappingsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse mappings {}: {e}", path.display()))?;

    if file.schema != "probe/mappings" {
        return Err(format!(
            "{}: expected schema \"probe/mappings\", got \"{}\"",
            path.display(),
            file.schema
        ));
    }

    let mut from_to: HashMap<String, Vec<String>> = HashMap::new();
    let mut to_from: HashMap<String, Vec<String>> = HashMap::new();

    for mapping in &file.mappings {
        from_to
            .entry(mapping.from.clone())
            .or_default()
            .push(mapping.to.clone());
        to_from
            .entry(mapping.to.clone())
            .or_default()
            .push(mapping.from.clone());
    }

    Ok((from_to, to_from))
}

/// Load any Schema 3.0 data file as opaque JSON entries.
///
/// Works for atoms, specs, and proofs. Returns the data as generic JSON
/// values along with provenance and the detected category.
pub fn load_generic_file(path: &std::path::Path) -> Result<GenericLoadResult, String> {
    let meta = load_envelope(path)?;
    let data: BTreeMap<String, serde_json::Value> = serde_json::from_value(meta.data_value)
        .map_err(|e| format!("{}: failed to deserialize data: {e}", path.display()))?;
    Ok((data, meta.provenance, meta.category))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR-006 Decision 2: `status-origin` is an enum over exactly two
    /// values. Both values and an absent marker pass; unknown strings and
    /// non-string values are rejected (fail-closed — the runtime does not
    /// schema-validate).
    #[test]
    fn validate_status_origins_enforces_the_two_value_enum() {
        let atom_with = |origin: serde_json::Value| -> BTreeMap<String, Atom> {
            let mut atom = serde_json::json!({
                "display-name": "f",
                "dependencies": [],
                "code-module": "",
                "code-path": "src/lib.rs",
                "code-text": {"lines-start": 1, "lines-end": 2},
                "kind": "exec",
                "language": "rust"
            });
            atom["status-origin"] = origin;
            let atom: Atom = serde_json::from_value(atom).unwrap();
            let mut map = BTreeMap::new();
            map.insert("f".to_string(), atom);
            map
        };

        for good in ["translation", "kernel-taint"] {
            let atoms = atom_with(serde_json::json!(good));
            assert!(validate_status_origins(&atoms, "t").is_ok(), "{good}");
        }

        // Absent marker passes.
        let mut unmarked = atom_with(serde_json::json!("translation"));
        unmarked
            .get_mut("f")
            .unwrap()
            .extensions
            .remove("status-origin");
        assert!(validate_status_origins(&unmarked, "t").is_ok());

        for bad in [
            serde_json::json!("graph-taint"),
            serde_json::json!(""),
            serde_json::json!(null),
            serde_json::json!(42),
            serde_json::json!({}),
        ] {
            let atoms = atom_with(bad.clone());
            let err = validate_status_origins(&atoms, "t").unwrap_err();
            assert!(err.contains("invalid status-origin"), "{bad}: {err}");
            assert!(err.contains("\"f\""), "{err}");
        }
    }

    /// P9: a composed envelope's inventory must survive loading — a malformed
    /// `inputs` array is an error, never a silently emptied inventory.
    #[test]
    fn parse_envelope_rejects_malformed_inputs() {
        let raw = serde_json::json!({
            "schema": "probe/merged-atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe", "version": "0.5.0", "command": "merge"},
            "inputs": [ {"schema": "probe-rust/extract"} ],
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        });
        let err = parse_envelope(&raw, "test-input").unwrap_err();
        assert!(err.contains("malformed \"inputs\""), "{err}");

        let not_an_array = serde_json::json!({
            "schema": "probe/merged-atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe", "version": "0.5.0", "command": "merge"},
            "inputs": "oops",
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        });
        let err = parse_envelope(&not_an_array, "test-input").unwrap_err();
        assert!(err.contains("malformed \"inputs\""), "{err}");
    }

    /// Provenance shape is the composed/single-tool discriminator: an
    /// envelope carrying both `source` and `inputs` is ambiguous and must be
    /// rejected (never `inputs`-wins with `source` silently dropped), and an
    /// empty `inputs` inventory is an error (never propagated into output
    /// that violates the executable schema's `minItems: 1`).
    #[test]
    fn parse_envelope_rejects_ambiguous_and_empty_provenance() {
        let both = serde_json::json!({
            "schema": "probe-lean/extract",
            "schema-version": "3.0",
            "tool": {"name": "probe-lean", "version": "0.16.0", "command": "extract"},
            "source": {"repo": "r", "commit": "c", "language": "lean",
                       "package": "p", "package-version": "1.0"},
            "inputs": [
                {"schema": "probe-lean/extract", "source": {"repo": "r", "commit": "c",
                 "language": "lean", "package": "p", "package-version": "1.0"}}
            ],
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        });
        let err = parse_envelope(&both, "test-input").unwrap_err();
        assert!(err.contains("ambiguous provenance"), "{err}");

        let empty = serde_json::json!({
            "schema": "probe/merged-atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe", "version": "0.5.0", "command": "merge"},
            "inputs": [],
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        });
        let err = parse_envelope(&empty, "test-input").unwrap_err();
        assert!(err.contains("empty \"inputs\""), "{err}");
    }

    /// Composed detection is structural (P9): an `inputs` array marks a
    /// composed envelope regardless of schema string; `source` marks
    /// single-tool.
    #[test]
    fn parse_envelope_detects_composed_shape_structurally() {
        let composed = serde_json::json!({
            "schema": "probe-aeneas/extract",
            "schema-version": "3.0",
            "tool": {"name": "probe-aeneas", "version": "0.21.0", "command": "extract"},
            "inputs": [
                {"schema": "probe-rust/extract", "source": {"repo": "r", "commit": "c",
                 "language": "rust", "package": "pkg-rust", "package-version": "1.0"}},
                {"schema": "probe-lean/extract", "source": {"repo": "r", "commit": "c",
                 "language": "lean", "package": "pkg-lean", "package-version": "1.0"}}
            ],
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        });
        let meta = parse_envelope(&composed, "aeneas.json").unwrap();
        assert_eq!(meta.provenance.len(), 2, "composed inventory preserved");
        assert_eq!(meta.provenance[0].source.package, "pkg-rust");
        assert_eq!(meta.provenance[1].source.package, "pkg-lean");
    }

    /// Unknown `source` fields (e.g. `class`) must survive a deserialize →
    /// serialize round-trip instead of being dropped.
    #[test]
    fn source_preserves_unknown_fields() {
        let json = r#"{
            "repo": "r", "commit": "c", "language": "lean",
            "package": "p", "package-version": "1.0",
            "class": "security-protocol"
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        assert_eq!(
            source.extensions.get("class").and_then(|v| v.as_str()),
            Some("security-protocol"),
            "unknown source field captured via flatten"
        );
        let out = serde_json::to_value(&source).unwrap();
        assert_eq!(
            out.get("class").and_then(|v| v.as_str()),
            Some("security-protocol"),
            "unknown source field re-emitted, not dropped"
        );
        // Known fields still serialize under their renamed keys.
        assert_eq!(
            out.get("package-version").and_then(|v| v.as_str()),
            Some("1.0")
        );
    }
}
