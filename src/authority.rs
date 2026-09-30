// @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
// @kb: kb/decisions/006-correspondence-records.md

//! Shared authority validator for envelope boundaries (ADR-006).
//!
//! Two rejection predicates, applied to atoms-category envelopes:
//!
//! - **Projection rejection** (Decision 6): a projection is a view of an
//!   authoritative graph; recomputing enrichment over it would launder
//!   truncated views into stronger labels. Detected as the
//!   `probe/projected-atoms` schema string OR the presence of a `projection`
//!   envelope field (the legacy `probe/merged-atoms` form) — presence-based,
//!   so `projection: null` counts as present.
//! - **Per-producer version gate** (Decision 7): envelopes whose `tool.name`
//!   is in the reserved-threshold table with `tool.version` below that
//!   producer's contract release are pre-contract evidence and are rejected.
//!   The `probe` entry is an interval (pre-1.0 hub releases only) plus a
//!   `tool.command: "merge-atoms"` rejection at any version (a command the
//!   hub never shipped; the legacy probe-verus composer masquerades under
//!   `tool.name: "probe"` at its own 8.x version).
//!
//! Recomputation boundaries (`probe merge`, `probe enrich`) apply both
//! predicates; read-only consumers (`probe summary`, `probe project`) apply
//! only the version gate — views with inherited labels are legitimate to
//! read. The bare-map library API (`merge_atom_maps`,
//! `enrich_verification_status`) cannot check authority: callers own the
//! envelope boundary.

use crate::types::{
    load_envelope, Atom, EnvelopeMeta, InputProvenance, SchemaCategory, PROJECTED_ATOMS_SCHEMA,
};
use std::collections::BTreeMap;
use std::path::Path;

/// A parsed `tool.version`: (major, minor, patch).
type Version = (u64, u64, u64);

// @kb: kb/decisions/006-correspondence-records.md — Decision 7 gate table
/// Reserved contract-release thresholds (floors). Each producer's contract
/// release must ship with exactly its reserved version; a hub-side change
/// would strand composers that compile this table in via their pinned hub
/// dependency.
const GATE_FLOORS: [(&str, Version); 4] = [
    ("probe-aeneas", (0, 21, 0)),
    ("probe-lean", (0, 16, 0)),
    ("probe-leanblueprint", (0, 11, 0)),
    ("probe-vcvio", (0, 2, 0)),
];

/// The `probe` gate entry is an interval, not a floor: `PROBE_GATE_MIN` is
/// the reserved hub contract release, `PROBE_GATE_CEILING` is exclusive
/// (crossing 1.0 is a coordinated major release that re-bumps composers).
const PROBE_GATE_MIN: Version = (0, 5, 0);
const PROBE_GATE_CEILING: Version = (1, 0, 0);

/// The command string of the legacy probe-verus composer, rejected at any
/// version on `tool.name: "probe"` envelopes (the hub never shipped it).
const LEGACY_COMPOSER_COMMAND: &str = "merge-atoms";

/// Which rejection predicates apply at a given envelope boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityScope {
    /// Recomputation boundary (merge, enrich): projection rejection + gate.
    Recompute,
    /// Read-only consumer (summary, project input): version gate only —
    /// projections stay readable.
    ReadOnly,
}

/// Parse a `tool.version` string as `major.minor` or `major.minor.patch`
/// with purely numeric components. Anything else (including pre-release
/// suffixes) is unparsable — rejected on gated tool names, fail-closed.
fn parse_version(s: &str) -> Option<Version> {
    // Digits only: `u64::from_str` alone would also accept a leading `+`.
    fn component(p: &str) -> Option<u64> {
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        p.parse().ok()
    }
    let mut parts = s.split('.');
    let major = component(parts.next()?)?;
    let minor = component(parts.next()?)?;
    let patch = match parts.next() {
        None => 0,
        Some(p) => component(p)?,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

// @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
/// Validate an envelope's authority per ADR-006 Decisions 6 and 7.
///
/// Applies only to atoms-category envelopes (specs/proofs carry no
/// verification evidence subject to enrichment); non-atoms envelopes pass.
/// `origin` names the input in error messages (usually a path).
pub fn validate_authority(
    meta: &EnvelopeMeta,
    origin: &str,
    scope: AuthorityScope,
) -> Result<(), String> {
    if meta.category != SchemaCategory::Atoms {
        return Ok(());
    }

    if scope == AuthorityScope::Recompute {
        if meta.schema == PROJECTED_ATOMS_SCHEMA {
            return Err(format!(
                "{origin}: projected input (schema \"{PROJECTED_ATOMS_SCHEMA}\") rejected — \
                 projections are views; regenerate from the authoritative graph (ADR-006)"
            ));
        }
        if meta.has_projection_field {
            return Err(format!(
                "{origin}: legacy projection rejected (envelope carries a \"projection\" field) — \
                 projections are views; regenerate from the authoritative graph (ADR-006)"
            ));
        }
    }

    let tool = meta.tool.as_ref().ok_or_else(|| {
        format!(
            "{origin}: missing or malformed \"tool\" metadata \
             (name/version/command required for authority validation, ADR-006)"
        )
    })?;

    if tool.name == "probe" {
        if tool.command == LEGACY_COMPOSER_COMMAND {
            return Err(format!(
                "{origin}: rejected — tool.command \"{LEGACY_COMPOSER_COMMAND}\" under \
                 tool.name \"probe\" is the legacy probe-verus composer, which the hub \
                 never shipped (ADR-006 Decision 7)"
            ));
        }
        let version = parse_version(&tool.version).ok_or_else(|| {
            format!(
                "{origin}: unparsable tool.version \"{}\" on gated tool \"probe\" (ADR-006)",
                tool.version
            )
        })?;
        if version < PROBE_GATE_MIN || version >= PROBE_GATE_CEILING {
            return Err(format!(
                "{origin}: pre-contract envelope rejected — tool \"probe\" version \"{}\" is \
                 outside the accepted interval 0.5.0 ≤ v < 1.0.0; regenerate with the \
                 contract release (ADR-006 Decision 7)",
                tool.version
            ));
        }
        return Ok(());
    }

    if let Some((_, floor)) = GATE_FLOORS.iter().find(|(name, _)| *name == tool.name) {
        let version = parse_version(&tool.version).ok_or_else(|| {
            format!(
                "{origin}: unparsable tool.version \"{}\" on gated tool \"{}\" (ADR-006)",
                tool.version, tool.name
            )
        })?;
        if version < *floor {
            return Err(format!(
                "{origin}: pre-contract envelope rejected — tool \"{}\" version \"{}\" is \
                 below its contract-release threshold {}.{}.{}; re-extract with the \
                 contract release (ADR-006 Decision 7)",
                tool.name, tool.version, floor.0, floor.1, floor.2
            ));
        }
    }

    // Unknown tool names pass at any version (ADR-006 Decision 7: the gate
    // quantifies over the audited producer population).
    Ok(())
}

/// Result of a validated atoms load ([`load_validated_atom_file`]).
pub struct ValidatedAtomFile {
    pub atoms: BTreeMap<String, Atom>,
    pub provenance: Vec<InputProvenance>,
    /// Whether the input is a projection, in either format (the
    /// `probe/projected-atoms` schema or a legacy `projection` field). Only
    /// meaningful under [`AuthorityScope::ReadOnly`] — the `Recompute` scope
    /// rejects projections outright. `probe project` needs it: a projection's
    /// labels are inherited, never recomputed over the trimmed view
    /// (ADR-006), so enrich-before-trim must know the shape the loader
    /// otherwise discards.
    pub projected: bool,
}

/// Load an atoms file through the authority validator.
///
/// `load_envelope` + [`validate_authority`] + atoms-category check +
/// deserialization. Recomputation boundaries pass
/// [`AuthorityScope::Recompute`]; read-only consumers pass
/// [`AuthorityScope::ReadOnly`].
pub fn load_validated_atom_file(
    path: &Path,
    scope: AuthorityScope,
) -> Result<ValidatedAtomFile, String> {
    let meta = load_envelope(path)?;
    let origin = path.display().to_string();
    validate_authority(&meta, &origin, scope)?;
    if meta.category != SchemaCategory::Atoms {
        return Err(format!(
            "{origin}: expected atoms schema, got {} (\"{}\")",
            meta.category, meta.schema
        ));
    }
    let projected = meta.schema == PROJECTED_ATOMS_SCHEMA || meta.has_projection_field;
    let atoms: BTreeMap<String, Atom> = serde_json::from_value(meta.data_value)
        .map_err(|e| format!("{origin}: failed to deserialize atoms: {e}"))?;
    Ok(ValidatedAtomFile {
        atoms,
        provenance: meta.provenance,
        projected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::parse_envelope;
    use serde_json::json;

    fn envelope(schema: &str, tool: serde_json::Value) -> serde_json::Value {
        json!({
            "schema": schema,
            "schema-version": "3.0",
            "tool": tool,
            "source": {
                "repo": "r", "commit": "c", "language": "rust",
                "package": "p", "package-version": "1.0"
            },
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        })
    }

    fn meta_of(raw: &serde_json::Value) -> EnvelopeMeta {
        parse_envelope(raw, "test-input").unwrap()
    }

    fn validate(raw: &serde_json::Value, scope: AuthorityScope) -> Result<(), String> {
        validate_authority(&meta_of(raw), "test-input", scope)
    }

    fn tool(name: &str, version: &str, command: &str) -> serde_json::Value {
        json!({"name": name, "version": version, "command": command})
    }

    // -- projection rejection (Decision 6) ----------------------------------

    #[test]
    fn projected_schema_rejected_on_recompute() {
        let mut raw = envelope("probe/projected-atoms", tool("probe", "0.5.0", "project"));
        // projected envelopes are composed-shape
        raw["inputs"] = json!([{
            "schema": "probe-rust/extract",
            "source": {"repo": "r", "commit": "c", "language": "rust",
                       "package": "p", "package-version": "1.0"}
        }]);
        raw.as_object_mut().unwrap().remove("source");
        let err = validate(&raw, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("projected input"), "{err}");
    }

    #[test]
    fn legacy_projection_field_rejected_on_recompute() {
        let mut raw = envelope("probe/merged-atoms", tool("probe", "0.5.0", "project"));
        raw["inputs"] = json!([{
            "schema": "probe-rust/extract",
            "source": {"repo": "r", "commit": "c", "language": "rust",
                       "package": "p", "package-version": "1.0"}
        }]);
        raw.as_object_mut().unwrap().remove("source");
        raw["projection"] = json!({"mappings-file": "m.json", "seeds": 1,
            "forward-depth": 2, "reverse-depth": 0,
            "atoms-in": 10, "atoms-out": 3, "deps-trimmed": 4});
        let err = validate(&raw, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("legacy projection"), "{err}");
    }

    #[test]
    fn projection_null_counts_as_present() {
        let mut raw = envelope("probe/merged-atoms", tool("probe", "0.5.0", "merge"));
        raw["inputs"] = json!([{
            "schema": "probe-rust/extract",
            "source": {"repo": "r", "commit": "c", "language": "rust",
                       "package": "p", "package-version": "1.0"}
        }]);
        raw.as_object_mut().unwrap().remove("source");
        raw["projection"] = json!(null);
        let err = validate(&raw, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("legacy projection"), "{err}");
    }

    #[test]
    fn projections_readable_by_read_only_consumers() {
        let mut raw = envelope("probe/projected-atoms", tool("probe", "0.5.0", "project"));
        raw["inputs"] = json!([{
            "schema": "probe-rust/extract",
            "source": {"repo": "r", "commit": "c", "language": "rust",
                       "package": "p", "package-version": "1.0"}
        }]);
        raw.as_object_mut().unwrap().remove("source");
        raw["projection"] = json!(null);
        assert!(validate(&raw, AuthorityScope::ReadOnly).is_ok());
    }

    // -- version gate (Decision 7) ------------------------------------------

    #[test]
    fn gated_producers_below_threshold_rejected_at_threshold_accepted() {
        for (name, threshold, below) in [
            ("probe-lean", "0.16.0", "0.15.9"),
            ("probe-aeneas", "0.21.0", "0.20.0"),
            ("probe-leanblueprint", "0.11.0", "0.10.2"),
            ("probe-vcvio", "0.2.0", "0.1.0"),
        ] {
            let rejected = envelope("probe-lean/extract", tool(name, below, "extract"));
            let err = validate(&rejected, AuthorityScope::Recompute).unwrap_err();
            assert!(err.contains("pre-contract"), "{name} {below}: {err}");

            let accepted = envelope("probe-lean/extract", tool(name, threshold, "extract"));
            assert!(
                validate(&accepted, AuthorityScope::Recompute).is_ok(),
                "{name} {threshold} should pass"
            );
        }
    }

    #[test]
    fn probe_interval_floor_and_ceiling() {
        for (version, ok) in [
            ("0.4.0", false),
            ("0.4.9", false),
            ("0.5.0", true),
            ("0.17.3", true),
            ("1.0.0", false),
            ("8.0.1", false),
        ] {
            let raw = envelope("probe/merged-atoms", tool("probe", version, "merge"));
            let result = validate(&raw, AuthorityScope::Recompute);
            assert_eq!(result.is_ok(), ok, "probe {version}: {result:?}");
        }
    }

    #[test]
    fn probe_merge_atoms_command_rejected_at_any_version() {
        // In-interval version: only the command check catches it.
        let in_interval = envelope("probe/merged-atoms", tool("probe", "0.6.0", "merge-atoms"));
        let err = validate(&in_interval, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("merge-atoms"), "{err}");

        // The actual legacy Verus-composer shape: probe-verus's own 8.x
        // package version — caught by the command check and the ceiling.
        let legacy = envelope("probe/merged-atoms", tool("probe", "8.0.1", "merge-atoms"));
        let err = validate(&legacy, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("merge-atoms"), "{err}");
    }

    #[test]
    fn ungated_and_unknown_tools_pass_at_any_version() {
        // probe-verus extracts carry no imported evidence: no gate entry.
        let verus = envelope("probe-verus/atoms", tool("probe-verus", "2.0.0", "atomize"));
        assert!(validate(&verus, AuthorityScope::Recompute).is_ok());

        // Unknown names pass even with an unparsable version (audited-
        // population scope, ADR-006 Decision 7).
        let unknown = envelope(
            "probe-verus/atoms",
            tool("my-new-tool", "not.a.version", "x"),
        );
        assert!(validate(&unknown, AuthorityScope::Recompute).is_ok());
    }

    #[test]
    fn unparsable_version_on_gated_name_rejected() {
        for version in [
            "0.16.0-rc1",
            "v0.16.0",
            "0.16.0.1",
            "sixteen",
            "+0.16.0",
            "0.+16.0",
        ] {
            let raw = envelope("probe-lean/extract", tool("probe-lean", version, "extract"));
            let err = validate(&raw, AuthorityScope::Recompute).unwrap_err();
            assert!(err.contains("unparsable"), "{version}: {err}");
        }
    }

    #[test]
    fn missing_or_malformed_tool_metadata_rejected() {
        let mut missing = envelope("probe-lean/extract", json!(null));
        missing.as_object_mut().unwrap().remove("tool");
        let err = validate(&missing, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("tool"), "{err}");

        // Malformed: version is not a string.
        let malformed = envelope(
            "probe-lean/extract",
            json!({"name": "probe-lean", "version": 16, "command": "extract"}),
        );
        let err = validate(&malformed, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("tool"), "{err}");

        // Malformed: command missing entirely.
        let no_command = envelope(
            "probe-lean/extract",
            json!({"name": "probe-lean", "version": "0.16.0"}),
        );
        let err = validate(&no_command, AuthorityScope::Recompute).unwrap_err();
        assert!(err.contains("tool"), "{err}");
    }

    #[test]
    fn read_only_scope_still_runs_the_gate() {
        let raw = envelope(
            "probe-aeneas/extract",
            tool("probe-aeneas", "0.20.0", "extract"),
        );
        let err = validate(&raw, AuthorityScope::ReadOnly).unwrap_err();
        assert!(err.contains("pre-contract"), "{err}");
    }

    #[test]
    fn non_atoms_envelopes_pass_untouched() {
        // The gate applies to atoms envelopes only (ADR-006 Decision 7).
        let mut specs = envelope("probe-lean/specs", tool("probe-lean", "0.1.0", "specify"));
        specs["data"] = json!({});
        assert!(validate(&specs, AuthorityScope::Recompute).is_ok());
    }

    #[test]
    fn version_two_component_form_parses() {
        assert_eq!(parse_version("0.16"), Some((0, 16, 0)));
        assert_eq!(parse_version("0.16.2"), Some((0, 16, 2)));
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("1"), None);
    }
}
