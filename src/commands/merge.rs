use crate::authority::{load_validated_atom_file, validate_authority, AuthorityScope};
use crate::commands::propagate::enrich_verification_status;
use crate::types::{
    load_envelope, load_mappings, normalize_code_name, validate_mappings, validate_status_origins,
    Atom, InputProvenance, Mapping, MergedAtomEnvelope, MergedGenericEnvelope, SchemaCategory,
    Tool, MAPPING_CONFIDENCE_VALUES,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Merge statistics reported after the operation.
#[derive(Debug)]
pub struct MergeStats {
    pub total_entries: usize,
    pub stubs_replaced: usize,
    pub stubs_remaining: usize,
    pub entries_added: usize,
    pub keys_normalized: usize,
    pub conflicts: usize,
    /// Correspondence records newly attached by `--mappings` (0–2 per
    /// mapping: `maps-to` + `mapped-from`; re-application counts zero).
    pub records_attached: usize,
    /// Enrichment recomputation counts (atoms category; zero on the raw
    /// staging path and for specs/proofs).
    pub enriched_transitive: usize,
    pub enriched_local: usize,
}

impl MergeStats {
    fn new() -> Self {
        MergeStats {
            total_entries: 0,
            stubs_replaced: 0,
            stubs_remaining: 0,
            entries_added: 0,
            keys_normalized: 0,
            conflicts: 0,
            records_attached: 0,
            enriched_transitive: 0,
            enriched_local: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Correspondence records (maps-to / mapped-from)
// ---------------------------------------------------------------------------

const MAPS_TO: &str = "maps-to";
const MAPPED_FROM: &str = "mapped-from";
const RECORD_FIELDS: [&str; 2] = [MAPS_TO, MAPPED_FROM];

/// Record identity/sort key: the `(target, confidence, method)` triple, with
/// absent `method` keying as the empty string (P27). Boundary validation
/// ([`validate_and_canonicalize_records`]) guarantees every record reaching
/// this key is a canonical `{target, confidence, method?}` object, so the
/// triple determines the record.
fn record_key(record: &serde_json::Value) -> (String, String, String) {
    let field = |name: &str| {
        record
            .get(name)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    (field("target"), field("confidence"), field("method"))
}

/// Sort a record array by the identity triple and collapse duplicates (P27,
/// P14). Dedup is by the identity triple — the same relation the sort orders
/// by — so equal-identity records always collapse regardless of where they
/// sit in the array (a tied record between two duplicates must not shield
/// them). The first occurrence wins; after boundary validation and
/// canonicalization, equal triples are byte-identical records anyway.
fn sort_dedup_records(records: &mut Vec<serde_json::Value>) {
    records.sort_by_key(record_key);
    records.dedup_by(|a, b| record_key(a) == record_key(b));
}

// @kb: kb/engineering/schema.md#correspondence-records-maps-to-mapped-from
/// Validate and canonicalize one atom's `maps-to`/`mapped-from` fields,
/// fail-closed (ADR-006). Merge owns these reserved fields and re-emits them,
/// so a shape the executable schema rejects must not pass through — and the
/// union machinery is only lossless on canonical records (a malformed field
/// swallowing a valid union would make the result grouping-dependent,
/// breaking P4).
///
/// Validation: the field is an array; every entry is an object with a string
/// `target`, an in-vocabulary `confidence`, optionally a string `method`, and
/// nothing else (the executable schema's `correspondenceRecord` shape).
/// Canonicalization: `target` is normalized (P8), an empty `method` becomes
/// absent (`""` names no matching method — the canonical encoding of "none"
/// is omission, P27), and the array is sorted and deduped by the identity
/// triple.
fn validate_and_canonicalize_records(code_name: &str, atom: &mut Atom) -> Result<(), String> {
    for field in RECORD_FIELDS {
        let Some(value) = atom.extensions.get_mut(field) else {
            continue;
        };
        let Some(arr) = value.as_array_mut() else {
            return Err(format!(
                "atom {code_name:?}: {field} must be an array of correspondence records (ADR-006)"
            ));
        };
        for entry in arr.iter_mut() {
            let Some(obj) = entry.as_object_mut() else {
                return Err(format!(
                    "atom {code_name:?}: {field} entry {entry} is not a correspondence-record object (ADR-006)"
                ));
            };
            if let Some(unexpected) = obj
                .keys()
                .find(|k| !matches!(k.as_str(), "target" | "confidence" | "method"))
            {
                return Err(format!(
                    "atom {code_name:?}: {field} record carries unexpected field {unexpected:?} \
                     (expected only target/confidence/method, ADR-006)"
                ));
            }
            let target = match obj.get("target").and_then(|v| v.as_str()) {
                Some(t) => normalize_code_name(t),
                None => {
                    return Err(format!(
                        "atom {code_name:?}: {field} record is missing a string \"target\" (ADR-006)"
                    ));
                }
            };
            obj.insert("target".to_string(), serde_json::Value::String(target));
            match obj.get("confidence").and_then(|v| v.as_str()) {
                Some(c) if MAPPING_CONFIDENCE_VALUES.contains(&c) => {}
                _ => {
                    return Err(format!(
                        "atom {code_name:?}: {field} record carries invalid confidence {} (expected one of: {})",
                        obj.get("confidence").unwrap_or(&serde_json::Value::Null),
                        MAPPING_CONFIDENCE_VALUES.join(", ")
                    ));
                }
            }
            match obj.get("method") {
                None => {}
                Some(serde_json::Value::String(m)) if m.is_empty() => {
                    obj.remove("method");
                }
                Some(serde_json::Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "atom {code_name:?}: {field} record carries non-string method {other} (ADR-006)"
                    ));
                }
            }
        }
        sort_dedup_records(arr);
    }
    Ok(())
}

// @kb: kb/engineering/properties.md#p27-correspondence-records-are-unioned-and-inert
/// Union `other`'s `maps-to`/`mapped-from` arrays into `survivor`'s (P27):
/// applied on every equal-key merge resolution, so the surviving atom carries
/// the set union of both sides' correspondence records — they cannot be
/// re-derived without the mappings file. Both sides have passed
/// [`validate_and_canonicalize_records`] (every atom is validated during
/// per-input normalization before any resolution), so the non-array guards
/// below are defensive only.
fn union_correspondence_records(survivor: &mut Atom, other: &Atom) {
    for field in RECORD_FIELDS {
        let Some(incoming) = other.extensions.get(field).and_then(|v| v.as_array()) else {
            continue;
        };
        let entry = survivor
            .extensions
            .entry(field.to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        if let Some(existing) = entry.as_array_mut() {
            existing.extend(incoming.iter().cloned());
            sort_dedup_records(existing);
        }
    }
}

/// Compare two atoms ignoring their correspondence-record arrays. Equal-key
/// atoms that differ only in records are the same evidence (records union
/// through the collision, P27), not a distinct-real conflict.
fn atoms_equal_modulo_records(a: &Atom, b: &Atom) -> bool {
    let strip = |atom: &Atom| {
        let mut atom = atom.clone();
        for field in RECORD_FIELDS {
            atom.extensions.remove(field);
        }
        atom
    };
    strip(a) == strip(b)
}

/// Insert one correspondence record into `atom.<field>` (set-like, P13/P27):
/// returns `true` when the record was new, `false` on a no-op re-application.
/// The record is constructed in canonical form: an empty `method` is treated
/// as absent (P27 canonicalization).
fn push_record(
    atom: &mut Atom,
    field: &str,
    target: &str,
    confidence: &str,
    method: Option<&str>,
) -> bool {
    let mut record = serde_json::json!({ "target": target, "confidence": confidence });
    if let Some(m) = method.filter(|m| !m.is_empty()) {
        record["method"] = serde_json::Value::String(m.to_string());
    }
    let entry = atom
        .extensions
        .entry(field.to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let Some(records) = entry.as_array_mut() else {
        // Defensive: unreachable on merge paths (per-input validation
        // guarantees an array), kept for direct library callers.
        eprintln!(
            "Warning: atom field {field:?} is not an array; skipping record attachment for target {target:?}"
        );
        return false;
    };
    if records.contains(&record) {
        return false;
    }
    records.push(record);
    sort_dedup_records(records);
    true
}

// @kb: kb/engineering/properties.md#p13-correspondence-records-attach-unconditionally
/// `F_M`: attach correspondence records to the atoms of `base` for the fixed
/// mapping set (ADR-006). Attachment is unconditional (a dangling target is
/// warned about, never skipped), key-local, and set-like (P13), which is what
/// makes `F_M` idempotent and compatible with merge across groupings (P4 law
/// 3). `dependencies` is never modified. Returns the number of records newly
/// attached.
fn attach_correspondence_records(base: &mut BTreeMap<String, Atom>, mappings: &[Mapping]) -> usize {
    let mut attached = 0;
    for mapping in mappings {
        // Endpoints are normalized at file load (P8); re-normalize here so
        // bare-map callers constructing `Mapping` values in memory get the
        // same lookup rule.
        let from = normalize_code_name(&mapping.from);
        let to = normalize_code_name(&mapping.to);
        let from_present = base.contains_key(&from);
        let to_present = base.contains_key(&to);
        let confidence = mapping.confidence.as_str();
        let method = mapping.method.as_deref();

        if from_present {
            if !to_present {
                eprintln!(
                    "Warning: maps-to target {to:?} not present in merged map (record attached on {from:?}; dangling target, P13)"
                );
            }
            if push_record(
                base.get_mut(&from).unwrap(),
                MAPS_TO,
                &to,
                confidence,
                method,
            ) {
                attached += 1;
            }
        }
        if to_present {
            if !from_present {
                eprintln!(
                    "Warning: mapped-from target {from:?} not present in merged map (record attached on {to:?}; dangling target, P13)"
                );
            }
            if push_record(
                base.get_mut(&to).unwrap(),
                MAPPED_FROM,
                &from,
                confidence,
                method,
            ) {
                attached += 1;
            }
        }
    }
    attached
}

// ---------------------------------------------------------------------------
// Atom-specific merge (stub replacement, first-wins)
// ---------------------------------------------------------------------------

/// Normalize all keys and dependency references in an atom map.
/// Returns the normalized map, a count of keys that changed, and the
/// post-normalization collisions between *distinct real* atoms, as
/// `(colliding original key, occupied normalized key)` pairs — distinctness
/// is judged modulo correspondence records ([P27]: records union through
/// benign collapses instead of making atoms "distinct"). Collapsing a stub or
/// an identical-modulo-records duplicate is benign and not reported (P8);
/// correspondence records are unioned across every benign collision.
///
/// Correspondence-record fields are validated fail-closed and put in
/// canonical form here ([`validate_and_canonicalize_records`]): a malformed
/// record shape errors the whole input — merge re-emits these fields, and
/// the record union is only lossless on canonical arrays.
///
/// Shared by every recomputation boundary (ADR-006 carrier preparation):
/// a distinct-real intra-input collision is producer error, and both `probe
/// merge` (per input) and the unary `probe enrich` boundary reject it —
/// silently selecting one atom's evidence can launder contamination (P8).
#[allow(clippy::type_complexity)]
pub(crate) fn normalize_atoms(
    atoms: BTreeMap<String, Atom>,
) -> Result<(BTreeMap<String, Atom>, usize, Vec<(String, String)>), String> {
    let mut out: BTreeMap<String, Atom> = BTreeMap::new();
    let mut changed = 0;
    let mut dropped: Vec<(String, String)> = Vec::new();

    for (key, mut atom) in atoms {
        let norm_key = normalize_code_name(&key);
        if norm_key != key {
            changed += 1;
        }

        atom.dependencies = atom
            .dependencies
            .into_iter()
            .map(|d| normalize_code_name(&d))
            .collect();

        if let Some(dwl) = atom.extensions.get_mut("dependencies-with-locations") {
            if let Some(arr) = dwl.as_array_mut() {
                for entry in arr {
                    if let Some(cn) = entry.get("code-name").and_then(|v| v.as_str()) {
                        let norm = normalize_code_name(cn);
                        if let Some(obj) = entry.as_object_mut() {
                            obj.insert("code-name".to_string(), serde_json::Value::String(norm));
                        }
                    }
                }
            }
        }

        // P8/P27: validate record shape fail-closed and canonicalize —
        // record targets are code-names too.
        validate_and_canonicalize_records(&key, &mut atom)?;

        match out.get(&norm_key) {
            Some(existing) if existing.is_stub() && !atom.is_stub() => {
                union_correspondence_records(&mut atom, &out[&norm_key]);
                out.insert(norm_key, atom);
            }
            Some(existing) => {
                // Collapsing a stub or an identical-modulo-records duplicate
                // is benign (records union, P27); a colliding distinct real
                // atom is producer error and is surfaced to the caller (P8).
                if !atom.is_stub() && !atoms_equal_modulo_records(existing, &atom) {
                    dropped.push((key, norm_key));
                } else {
                    union_correspondence_records(out.get_mut(&norm_key).unwrap(), &atom);
                }
            }
            None => {
                out.insert(norm_key, atom);
            }
        }
    }

    Ok((out, changed, dropped))
}

/// Format a distinct-real intra-input collision rejection (P8): the error
/// names every colliding pair so the producer aliases can be fixed.
fn collision_error(dropped: &[(String, String)]) -> String {
    let pairs: Vec<String> = dropped
        .iter()
        .map(|(colliding, occupied)| format!("{colliding:?} vs {occupied:?}"))
        .collect();
    format!(
        "normalization collided {} distinct real atom(s) within one input ({}); \
         a single input offering two distinct atoms for one code-name is producer \
         error — fix the producer aliases and regenerate (P8)",
        dropped.len(),
        pairs.join(", ")
    )
}

// @kb: kb/engineering/properties.md#p6-atom-merge-is-first-wins-with-stub-replacement
// @kb: kb/engineering/properties.md#p13-correspondence-records-attach-unconditionally
/// Raw staging variant of [`merge_atom_maps`]: identical normalization,
/// conflict resolution, and record attachment, but **no enrichment
/// recomputation** — the output's derived statuses (`verified` /
/// `transitively-verified`) are potentially stale (off-carrier) until a final
/// enrichment pass runs.
///
/// This exists for multi-step pipelines (probe-aeneas) that mutate
/// verification statuses between merge steps and enrich exactly once at the
/// end; everyone else should use [`merge_atom_maps`]. Raw means *skip
/// recomputation*, never *skip validation*: mapping confidences and
/// correspondence-record shapes are validated fail-closed here (merge emits
/// those fields and must not violate the executable schema), and the
/// file-level counterpart [`merge_atom_files_raw`] applies the same authority
/// rejection as the public entry points. Per-input rejections are prefixed
/// with the 1-based input position.
pub fn merge_atom_maps_raw(
    maps: Vec<BTreeMap<String, Atom>>,
    mappings: Option<&[Mapping]>,
) -> Result<(BTreeMap<String, Atom>, MergeStats), String> {
    let mut stats = MergeStats::new();

    // Fail closed on an out-of-vocabulary mapping confidence before touching
    // any atom: merge writes `confidence` into the records it emits, and the
    // executable schema constrains it. `load_mappings` already validates file
    // inputs; this covers in-memory `Mapping` construction by library callers.
    if let Some(mappings) = mappings {
        validate_mappings(mappings)?;
    }

    let total = maps.len();
    // The 1-based input position, prefixed onto per-input rejections so the
    // offending producer file is identifiable from the argument order.
    let input_context = move |i: usize, e: String| format!("input #{} of {total}: {e}", i + 1);

    let mut maps_iter = maps.into_iter().enumerate();
    let (_, first) = maps_iter.next().unwrap_or_default();
    // P8: normalization runs per input, before conflict resolution — aliases
    // collapse within their own input before evidence from other inputs is
    // considered. A distinct-real intra-input collision is producer error and
    // rejects the merge (same rule as the unary enrich boundary; silently
    // selecting one atom's evidence would feed enrichment inside merge).
    let (mut base, norm_count, dropped) =
        normalize_atoms(first).map_err(|e| input_context(0, e))?;
    stats.keys_normalized += norm_count;
    if !dropped.is_empty() {
        return Err(input_context(0, collision_error(&dropped)));
    }

    for (i, incoming) in maps_iter {
        let (incoming, norm_count, dropped) =
            normalize_atoms(incoming).map_err(|e| input_context(i, e))?;
        stats.keys_normalized += norm_count;
        if !dropped.is_empty() {
            return Err(input_context(i, collision_error(&dropped)));
        }

        for (key, incoming_atom) in incoming {
            // P27: on every equal-key resolution the survivor carries the set
            // union of both sides' correspondence records.
            match base.get(&key) {
                Some(existing) if existing.is_stub() && !incoming_atom.is_stub() => {
                    let mut incoming_atom = incoming_atom;
                    union_correspondence_records(&mut incoming_atom, &base[&key]);
                    base.insert(key, incoming_atom);
                    stats.stubs_replaced += 1;
                }
                Some(existing) => {
                    if !existing.is_stub() && !incoming_atom.is_stub() {
                        stats.conflicts += 1;
                        eprintln!(
                            "  Warning: conflict for '{}' (keeping base version from {})",
                            key, existing.code_path
                        );
                    }
                    union_correspondence_records(base.get_mut(&key).unwrap(), &incoming_atom);
                }
                None => {
                    base.insert(key, incoming_atom);
                    stats.entries_added += 1;
                }
            }
        }
    }

    // F_M: attach correspondence records for the fixed mapping set (P13).
    if let Some(mappings) = mappings {
        stats.records_attached = attach_correspondence_records(&mut base, mappings);
    }

    stats.stubs_remaining = base.values().filter(|a| a.is_stub()).count();
    stats.total_entries = base.len();

    Ok((base, stats))
}

// @kb: kb/engineering/properties.md#p4-merge-associativity-on-the-carrier
// @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
/// Merge multiple atom maps into one (μ, optionally `F_M ∘ μ` with mappings).
///
/// The first map is the base. Each input is normalized (P8) before conflict
/// resolution; a distinct-real intra-input collision is an error, as is a
/// malformed correspondence-record shape or an out-of-vocabulary mapping
/// confidence (fail-closed: merge emits those fields and must not violate the
/// executable schema). For each subsequent map:
/// - Stubs in the base are replaced by real atoms from the incoming map.
/// - New atoms (not in base) are added.
/// - Real-vs-real conflicts keep the base version (first wins, P6).
/// - On every equal-key resolution, correspondence records union (P27).
///
/// If `mappings` is provided, correspondence records (`maps-to` on the `from`
/// atom, `mapped-from` on the `to` atom) are attached (P13) — `dependencies`
/// is never modified.
///
/// After all inputs are combined, enrichment is **recomputed** (P23) — stub
/// resolution can invalidate labels computed at extract time, so merged
/// labels are never inherited and every merge output is on the carrier (P4).
/// Multi-step pipelines that enrich once at the end stage on
/// [`merge_atom_maps_raw`] instead.
pub fn merge_atom_maps(
    maps: Vec<BTreeMap<String, Atom>>,
    mappings: Option<&[Mapping]>,
) -> Result<(BTreeMap<String, Atom>, MergeStats), String> {
    let (mut base, mut stats) = merge_atom_maps_raw(maps, mappings)?;
    let (transitive, local, _missing) = enrich_verification_status(&mut base);
    stats.enriched_transitive = transitive;
    stats.enriched_local = local;
    Ok((base, stats))
}

/// Shared loading for the file-level merge entry points: authority validation
/// (projection rejection + version gate, ADR-006), plus fail-closed
/// `status-origin` validation (Decision 2) — merge recomputes enrichment, so
/// a malformed marker must not silently read as absent.
#[allow(clippy::type_complexity)]
fn load_atom_inputs(
    paths: &[&Path],
) -> Result<(Vec<BTreeMap<String, Atom>>, Vec<InputProvenance>), String> {
    let mut maps = Vec::with_capacity(paths.len());
    let mut provenance = Vec::new();

    for path in paths {
        let (atoms, prov) = load_validated_atom_file(path, AuthorityScope::Recompute)?;
        validate_status_origins(&atoms).map_err(|e| format!("{}: {e}", path.display()))?;
        maps.push(atoms);
        provenance.extend(prov);
    }

    Ok((maps, provenance))
}

/// Load multiple atom files, flatten their provenance, and merge them.
///
/// This is a mid-level convenience between `merge_atom_maps` (pure in-memory)
/// and `cmd_merge` (full CLI with category detection and envelope writing).
/// Each input passes the shared authority validator (projection rejection +
/// version gate, ADR-006) before merging; output labels are re-enriched.
// @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
#[allow(clippy::type_complexity)]
pub fn merge_atom_files(
    paths: &[&Path],
    mappings: Option<&[Mapping]>,
) -> Result<(BTreeMap<String, Atom>, Vec<InputProvenance>, MergeStats), String> {
    let (maps, provenance) = load_atom_inputs(paths)?;
    let (merged, stats) = merge_atom_maps(maps, mappings)?;
    Ok((merged, provenance, stats))
}

/// Raw staging variant of [`merge_atom_files`] (see [`merge_atom_maps_raw`]):
/// same authority and `status-origin` validation on every input — raw means
/// skip recomputation, not skip validation (ADR-006) — but the merged output
/// carries potentially stale derived statuses until a final enrichment pass.
#[allow(clippy::type_complexity)]
pub fn merge_atom_files_raw(
    paths: &[&Path],
    mappings: Option<&[Mapping]>,
) -> Result<(BTreeMap<String, Atom>, Vec<InputProvenance>, MergeStats), String> {
    let (maps, provenance) = load_atom_inputs(paths)?;
    let (merged, stats) = merge_atom_maps_raw(maps, mappings)?;
    Ok((merged, provenance, stats))
}

// ---------------------------------------------------------------------------
// Generic merge for specs/proofs (last-wins, no stubs)
// ---------------------------------------------------------------------------

/// Normalize keys in a generic data map. Only normalizes the dictionary keys
/// (trailing-dot stripping); values are passed through untouched.
fn normalize_generic(
    data: BTreeMap<String, serde_json::Value>,
) -> (BTreeMap<String, serde_json::Value>, usize) {
    let mut out: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut changed = 0;

    for (key, value) in data {
        let norm_key = normalize_code_name(&key);
        if norm_key != key {
            changed += 1;
        }
        out.insert(norm_key, value);
    }

    (out, changed)
}

// @kb: kb/engineering/properties.md#p7-specsproofs-merge-is-last-wins
/// Merge multiple generic data maps into one (for specs and proofs).
///
/// Uses **last-wins** semantics: when the same code-name appears in multiple
/// inputs, the later one replaces the earlier one. This is appropriate for
/// specs/proofs where re-running a tool should override stale results.
pub fn merge_generic_maps(
    maps: Vec<BTreeMap<String, serde_json::Value>>,
) -> (BTreeMap<String, serde_json::Value>, MergeStats) {
    let mut stats = MergeStats::new();

    let mut maps_iter = maps.into_iter();
    let first = maps_iter.next().unwrap_or_default();
    let (mut base, norm_count) = normalize_generic(first);
    stats.keys_normalized += norm_count;

    for incoming in maps_iter {
        let (incoming, norm_count) = normalize_generic(incoming);
        stats.keys_normalized += norm_count;

        for (key, value) in incoming {
            if base.contains_key(&key) {
                stats.conflicts += 1;
            } else {
                stats.entries_added += 1;
            }
            base.insert(key, value);
        }
    }

    stats.total_entries = base.len();
    (base, stats)
}

// ---------------------------------------------------------------------------
// Unified merge command
// ---------------------------------------------------------------------------

// @kb: kb/tools/probe-merge.md — end-to-end merge pipeline
// @kb: kb/engineering/properties.md#p17-schema-category-consistency
/// Execute the `merge` command, auto-detecting the schema category.
pub fn cmd_merge(inputs: Vec<PathBuf>, output: PathBuf, mappings_path: Option<PathBuf>) {
    if inputs.len() < 2 {
        eprintln!("Error: merge requires at least 2 input files");
        std::process::exit(1);
    }

    let mut envelopes = Vec::new();

    for path in &inputs {
        println!("  Loading {}...", path.display());
        match load_envelope(path) {
            Ok(meta) => {
                if let Err(e) = validate_authority(
                    &meta,
                    &path.display().to_string(),
                    AuthorityScope::Recompute,
                ) {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
                println!(
                    "    schema: \"{}\" ({}), {} provenance entries",
                    meta.schema,
                    meta.category,
                    meta.provenance.len()
                );
                envelopes.push(meta);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }

    let category = envelopes[0].category;
    for (i, meta) in envelopes.iter().enumerate().skip(1) {
        if meta.category != category {
            eprintln!(
                "Error: category mismatch -- {} is {} but {} is {}. All inputs must be the same category.",
                inputs[0].display(), category,
                inputs[i].display(), meta.category,
            );
            std::process::exit(1);
        }
    }

    let mut provenance = Vec::new();
    for meta in &envelopes {
        provenance.extend(meta.provenance.clone());
    }

    println!();
    println!("Merging {} {} files...", inputs.len(), category);

    let tool = Tool {
        name: "probe".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: "merge".to_string(),
    };
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let merged_schema = category.merged_schema().to_string();

    let mappings = if let Some(ref m_path) = mappings_path {
        println!("  Loading mappings from {}...", m_path.display());
        match load_mappings(m_path) {
            Ok(m) => {
                println!("    {} mapping records", m.len());
                Some(m)
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    match category {
        SchemaCategory::Atoms => {
            let maps: Result<Vec<BTreeMap<String, Atom>>, String> = envelopes
                .into_iter()
                .enumerate()
                .map(|(i, meta)| {
                    serde_json::from_value(meta.data_value).map_err(|e| {
                        format!("{}: failed to deserialize atoms: {e}", inputs[i].display())
                    })
                })
                .collect();
            let maps = match maps {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };

            // Fail closed on out-of-enum status-origin markers (ADR-006
            // Decision 2): merge recomputes enrichment, and a malformed
            // marker must not silently read as absent.
            for (i, map) in maps.iter().enumerate() {
                if let Err(e) = validate_status_origins(map) {
                    eprintln!("Error: {}: {e}", inputs[i].display());
                    std::process::exit(1);
                }
            }

            let (merged, stats) = match merge_atom_maps(maps, mappings.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };

            let envelope = MergedAtomEnvelope {
                schema: merged_schema,
                schema_version: "3.1".to_string(),
                tool,
                inputs: provenance,
                timestamp,
                data: merged,
            };

            let json = match serde_json::to_string_pretty(&envelope) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error: failed to serialize merged atoms: {e}");
                    std::process::exit(1);
                }
            };
            if let Err(e) = std::fs::write(&output, &json) {
                eprintln!("Error: failed to write {}: {e}", output.display());
                std::process::exit(1);
            }

            print_stats(&output, &stats);
        }
        SchemaCategory::Specs | SchemaCategory::Proofs => {
            let maps: Result<Vec<BTreeMap<String, serde_json::Value>>, String> = envelopes
                .into_iter()
                .enumerate()
                .map(|(i, meta)| {
                    serde_json::from_value(meta.data_value).map_err(|e| {
                        format!("{}: failed to deserialize data: {e}", inputs[i].display())
                    })
                })
                .collect();
            let maps = match maps {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };

            let (merged, stats) = merge_generic_maps(maps);

            let envelope = MergedGenericEnvelope {
                schema: merged_schema,
                schema_version: "3.1".to_string(),
                tool,
                inputs: provenance,
                timestamp,
                data: merged,
            };

            let json = match serde_json::to_string_pretty(&envelope) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error: failed to serialize merged data: {e}");
                    std::process::exit(1);
                }
            };
            if let Err(e) = std::fs::write(&output, &json) {
                eprintln!("Error: failed to write {}: {e}", output.display());
                std::process::exit(1);
            }

            print_stats(&output, &stats);
        }
    }
}

fn print_stats(output: &std::path::Path, stats: &MergeStats) {
    println!();
    println!("Output: {}", output.display());
    println!("  Total entries:    {}", stats.total_entries);
    if stats.stubs_replaced > 0 {
        println!("  Stubs replaced:   {}", stats.stubs_replaced);
    }
    if stats.stubs_remaining > 0 {
        println!("  Stubs remaining:  {}", stats.stubs_remaining);
    }
    println!("  New entries added: {}", stats.entries_added);
    if stats.keys_normalized > 0 {
        println!("  Keys normalized:  {}", stats.keys_normalized);
    }
    if stats.conflicts > 0 {
        println!("  Conflicts:        {}", stats.conflicts);
    }
    if stats.records_attached > 0 {
        println!("  Records attached: {}", stats.records_attached);
    }
    if stats.enriched_transitive > 0 || stats.enriched_local > 0 {
        println!(
            "  Enrichment:       {} transitively-verified, {} locally-scoped verified",
            stats.enriched_transitive, stats.enriched_local
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::propagate::prepare_atoms;
    use crate::types::{endpoint_lookup_maps, CodeText};
    use std::collections::BTreeSet;

    fn make_real_atom(name: &str, code_path: &str, language: &str, kind: &str) -> Atom {
        Atom {
            display_name: name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: code_path.to_string(),
            code_text: CodeText {
                lines_start: 10,
                lines_end: 20,
            },
            kind: kind.to_string(),
            language: language.to_string(),
            extensions: BTreeMap::new(),
        }
    }

    fn make_stub(name: &str, language: &str) -> Atom {
        Atom {
            display_name: name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: String::new(),
            code_text: CodeText {
                lines_start: 0,
                lines_end: 0,
            },
            kind: "exec".to_string(),
            language: language.to_string(),
            extensions: BTreeMap::new(),
        }
    }

    fn set_status(atom: &mut Atom, status: &str) {
        atom.extensions.insert(
            "verification-status".to_string(),
            serde_json::Value::String(status.to_string()),
        );
    }

    fn get_status(atom: &Atom) -> Option<&str> {
        atom.extensions
            .get("verification-status")
            .and_then(|v| v.as_str())
    }

    fn mapping(from: &str, to: &str, confidence: &str) -> Mapping {
        Mapping {
            from: from.to_string(),
            to: to.to_string(),
            confidence: confidence.to_string(),
            method: None,
        }
    }

    fn get_records<'a>(atom: &'a Atom, field: &str) -> &'a Vec<serde_json::Value> {
        atom.extensions
            .get(field)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("missing {field} array"))
    }

    fn merge(maps: Vec<BTreeMap<String, Atom>>) -> (BTreeMap<String, Atom>, MergeStats) {
        merge_atom_maps(maps, None).expect("merge should succeed")
    }

    fn merge_mapped(
        maps: Vec<BTreeMap<String, Atom>>,
        mappings: &[Mapping],
    ) -> (BTreeMap<String, Atom>, MergeStats) {
        merge_atom_maps(maps, Some(mappings)).expect("merge should succeed")
    }

    // -----------------------------------------------------------------------
    // Atom merge tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_stub_replaced_by_real() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/helper()".to_string(),
            make_stub("helper", "rust"),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/helper()".to_string(),
            make_real_atom("helper", "src/lib.rs", "rust", "exec"),
        );

        let (merged, stats) = merge(vec![base, incoming]);

        assert_eq!(stats.stubs_replaced, 1);
        assert_eq!(stats.stubs_remaining, 0);
        assert_eq!(merged["probe:a/1.0/mod/helper()"].code_path, "src/lib.rs");
    }

    #[test]
    fn test_new_atoms_added() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/foo()".to_string(),
            make_real_atom("foo", "src/a.rs", "rust", "exec"),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:b/1.0/mod/bar()".to_string(),
            make_real_atom("bar", "src/b.rs", "rust", "exec"),
        );

        let (merged, stats) = merge(vec![base, incoming]);

        assert_eq!(stats.entries_added, 1);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_real_vs_real_conflict_keeps_base() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f()".to_string(),
            make_real_atom("f", "src/base.rs", "rust", "exec"),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/f()".to_string(),
            make_real_atom("f", "src/other.rs", "rust", "exec"),
        );

        let (merged, stats) = merge(vec![base, incoming]);

        assert_eq!(stats.conflicts, 1);
        assert_eq!(merged["probe:a/1.0/mod/f()"].code_path, "src/base.rs");
    }

    #[test]
    fn test_trailing_dot_normalization() {
        let mut base = BTreeMap::new();
        base.insert("probe:a/1.0/mod/f().".to_string(), make_stub("f", "rust"));

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/f()".to_string(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );

        let (merged, stats) = merge(vec![base, incoming]);

        assert_eq!(stats.keys_normalized, 1);
        assert_eq!(stats.stubs_replaced, 1);
        assert!(merged.contains_key("probe:a/1.0/mod/f()"));
        assert!(!merged.contains_key("probe:a/1.0/mod/f()."));
    }

    // P8: a post-normalization collision between distinct real atoms within
    // one input is producer error; collapsing a stub or an identical
    // (modulo records) duplicate is benign.
    #[test]
    fn test_normalization_collision_classification() {
        // Two distinct real atoms whose keys collide after normalization.
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/g()".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert(
            "probe:a/1.0/g().".to_string(),
            make_real_atom("g", "src/other.rs", "rust", "exec"),
        );
        let (out, changed, dropped) = normalize_atoms(atoms).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(changed, 1);
        assert_eq!(
            dropped,
            vec![(
                "probe:a/1.0/g().".to_string(),
                "probe:a/1.0/g()".to_string()
            )]
        );

        // A colliding stub is absorbed silently (stub replacement / drop).
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/g()".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert("probe:a/1.0/g().".to_string(), make_stub("g", "rust"));
        let (_, _, dropped) = normalize_atoms(atoms).unwrap();
        assert!(dropped.is_empty());

        // Identical duplicates collapse silently (dedup, no evidence lost).
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/g()".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert(
            "probe:a/1.0/g().".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        let (_, _, dropped) = normalize_atoms(atoms).unwrap();
        assert!(dropped.is_empty());

        // Duplicates differing only in correspondence records are the same
        // evidence: benign collapse, records unioned (P27).
        let mut with_record = make_real_atom("g", "src/lib.rs", "rust", "exec");
        assert!(push_record(
            &mut with_record,
            MAPS_TO,
            "probe:Pkg.g",
            "exact",
            None
        ));
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/g()".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert("probe:a/1.0/g().".to_string(), with_record);
        let (out, _, dropped) = normalize_atoms(atoms).unwrap();
        assert!(dropped.is_empty());
        assert_eq!(get_records(&out["probe:a/1.0/g()"], MAPS_TO).len(), 1);
    }

    // P8 reconciliation (merge-soundness PR 3b): merge re-enriches, so a
    // silently first-wins-selected atom would feed enrichment inside merge —
    // the same laundering path that makes the unary enrich boundary reject.
    // A distinct-real intra-input collision is therefore an error at merge
    // too, in the first input and in subsequent inputs alike.
    #[test]
    fn test_intra_input_distinct_real_collision_rejected() {
        let colliding = || {
            let mut m = BTreeMap::new();
            m.insert(
                "probe:a/1.0/g()".to_string(),
                make_real_atom("g", "src/lib.rs", "rust", "exec"),
            );
            m.insert(
                "probe:a/1.0/g().".to_string(),
                make_real_atom("g", "src/other.rs", "rust", "exec"),
            );
            m
        };
        let partner = || {
            let mut m = BTreeMap::new();
            m.insert(
                "probe:a/1.0/h()".to_string(),
                make_real_atom("h", "src/lib.rs", "rust", "exec"),
            );
            m
        };

        let err = merge_atom_maps(vec![colliding(), partner()], None).unwrap_err();
        assert!(err.contains("distinct real atom"), "{err}");
        assert!(err.contains("probe:a/1.0/g()."), "{err}");

        let err = merge_atom_maps(vec![partner(), colliding()], None).unwrap_err();
        assert!(err.contains("distinct real atom"), "{err}");

        // The raw staging path applies the same rule.
        let err = merge_atom_maps_raw(vec![colliding()], None).unwrap_err();
        assert!(err.contains("distinct real atom"), "{err}");

        // A collision-free merge is unaffected.
        let mut other = BTreeMap::new();
        other.insert(
            "probe:a/1.0/i()".to_string(),
            make_real_atom("i", "src/lib.rs", "rust", "exec"),
        );
        let (_, stats) = merge(vec![partner(), other]);
        assert_eq!(stats.conflicts, 0);
    }

    // P8: normalization strips *all* trailing dots, so it is a fixed point —
    // a repeated-dot alias ("g()..") collides with "g()" in the same pass and
    // cannot smuggle a distinct atom past the collision guard.
    #[test]
    fn test_repeated_trailing_dots_normalize_in_one_pass() {
        assert_eq!(normalize_code_name("g().."), "g()");
        assert_eq!(normalize_code_name("g()"), "g()");

        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/g()".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert(
            "probe:a/1.0/g()..".to_string(),
            make_real_atom("g", "src/other.rs", "rust", "exec"),
        );
        let (out, _, dropped) = normalize_atoms(atoms).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(
            dropped,
            vec![(
                "probe:a/1.0/g()..".to_string(),
                "probe:a/1.0/g()".to_string()
            )]
        );
    }

    #[test]
    fn test_cross_language_merge() {
        let mut rust_atoms = BTreeMap::new();
        rust_atoms.insert(
            "probe:dalek/4.1.3/scalar/add()".to_string(),
            make_real_atom("add", "src/scalar.rs", "rust", "exec"),
        );

        let mut lean_atoms = BTreeMap::new();
        lean_atoms.insert(
            "probe:Curve25519Dalek.Scalar.add".to_string(),
            make_real_atom("add", "Curve25519Dalek/Scalar.lean", "lean", "def"),
        );

        let (merged, stats) = merge(vec![rust_atoms, lean_atoms]);

        assert_eq!(stats.entries_added, 1);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged["probe:dalek/4.1.3/scalar/add()"].language, "rust");
        assert_eq!(merged["probe:Curve25519Dalek.Scalar.add"].language, "lean");
    }

    #[test]
    fn test_is_stub() {
        let stub = make_stub("f", "rust");
        assert!(stub.is_stub());

        let real = make_real_atom("f", "src/lib.rs", "rust", "exec");
        assert!(!real.is_stub());
    }

    #[test]
    fn test_extensions_preserved() {
        let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
        atom.extensions.insert(
            "dependencies-with-locations".to_string(),
            serde_json::json!([{"code-name": "probe:a/1.0/g()", "location": "inner", "line": 42}]),
        );

        let mut base = BTreeMap::new();
        base.insert("probe:a/1.0/f()".to_string(), atom);

        let (merged, _) = merge(vec![base]);

        let f = &merged["probe:a/1.0/f()"];
        assert!(f.extensions.contains_key("dependencies-with-locations"));
    }

    // -----------------------------------------------------------------------
    // Correspondence-record attachment (P13) — replaces the edge-injection
    // tests: `dependencies` is never modified by mappings (ADR-006).
    // -----------------------------------------------------------------------

    #[test]
    fn test_mappings_attach_records_not_edges() {
        let mut rust_atoms = BTreeMap::new();
        let mut rust_main = make_real_atom("main", "src/lib.rs", "rust", "exec");
        rust_main
            .dependencies
            .insert("probe:mycrate/1.0/reduce()".to_string());
        rust_atoms.insert("probe:mycrate/1.0/main()".to_string(), rust_main);
        rust_atoms.insert(
            "probe:mycrate/1.0/reduce()".to_string(),
            make_real_atom("reduce", "src/field.rs", "rust", "exec"),
        );

        let mut lean_atoms = BTreeMap::new();
        lean_atoms.insert(
            "probe:mycrate.field.reduce".to_string(),
            make_real_atom("reduce", "Field.lean", "lean", "exec"),
        );

        let mappings = vec![mapping(
            "probe:mycrate/1.0/reduce()",
            "probe:mycrate.field.reduce",
            "exact",
        )];

        let (merged, stats) = merge_mapped(vec![rust_atoms, lean_atoms], &mappings);

        // One maps-to record on the from atom, one mapped-from on the to atom.
        assert_eq!(stats.records_attached, 2);
        let from_atom = &merged["probe:mycrate/1.0/reduce()"];
        assert_eq!(
            get_records(from_atom, MAPS_TO),
            &vec![serde_json::json!({
                "target": "probe:mycrate.field.reduce", "confidence": "exact"
            })]
        );
        let to_atom = &merged["probe:mycrate.field.reduce"];
        assert_eq!(
            get_records(to_atom, MAPPED_FROM),
            &vec![serde_json::json!({
                "target": "probe:mycrate/1.0/reduce()", "confidence": "exact"
            })]
        );

        // Dependencies are never modified (ADR-006): main still depends only
        // on the Rust reduce, and no atom gained a cross-language edge.
        let main_atom = &merged["probe:mycrate/1.0/main()"];
        assert_eq!(
            main_atom.dependencies,
            BTreeSet::from(["probe:mycrate/1.0/reduce()".to_string()])
        );
        assert!(to_atom.dependencies.is_empty());
    }

    // P13: attachment is unconditional — a dangling target is warned about,
    // never skipped (an existence check would make attachment depend on the
    // surrounding key set and break the P4 mapping-compatibility law).
    #[test]
    fn test_dangling_target_still_attaches() {
        let mut rust_atoms = BTreeMap::new();
        rust_atoms.insert(
            "probe:a/1.0/encrypt()".to_string(),
            make_real_atom("encrypt", "src/crypto.rs", "rust", "exec"),
        );

        let mappings = vec![mapping(
            "probe:a/1.0/encrypt()",
            "probe:Ghost.encrypt",
            "exact",
        )];

        let (merged, stats) = merge_mapped(vec![rust_atoms], &mappings);

        assert_eq!(stats.records_attached, 1);
        let from_atom = &merged["probe:a/1.0/encrypt()"];
        assert_eq!(
            get_records(from_atom, MAPS_TO),
            &vec![serde_json::json!({
                "target": "probe:Ghost.encrypt", "confidence": "exact"
            })]
        );
        assert!(!merged.contains_key("probe:Ghost.encrypt"));
    }

    // P13/P27: attachment is set-like — re-applying the same mappings to an
    // already-annotated map is a no-op (record identity is the full triple).
    #[test]
    fn test_reapplication_is_noop() {
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/f()".to_string(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert(
            "probe:Pkg.f".to_string(),
            make_real_atom("f", "Pkg.lean", "lean", "def"),
        );

        let mappings = vec![mapping("probe:a/1.0/f()", "probe:Pkg.f", "exact")];

        let (first, stats) = merge_mapped(vec![atoms], &mappings);
        assert_eq!(stats.records_attached, 2);

        let (second, stats) = merge_mapped(vec![first.clone()], &mappings);
        assert_eq!(stats.records_attached, 0, "re-application is a no-op");
        assert_eq!(second, first);
    }

    // Same target with different confidence/method are distinct assertions:
    // both records are kept, sorted by the identity triple (P27, P14).
    #[test]
    fn test_distinct_confidence_records_both_kept_sorted() {
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/f()".to_string(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );

        let mut with_method = mapping("probe:a/1.0/f()", "probe:Pkg.f", "heuristic");
        with_method.method = Some("file+display-name".to_string());
        let mappings = vec![
            with_method,
            mapping("probe:a/1.0/f()", "probe:Pkg.f", "exact"),
            mapping("probe:a/1.0/f()", "probe:Other.f", "manual"),
        ];

        let (merged, stats) = merge_mapped(vec![atoms], &mappings);
        assert_eq!(stats.records_attached, 3);

        let records = get_records(&merged["probe:a/1.0/f()"], MAPS_TO);
        assert_eq!(
            records,
            &vec![
                serde_json::json!({"target": "probe:Other.f", "confidence": "manual"}),
                serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"}),
                serde_json::json!({"target": "probe:Pkg.f", "confidence": "heuristic",
                                   "method": "file+display-name"}),
            ]
        );
    }

    /// 1-to-many: a single `from` key with two `to` targets produces two
    /// maps-to records and a mapped-from record on each present target.
    #[test]
    fn test_one_to_many_mapping_produces_multiple_records() {
        let mut rust_atoms = BTreeMap::new();
        rust_atoms.insert(
            "probe:mycrate/1.0/encrypt()".to_string(),
            make_real_atom("encrypt", "src/crypto.rs", "rust", "exec"),
        );

        let mut lean_atoms = BTreeMap::new();
        lean_atoms.insert(
            "probe:AEADScheme.encrypt".to_string(),
            make_real_atom("encrypt", "AEAD.lean", "lean", "def"),
        );
        lean_atoms.insert(
            "probe:DetSEAlg.encrypt".to_string(),
            make_real_atom("encrypt", "DetSE.lean", "lean", "def"),
        );

        let mappings = vec![
            mapping(
                "probe:mycrate/1.0/encrypt()",
                "probe:AEADScheme.encrypt",
                "exact",
            ),
            mapping(
                "probe:mycrate/1.0/encrypt()",
                "probe:DetSEAlg.encrypt",
                "manual",
            ),
        ];

        let (merged, stats) = merge_mapped(vec![rust_atoms, lean_atoms], &mappings);

        assert_eq!(stats.records_attached, 4, "2 maps-to + 2 mapped-from");
        let records = get_records(&merged["probe:mycrate/1.0/encrypt()"], MAPS_TO);
        assert_eq!(records.len(), 2);
        assert_eq!(
            get_records(&merged["probe:AEADScheme.encrypt"], MAPPED_FROM),
            &vec![serde_json::json!({
                "target": "probe:mycrate/1.0/encrypt()", "confidence": "exact"
            })]
        );
    }

    // P8/P27: record targets arriving on input atoms are normalized like any
    // other code-name.
    #[test]
    fn test_record_targets_normalized() {
        let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
        atom.extensions.insert(
            MAPS_TO.to_string(),
            serde_json::json!([{"target": "probe:Pkg.f.", "confidence": "exact"}]),
        );
        let mut atoms = BTreeMap::new();
        atoms.insert("probe:a/1.0/f()".to_string(), atom);

        let (out, _, dropped) = normalize_atoms(atoms).unwrap();
        assert!(dropped.is_empty());
        assert_eq!(
            get_records(&out["probe:a/1.0/f()"], MAPS_TO),
            &vec![serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"})]
        );
    }

    // P8/P27: records that become equal only after target normalization are
    // one assertion — they collapse to a single canonical record.
    #[test]
    fn test_records_equal_after_target_normalization_collapse() {
        let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
        atom.extensions.insert(
            MAPS_TO.to_string(),
            serde_json::json!([
                {"target": "probe:Pkg.f.", "confidence": "exact"},
                {"target": "probe:Pkg.f", "confidence": "exact"}
            ]),
        );
        let mut atoms = BTreeMap::new();
        atoms.insert("probe:a/1.0/f()".to_string(), atom);

        let (out, _, _) = normalize_atoms(atoms).unwrap();
        assert_eq!(
            get_records(&out["probe:a/1.0/f()"], MAPS_TO),
            &vec![serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"})]
        );
    }

    // P27 canonical form: `method: ""` names no matching method — it is
    // canonicalized to absent, so a record with and without the empty method
    // is one identity and collapses. Without this, identity (the triple) and
    // dedup disagree: a tied-key twin shields exact duplicates from
    // collapsing, and the set union grows on self-merge.
    #[test]
    fn test_empty_method_canonicalized_to_absent() {
        let r = serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"});
        let s = serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact", "method": ""});

        // Input-atom path: both encodings collapse to the canonical record.
        let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
        atom.extensions
            .insert(MAPS_TO.to_string(), serde_json::json!([r, s]));
        let mut atoms = BTreeMap::new();
        atoms.insert("probe:a/1.0/f()".to_string(), atom.clone());
        let (out, _, _) = normalize_atoms(atoms).unwrap();
        assert_eq!(
            get_records(&out["probe:a/1.0/f()"], MAPS_TO),
            &vec![serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"})]
        );

        // Self-merge is idempotent on the record set (no duplicate growth).
        let mut atoms = BTreeMap::new();
        atoms.insert("probe:a/1.0/f()".to_string(), atom);
        let (merged, _) = merge(vec![atoms.clone(), atoms]);
        assert_eq!(get_records(&merged["probe:a/1.0/f()"], MAPS_TO).len(), 1);

        // Mapping path: an empty method attaches the same record as no method.
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/g()".to_string(),
            make_real_atom("g", "src/lib.rs", "rust", "exec"),
        );
        let mut with_empty = mapping("probe:a/1.0/g()", "probe:Pkg.g", "exact");
        with_empty.method = Some(String::new());
        let (first, stats) = merge_mapped(vec![base], &[with_empty]);
        assert_eq!(stats.records_attached, 1);
        assert_eq!(
            get_records(&first["probe:a/1.0/g()"], MAPS_TO),
            &vec![serde_json::json!({"target": "probe:Pkg.g", "confidence": "exact"})]
        );
        let (_, stats) = merge_mapped(
            vec![first],
            &[mapping("probe:a/1.0/g()", "probe:Pkg.g", "exact")],
        );
        assert_eq!(stats.records_attached, 0, "same identity, no re-attachment");
    }

    // P4 regression: records tying on the identity triple must not make the
    // merge result grouping-dependent. With the empty-method twin encoding,
    // stub replacement used to reverse which record sequence was appended
    // first, and the stable tie sort preserved that order.
    #[test]
    fn test_tied_identity_records_keep_associativity() {
        let key = "probe:a/1.0/f()".to_string();
        let with_records = |mut atom: Atom, records: serde_json::Value| {
            atom.extensions.insert(MAPS_TO.to_string(), records);
            atom
        };

        let mut a = BTreeMap::new();
        a.insert(
            key.clone(),
            with_records(
                make_stub("f", "rust"),
                serde_json::json!([{"target": "probe:Pkg.f", "confidence": "exact"}]),
            ),
        );
        let mut b = BTreeMap::new();
        b.insert(
            key.clone(),
            with_records(
                make_stub("f", "rust"),
                serde_json::json!([{"target": "probe:Pkg.f", "confidence": "exact", "method": ""}]),
            ),
        );
        let mut c = BTreeMap::new();
        c.insert(
            key.clone(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );

        let (flat, _) = merge(vec![a.clone(), b.clone(), c.clone()]);
        let (ab, _) = merge(vec![a.clone(), b.clone()]);
        let (left, _) = merge(vec![ab, c.clone()]);
        let (bc, _) = merge(vec![b, c]);
        let (right, _) = merge(vec![a, bc]);

        assert_eq!(left, flat);
        assert_eq!(right, flat);
        assert_eq!(
            get_records(&flat[&key], MAPS_TO),
            &vec![serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"})]
        );
    }

    // ADR-006 fail-closed: a malformed correspondence-record shape rejects
    // the offending input — merge re-emits these fields (the executable
    // schema constrains them), and the union machinery is only lossless on
    // canonical records (a non-array field silently swallowing a union would
    // make the result grouping-dependent, breaking P4). The error names the
    // input position.
    #[test]
    fn test_malformed_record_shapes_rejected() {
        let with_field = |value: serde_json::Value| {
            let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
            atom.extensions.insert(MAPS_TO.to_string(), value);
            let mut atoms = BTreeMap::new();
            atoms.insert("probe:a/1.0/f()".to_string(), atom);
            atoms
        };
        let ok = || {
            let mut atoms = BTreeMap::new();
            atoms.insert(
                "probe:a/1.0/g()".to_string(),
                make_real_atom("g", "src/lib.rs", "rust", "exec"),
            );
            atoms
        };

        for (bad, why) in [
            (serde_json::json!("not-an-array"), "must be an array"),
            (
                serde_json::json!([42]),
                "not a correspondence-record object",
            ),
            (
                serde_json::json!([{"target": "probe:Pkg.f", "confidence": "exact", "note": "x"}]),
                "unexpected field",
            ),
            (
                serde_json::json!([{"target": "probe:Pkg.f", "confidence": "high"}]),
                "invalid confidence",
            ),
            (
                serde_json::json!([{"confidence": "exact"}]),
                "missing a string \"target\"",
            ),
            (
                serde_json::json!([{"target": "probe:Pkg.f", "confidence": "exact", "method": 3}]),
                "non-string method",
            ),
        ] {
            let err = merge_atom_maps(vec![ok(), with_field(bad.clone())], None).unwrap_err();
            assert!(err.contains(why), "{bad}: {err}");
            assert!(err.contains("input #2 of 2"), "{err}");
        }
    }

    // Fail-closed mapping validation: an out-of-vocabulary confidence rejects
    // the merge (merge writes `confidence` into the records it emits; the
    // executable schema constrains it).
    #[test]
    fn test_invalid_mapping_confidence_rejected() {
        let mut atoms = BTreeMap::new();
        atoms.insert(
            "probe:a/1.0/f()".to_string(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );
        let err = merge_atom_maps(
            vec![atoms],
            Some(&[mapping("probe:a/1.0/f()", "probe:Pkg.f", "high")]),
        )
        .unwrap_err();
        assert!(err.contains("invalid confidence"), "{err}");
    }

    // -----------------------------------------------------------------------
    // Record preservation through every equal-key resolution (P27)
    // -----------------------------------------------------------------------

    #[test]
    fn test_records_preserved_through_every_equal_key_case() {
        let annotate = |mut atom: Atom, target: &str| {
            assert!(push_record(&mut atom, MAPS_TO, target, "exact", None));
            atom
        };
        let record_of = |target: &str| serde_json::json!({"target": target, "confidence": "exact"});
        let key = "probe:a/1.0/f()".to_string();

        // Stub replacement: a record-carrying stub replaced by a record-less
        // real atom keeps the stub's records.
        let mut base = BTreeMap::new();
        base.insert(key.clone(), annotate(make_stub("f", "rust"), "probe:Pkg.f"));
        let mut incoming = BTreeMap::new();
        incoming.insert(
            key.clone(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );
        let (merged, stats) = merge(vec![base, incoming]);
        assert_eq!(stats.stubs_replaced, 1);
        assert!(!merged[&key].is_stub());
        assert_eq!(
            get_records(&merged[&key], MAPS_TO),
            &vec![record_of("probe:Pkg.f")]
        );

        // Real-vs-real first-wins: a record-less base winner unions the
        // annotated loser's records.
        let mut base = BTreeMap::new();
        base.insert(
            key.clone(),
            make_real_atom("f", "src/base.rs", "rust", "exec"),
        );
        let mut incoming = BTreeMap::new();
        incoming.insert(
            key.clone(),
            annotate(
                make_real_atom("f", "src/other.rs", "rust", "exec"),
                "probe:Pkg.f",
            ),
        );
        let (merged, stats) = merge(vec![base, incoming]);
        assert_eq!(stats.conflicts, 1);
        assert_eq!(merged[&key].code_path, "src/base.rs");
        assert_eq!(
            get_records(&merged[&key], MAPS_TO),
            &vec![record_of("probe:Pkg.f")]
        );

        // Stub-vs-stub: records union onto the kept base stub.
        let mut base = BTreeMap::new();
        base.insert(key.clone(), annotate(make_stub("f", "rust"), "probe:Pkg.f"));
        let mut incoming = BTreeMap::new();
        incoming.insert(
            key.clone(),
            annotate(make_stub("f", "rust"), "probe:Other.f"),
        );
        let (merged, _) = merge(vec![base, incoming]);
        assert_eq!(
            get_records(&merged[&key], MAPS_TO),
            &vec![record_of("probe:Other.f"), record_of("probe:Pkg.f")]
        );

        // Real-vs-stub: the kept real atom unions the losing stub's records.
        let mut base = BTreeMap::new();
        base.insert(
            key.clone(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );
        let mut incoming = BTreeMap::new();
        incoming.insert(key.clone(), annotate(make_stub("f", "rust"), "probe:Pkg.f"));
        let (merged, _) = merge(vec![base, incoming]);
        assert!(!merged[&key].is_stub());
        assert_eq!(
            get_records(&merged[&key], MAPS_TO),
            &vec![record_of("probe:Pkg.f")]
        );

        // Intra-input post-normalization collision (benign: stub collapse):
        // records union across the collision.
        let mut atoms = BTreeMap::new();
        atoms.insert(
            key.clone(),
            make_real_atom("f", "src/lib.rs", "rust", "exec"),
        );
        atoms.insert(
            "probe:a/1.0/f().".to_string(),
            annotate(make_stub("f", "rust"), "probe:Pkg.f"),
        );
        let (merged, _) = merge(vec![atoms]);
        assert!(!merged[&key].is_stub());
        assert_eq!(
            get_records(&merged[&key], MAPS_TO),
            &vec![record_of("probe:Pkg.f")]
        );
    }

    // -----------------------------------------------------------------------
    // Algebraic laws (P4/P5), tested against merge_atom_maps — the operation
    // §4 of the plan defines (μ, and F_M ∘ μ with mappings).
    // -----------------------------------------------------------------------

    /// Builds three overlapping on- and off-carrier inputs plus mappings that
    /// exercise stub replacement, real-vs-real conflicts, and dangling-then-
    /// resolved record targets across groupings.
    #[allow(clippy::type_complexity)]
    fn law_fixtures() -> (
        BTreeMap<String, Atom>,
        BTreeMap<String, Atom>,
        BTreeMap<String, Atom>,
        Vec<Mapping>,
    ) {
        let mut a = BTreeMap::new();
        let mut f = make_real_atom("f", "src/a.rs", "rust", "exec");
        set_status(&mut f, "verified");
        f.dependencies.insert("probe:a/1.0/g()".to_string());
        a.insert("probe:a/1.0/f()".to_string(), f);
        a.insert("probe:a/1.0/g()".to_string(), make_stub("g", "rust"));

        let mut b = BTreeMap::new();
        let mut g = make_real_atom("g", "src/b.rs", "rust", "exec");
        set_status(&mut g, "failed");
        b.insert("probe:a/1.0/g()".to_string(), g);
        let mut f_dup = make_real_atom("f", "src/b_dup.rs", "rust", "exec");
        set_status(&mut f_dup, "verified");
        b.insert("probe:a/1.0/f()".to_string(), f_dup);

        let mut c = BTreeMap::new();
        let mut lean_g = make_real_atom("g", "Pkg.lean", "lean", "def");
        set_status(&mut lean_g, "verified");
        c.insert("probe:Pkg.g".to_string(), lean_g);

        let mappings = vec![
            mapping("probe:a/1.0/g()", "probe:Pkg.g", "exact"),
            mapping("probe:a/1.0/f()", "probe:Pkg.f", "manual"), // dangling target
        ];

        (a, b, c, mappings)
    }

    // P4 law 1 + law 3: μ (with F_M) is associative across groupings.
    #[test]
    fn test_associativity_with_mappings_across_groupings() {
        let (a, b, c, m) = law_fixtures();

        let (flat, _) = merge_mapped(vec![a.clone(), b.clone(), c.clone()], &m);
        let (ab, _) = merge_mapped(vec![a.clone(), b.clone()], &m);
        let (left, _) = merge_mapped(vec![ab, c.clone()], &m);
        let (bc, _) = merge_mapped(vec![b, c], &m);
        let (right, _) = merge_mapped(vec![a, bc], &m);

        assert_eq!(left, flat, "((A·B)·C) == (A·B·C)");
        assert_eq!(right, flat, "(A·(B·C)) == (A·B·C)");
    }

    // P4 argument detail: enriching an intermediate merge never changes which
    // atom a later conflict selects — enrichment only rewrites
    // verified ↔ transitively-verified, and conflict resolution keys on stub
    // classification and base data, not on derived labels.
    #[test]
    fn test_intermediate_enrichment_does_not_change_selected_base_data() {
        // A's f is a verified leaf: the intermediate μ(A, B) enriches it to
        // transitively-verified before C's conflicting f arrives.
        let mut a = BTreeMap::new();
        let mut f = make_real_atom("f", "src/a.rs", "rust", "exec");
        set_status(&mut f, "verified");
        a.insert("probe:a/1.0/f()".to_string(), f);

        let mut b = BTreeMap::new();
        b.insert(
            "probe:b/1.0/other()".to_string(),
            make_real_atom("other", "src/b.rs", "rust", "exec"),
        );

        let mut c = BTreeMap::new();
        let mut f_c = make_real_atom("f", "src/c.rs", "rust", "exec");
        set_status(&mut f_c, "verified");
        c.insert("probe:a/1.0/f()".to_string(), f_c);

        let (ab, _) = merge(vec![a.clone(), b.clone()]);
        assert_eq!(
            get_status(&ab["probe:a/1.0/f()"]),
            Some("transitively-verified"),
            "intermediate enrichment rewrote the label"
        );

        let (nested, stats) = merge(vec![ab, c.clone()]);
        assert_eq!(stats.conflicts, 1);
        let (flat, _) = merge(vec![a, b, c]);

        assert_eq!(nested, flat);
        assert_eq!(nested["probe:a/1.0/f()"].code_path, "src/a.rs");
    }

    // P5: μ(A, ∅) = A exactly on the carrier (records included).
    #[test]
    fn test_identity_exact_on_carrier() {
        let (a, b, _, m) = law_fixtures();
        // Any merge output is on the carrier.
        let (carrier, _) = merge_mapped(vec![a, b], &m);

        let (left, _) = merge(vec![carrier.clone(), BTreeMap::new()]);
        assert_eq!(left, carrier, "μ(A, ∅) = A on the carrier");

        let (right, _) = merge(vec![BTreeMap::new(), carrier.clone()]);
        assert_eq!(right, carrier, "μ(∅, A) = A on the carrier");
    }

    // P5: a legacy (off-carrier) input is first brought onto the carrier —
    // μ(A, ∅) = enrich(normalize(A)).
    #[test]
    fn test_identity_up_to_preparation_on_legacy() {
        let mut legacy = BTreeMap::new();
        let mut f = make_real_atom("f", "src/a.rs", "rust", "exec");
        // Stale label and a dotted dependency alias.
        set_status(&mut f, "transitively-verified");
        f.dependencies.insert("probe:a/1.0/g().".to_string());
        legacy.insert("probe:a/1.0/f()".to_string(), f);
        let mut g = make_real_atom("g", "src/g.rs", "rust", "exec");
        set_status(&mut g, "failed");
        legacy.insert("probe:a/1.0/g().".to_string(), g);

        let (merged, _) = merge(vec![legacy.clone(), BTreeMap::new()]);
        let (prepared, _) = prepare_atoms(legacy).unwrap();
        assert_eq!(merged, prepared, "μ(A, ∅) = enrich(normalize(A)) on legacy");
        assert_eq!(get_status(&merged["probe:a/1.0/f()"]), Some("verified"));
        assert!(merged.contains_key("probe:a/1.0/g()"));
    }

    // P4 law 2: commutativity for disjoint keys.
    #[test]
    fn test_commutativity_disjoint_keys() {
        let (a, _, c, m) = law_fixtures(); // a and c have disjoint keys
        let (left, _) = merge_mapped(vec![a.clone(), c.clone()], &m);
        let (right, _) = merge_mapped(vec![c, a], &m);
        assert_eq!(left, right);
    }

    // P4 law 3: F_M(F_M(A)) = F_M(A) and F_M(μ(A, B)) = μ(F_M(A), F_M(B)).
    #[test]
    fn test_mapping_compatibility_laws() {
        let (a, b, _, m) = law_fixtures();

        // Idempotence: applying F_M to an already-mapped merge is a no-op.
        let (mapped, _) = merge_mapped(vec![a.clone(), b.clone()], &m);
        let (remapped, stats) = merge_mapped(vec![mapped.clone()], &m);
        assert_eq!(remapped, mapped, "F_M(F_M(A)) = F_M(A)");
        assert_eq!(stats.records_attached, 0);

        // Compatibility: F_M(μ(A, B)) = μ(F_M(A), F_M(B)). Records attached
        // per input (dangling targets included, P13) union through the plain
        // merge to the same result as attaching after.
        let (fa, _) = merge_mapped(vec![a.clone()], &m);
        let (fb, _) = merge_mapped(vec![b.clone()], &m);
        let (rhs, _) = merge(vec![fa, fb]);
        let (lhs, _) = merge_mapped(vec![a, b], &m);
        assert_eq!(lhs, rhs, "F_M(μ(A, B)) = μ(F_M(A), F_M(B))");
    }

    // -----------------------------------------------------------------------
    // Merge re-enriches (μ includes recomputation); the raw path defers it
    // -----------------------------------------------------------------------

    // Stub resolution can invalidate labels computed at extract time: a
    // verified atom whose stub dependency resolves to a failed real atom must
    // not stay promotable, and a verified leaf is promoted.
    #[test]
    fn test_merge_recomputes_enrichment() {
        let mut a = BTreeMap::new();
        let mut f = make_real_atom("f", "src/a.rs", "rust", "exec");
        set_status(&mut f, "transitively-verified"); // stale: dep was a stub
        f.dependencies.insert("probe:a/1.0/g()".to_string());
        a.insert("probe:a/1.0/f()".to_string(), f);
        a.insert("probe:a/1.0/g()".to_string(), make_stub("g", "rust"));
        let mut leaf = make_real_atom("leaf", "src/a.rs", "rust", "exec");
        set_status(&mut leaf, "verified");
        a.insert("probe:a/1.0/leaf()".to_string(), leaf);

        let mut b = BTreeMap::new();
        let mut g = make_real_atom("g", "src/b.rs", "rust", "exec");
        set_status(&mut g, "failed");
        b.insert("probe:a/1.0/g()".to_string(), g);

        let (merged, stats) = merge(vec![a, b]);
        assert_eq!(
            get_status(&merged["probe:a/1.0/f()"]),
            Some("verified"),
            "stale transitively-verified downgraded after stub resolution"
        );
        assert_eq!(
            get_status(&merged["probe:a/1.0/leaf()"]),
            Some("transitively-verified"),
            "clean verified leaf promoted"
        );
        assert_eq!(stats.enriched_local, 1);
        assert_eq!(stats.enriched_transitive, 1);
    }

    // The raw staging primitive skips recomputation only: same merge result,
    // stale derived statuses left in place (documented off-carrier output).
    #[test]
    fn test_raw_path_defers_enrichment() {
        let mut a = BTreeMap::new();
        let mut leaf = make_real_atom("leaf", "src/a.rs", "rust", "exec");
        set_status(&mut leaf, "verified");
        a.insert("probe:a/1.0/leaf()".to_string(), leaf);

        let (raw, stats) = merge_atom_maps_raw(vec![a.clone()], None).unwrap();
        assert_eq!(
            get_status(&raw["probe:a/1.0/leaf()"]),
            Some("verified"),
            "raw path must not promote"
        );
        assert_eq!(stats.enriched_transitive, 0);
        assert_eq!(stats.enriched_local, 0);

        let (enriched, _) = merge(vec![a]);
        assert_eq!(
            get_status(&enriched["probe:a/1.0/leaf()"]),
            Some("transitively-verified")
        );
    }

    // §3d (thirteenth review): per-input normalization runs *before* conflict
    // resolution — the ordering selects evidence. With input A carrying
    // f [verified] → "g." and a failed real "g.", and input B carrying a
    // verified real "g", A's failed g must win under P6 and f must stay
    // blocked. Single-input alias tests cannot catch a wrong post-union
    // normalization order, so this runs two inputs through merge_atom_maps.
    #[test]
    fn test_two_input_dotted_alias_evidence_selection() {
        let mut a = BTreeMap::new();
        let mut f = make_real_atom("f", "src/a.rs", "rust", "exec");
        set_status(&mut f, "verified");
        f.dependencies.insert("probe:a/1.0/g().".to_string());
        a.insert("probe:a/1.0/f()".to_string(), f);
        let mut g_failed = make_real_atom("g", "src/a.rs", "rust", "exec");
        set_status(&mut g_failed, "failed");
        a.insert("probe:a/1.0/g().".to_string(), g_failed);

        let mut b = BTreeMap::new();
        let mut g_verified = make_real_atom("g", "src/b.rs", "rust", "exec");
        set_status(&mut g_verified, "verified");
        b.insert("probe:a/1.0/g()".to_string(), g_verified);

        let (merged, stats) = merge(vec![a, b]);

        // A's failed g is the selected evidence (normalized before B's g
        // arrived, then first-wins).
        assert_eq!(stats.conflicts, 1);
        let g = &merged["probe:a/1.0/g()"];
        assert_eq!(g.code_path, "src/a.rs");
        assert_eq!(get_status(g), Some("failed"));
        // f reaches the failed seed: blocked, not promoted.
        assert_eq!(get_status(&merged["probe:a/1.0/f()"]), Some("verified"));
    }

    // -----------------------------------------------------------------------
    // File-level entry points: authority validation on both paths
    // -----------------------------------------------------------------------

    fn write_json(dir: &std::path::Path, name: &str, value: &serde_json::Value) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
        path
    }

    fn plain_atoms_envelope(data: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "schema": "probe-verus/atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "atomize"},
            "source": {"repo": "r", "commit": "c", "language": "rust",
                       "package": "p", "package-version": "1.0"},
            "timestamp": "2026-01-01T00:00:00Z",
            "data": data
        })
    }

    // §3c: raw means skip recomputation, not skip authority validation — the
    // raw staging path rejects both projection formats like the public entry
    // points (ADR-006).
    #[test]
    fn test_raw_path_rejects_both_projection_formats() {
        let dir = tempfile::tempdir().unwrap();
        let ok = write_json(
            dir.path(),
            "ok.json",
            &plain_atoms_envelope(serde_json::json!({})),
        );

        let projected = serde_json::json!({
            "schema": "probe/projected-atoms",
            "schema-version": "3.1",
            "tool": {"name": "probe", "version": "0.5.0", "command": "project"},
            "inputs": [{"schema": "probe-verus/atoms",
                        "source": {"repo": "r", "commit": "c", "language": "rust",
                                   "package": "p", "package-version": "1.0"}}],
            "projection": {"mappings-file": "m.json", "seeds": 1, "forward-depth": 1,
                           "reverse-depth": 0, "atoms-in": 2, "atoms-out": 1, "deps-trimmed": 1},
            "timestamp": "2026-01-01T00:00:00Z",
            "data": {}
        });
        let projected = write_json(dir.path(), "projected.json", &projected);

        let mut legacy = plain_atoms_envelope(serde_json::json!({}));
        legacy["projection"] = serde_json::Value::Null;
        let legacy = write_json(dir.path(), "legacy.json", &legacy);

        let err = merge_atom_files_raw(&[ok.as_path(), projected.as_path()], None).unwrap_err();
        assert!(err.contains("projected input"), "{err}");

        let err = merge_atom_files_raw(&[ok.as_path(), legacy.as_path()], None).unwrap_err();
        assert!(err.contains("legacy projection"), "{err}");
    }

    // ADR-006 Decision 2: merge is a recomputation boundary — an out-of-enum
    // status-origin marker fails closed on both file-level paths.
    #[test]
    fn test_file_paths_reject_invalid_status_origin() {
        let dir = tempfile::tempdir().unwrap();
        let bad = plain_atoms_envelope(serde_json::json!({
            "probe:a/1.0/f()": {
                "display-name": "f", "dependencies": [], "code-module": "",
                "code-path": "src/lib.rs",
                "code-text": {"lines-start": 1, "lines-end": 2},
                "kind": "exec", "language": "rust",
                "status-origin": "graph-taint"
            }
        }));
        let bad = write_json(dir.path(), "bad.json", &bad);

        let err = merge_atom_files(&[bad.as_path()], None).unwrap_err();
        assert!(err.contains("invalid status-origin"), "{err}");
        let err = merge_atom_files_raw(&[bad.as_path()], None).unwrap_err();
        assert!(err.contains("invalid status-origin"), "{err}");
    }

    #[test]
    fn test_recursive_merge_flattens_provenance() {
        use crate::types::{load_atom_file, MergedAtomEnvelope, Tool};
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();

        let file_a = dir.path().join("a.json");
        let file_b = dir.path().join("b.json");
        let atom_a = make_real_atom("foo", "src/a.rs", "rust", "exec");
        let atom_b = make_real_atom("bar", "src/b.rs", "lean", "def");

        let mut data_a = BTreeMap::new();
        data_a.insert("probe:a/1.0/mod/foo()".to_string(), atom_a);

        let mut data_b = BTreeMap::new();
        data_b.insert("probe:b/1.0/mod/bar()".to_string(), atom_b);

        let envelope_a = serde_json::json!({
            "schema": "verus-analyzer/atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe", "version": "0.1.0", "command": "extract"},
            "source": {"repo": "repo-a", "commit": "aaa", "language": "rust", "package": "pkg-a", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": data_a
        });
        let envelope_b = serde_json::json!({
            "schema": "lean-analyzer/atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe", "version": "0.1.0", "command": "extract"},
            "source": {"repo": "repo-b", "commit": "bbb", "language": "lean", "package": "pkg-b", "package-version": "2.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": data_b
        });

        std::fs::File::create(&file_a)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_a)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        std::fs::File::create(&file_b)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_b)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let (atoms_a, prov_a) = load_atom_file(&file_a).unwrap();
        let (atoms_b, prov_b) = load_atom_file(&file_b).unwrap();
        assert_eq!(prov_a.len(), 1);
        assert_eq!(prov_a[0].source.package, "pkg-a");
        assert_eq!(prov_b.len(), 1);
        assert_eq!(prov_b[0].source.package, "pkg-b");

        let (merged_data, _stats) = merge(vec![atoms_a, atoms_b]);

        let mut all_prov = Vec::new();
        all_prov.extend(prov_a);
        all_prov.extend(prov_b);

        let merged_envelope = MergedAtomEnvelope {
            schema: "probe/merged-atoms".to_string(),
            schema_version: "3.1".to_string(),
            tool: Tool {
                name: "probe".to_string(),
                version: "0.1.0".to_string(),
                command: "merge".to_string(),
            },
            inputs: all_prov,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            data: merged_data,
        };

        let merged_file = dir.path().join("merged_ab.json");
        std::fs::File::create(&merged_file)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&merged_envelope)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let file_c = dir.path().join("c.json");
        let atom_c = make_real_atom("baz", "src/c.rs", "rust", "exec");
        let mut data_c = BTreeMap::new();
        data_c.insert("probe:c/1.0/mod/baz()".to_string(), atom_c);

        let envelope_c = serde_json::json!({
            "schema": "verus-analyzer/atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe", "version": "0.1.0", "command": "extract"},
            "source": {"repo": "repo-c", "commit": "ccc", "language": "rust", "package": "pkg-c", "package-version": "3.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": data_c
        });
        std::fs::File::create(&file_c)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_c)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let (atoms_merged, prov_merged) = load_atom_file(&merged_file).unwrap();
        let (atoms_c, prov_c) = load_atom_file(&file_c).unwrap();

        assert_eq!(
            prov_merged.len(),
            2,
            "merged file provenance should be flattened"
        );
        assert_eq!(prov_c.len(), 1);

        let packages: Vec<&str> = prov_merged
            .iter()
            .map(|p| p.source.package.as_str())
            .collect();
        assert!(packages.contains(&"pkg-a"));
        assert!(packages.contains(&"pkg-b"));
        assert_eq!(prov_c[0].source.package, "pkg-c");

        let (final_data, _) = merge(vec![atoms_merged, atoms_c]);
        let mut final_prov = Vec::new();
        final_prov.extend(prov_merged);
        final_prov.extend(prov_c);

        assert_eq!(final_data.len(), 3);
        assert_eq!(
            final_prov.len(),
            3,
            "final provenance should have all 3 original sources"
        );

        let final_packages: Vec<&str> = final_prov
            .iter()
            .map(|p| p.source.package.as_str())
            .collect();
        assert!(final_packages.contains(&"pkg-a"));
        assert!(final_packages.contains(&"pkg-b"));
        assert!(final_packages.contains(&"pkg-c"));
    }

    // -----------------------------------------------------------------------
    // Generic merge tests (specs/proofs)
    // -----------------------------------------------------------------------

    #[test]
    fn test_generic_last_wins_on_conflict() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f()".to_string(),
            serde_json::json!({"verified": false, "status": "failure"}),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:a/1.0/mod/f()".to_string(),
            serde_json::json!({"verified": true, "status": "success"}),
        );

        let (merged, stats) = merge_generic_maps(vec![base, incoming]);

        assert_eq!(stats.conflicts, 1);
        assert_eq!(merged["probe:a/1.0/mod/f()"]["verified"], true);
        assert_eq!(merged["probe:a/1.0/mod/f()"]["status"], "success");
    }

    #[test]
    fn test_generic_new_entries_added() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f()".to_string(),
            serde_json::json!({"specified": true}),
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "probe:b/1.0/mod/g()".to_string(),
            serde_json::json!({"specified": false}),
        );

        let (merged, stats) = merge_generic_maps(vec![base, incoming]);

        assert_eq!(stats.entries_added, 1);
        assert_eq!(stats.conflicts, 0);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_generic_trailing_dot_normalization() {
        let mut base = BTreeMap::new();
        base.insert(
            "probe:a/1.0/mod/f().".to_string(),
            serde_json::json!({"specified": true}),
        );

        let (normalized, stats) = merge_generic_maps(vec![base]);

        assert_eq!(stats.keys_normalized, 1);
        assert!(normalized.contains_key("probe:a/1.0/mod/f()"));
        assert!(!normalized.contains_key("probe:a/1.0/mod/f()."));
    }

    #[test]
    fn test_generic_recursive_merge_flattens_provenance() {
        use crate::types::{load_generic_file, MergedGenericEnvelope, SchemaCategory, Tool};
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();

        let file_a = dir.path().join("specs_a.json");
        let file_b = dir.path().join("specs_b.json");

        let envelope_a = serde_json::json!({
            "schema": "probe-verus/specs",
            "schema-version": "3.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "specify"},
            "source": {"repo": "repo-a", "commit": "aaa", "language": "rust", "package": "pkg-a", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {
                "probe:a/1.0/mod/f()": {"specified": true, "has_requires": true, "has_ensures": false}
            }
        });
        let envelope_b = serde_json::json!({
            "schema": "probe-lean/specs",
            "schema-version": "3.0",
            "tool": {"name": "probe-lean", "version": "1.0.0", "command": "specify"},
            "source": {"repo": "repo-b", "commit": "bbb", "language": "lean", "package": "pkg-b", "package-version": "2.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {
                "probe:b/1.0/mod/g()": {"specified": false}
            }
        });

        std::fs::File::create(&file_a)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_a)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        std::fs::File::create(&file_b)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&envelope_b)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        let (data_a, prov_a, cat_a) = load_generic_file(&file_a).unwrap();
        let (data_b, prov_b, cat_b) = load_generic_file(&file_b).unwrap();
        assert_eq!(cat_a, SchemaCategory::Specs);
        assert_eq!(cat_b, SchemaCategory::Specs);

        let (merged_data, _) = merge_generic_maps(vec![data_a, data_b]);

        let mut all_prov = Vec::new();
        all_prov.extend(prov_a);
        all_prov.extend(prov_b);

        let merged_envelope = MergedGenericEnvelope {
            schema: "probe/merged-specs".to_string(),
            schema_version: "3.1".to_string(),
            tool: Tool {
                name: "probe".to_string(),
                version: "0.1.0".to_string(),
                command: "merge".to_string(),
            },
            inputs: all_prov,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            data: merged_data,
        };

        let merged_file = dir.path().join("merged_specs.json");
        std::fs::File::create(&merged_file)
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&merged_envelope)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

        // Load the merged file back and verify provenance is flattened.
        let (_data, prov, cat) = load_generic_file(&merged_file).unwrap();
        assert_eq!(cat, SchemaCategory::Specs);
        assert_eq!(
            prov.len(),
            2,
            "merged specs should carry both original provenance entries"
        );

        let packages: Vec<&str> = prov.iter().map(|p| p.source.package.as_str()).collect();
        assert!(packages.contains(&"pkg-a"));
        assert!(packages.contains(&"pkg-b"));
    }

    #[test]
    fn test_category_detection() {
        use crate::types::detect_category;

        assert_eq!(
            detect_category("probe-verus/atoms"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe-lean/enriched-atoms"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe/merged-atoms"),
            Some(SchemaCategory::Atoms)
        );
        // `*/extract` envelopes (probe-aeneas, probe-leanblueprint) are atoms.
        assert_eq!(
            detect_category("probe-leanblueprint/extract"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe-aeneas/extract"),
            Some(SchemaCategory::Atoms)
        );
        assert_eq!(
            detect_category("probe-verus/specs"),
            Some(SchemaCategory::Specs)
        );
        assert_eq!(
            detect_category("probe-lean/specs"),
            Some(SchemaCategory::Specs)
        );
        assert_eq!(
            detect_category("probe/merged-specs"),
            Some(SchemaCategory::Specs)
        );
        assert_eq!(
            detect_category("probe-verus/proofs"),
            Some(SchemaCategory::Proofs)
        );
        assert_eq!(
            detect_category("probe-lean/proofs"),
            Some(SchemaCategory::Proofs)
        );
        assert_eq!(
            detect_category("probe/merged-proofs"),
            Some(SchemaCategory::Proofs)
        );
        assert_eq!(detect_category("probe-verus/stubs"), None);
        assert_eq!(detect_category("something-else"), None);
    }

    #[test]
    fn test_category_mismatch_detected_by_loader() {
        use crate::types::load_generic_file;
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();

        let specs_file = dir.path().join("specs.json");
        let atoms_file = dir.path().join("atoms.json");

        let specs = serde_json::json!({
            "schema": "probe-verus/specs",
            "schema-version": "3.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "specify"},
            "source": {"repo": "r", "commit": "c", "language": "rust", "package": "p", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {"probe:a/1.0/f()": {"specified": true}}
        });
        let atoms = serde_json::json!({
            "schema": "probe-verus/atoms",
            "schema-version": "3.0",
            "tool": {"name": "probe-verus", "version": "2.0.0", "command": "atomize"},
            "source": {"repo": "r", "commit": "c", "language": "rust", "package": "p", "package-version": "1.0"},
            "timestamp": "2025-01-01T00:00:00Z",
            "data": {"probe:a/1.0/f()": {"display-name": "f", "dependencies": [], "code-module": "", "code-path": "a.rs", "code-text": {"lines-start": 1, "lines-end": 10}, "kind": "exec", "language": "rust"}}
        });

        std::fs::File::create(&specs_file)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&specs).unwrap().as_bytes())
            .unwrap();
        std::fs::File::create(&atoms_file)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&atoms).unwrap().as_bytes())
            .unwrap();

        let (_, _, cat_s) = load_generic_file(&specs_file).unwrap();
        let (_, _, cat_a) = load_generic_file(&atoms_file).unwrap();

        assert_eq!(cat_s, SchemaCategory::Specs);
        assert_eq!(cat_a, SchemaCategory::Atoms);
        assert_ne!(
            cat_s, cat_a,
            "different categories should be distinguishable"
        );
    }

    // =========================================================================
    // Mappings loader
    // =========================================================================

    /// Duplicate `from` keys are 1-to-many mappings: every record survives the
    /// load, and the endpoint lookup helper groups them.
    #[test]
    fn test_duplicate_from_keys_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mappings.json");
        let content = serde_json::json!({
            "schema": "probe/mappings",
            "schema-version": "3.0",
            "mappings": [
                {"from": "probe:a/1.0/f()", "to": "probe:a.lean.f", "confidence": "manual"},
                {"from": "probe:a/1.0/f()", "to": "probe:a.lean.g", "confidence": "manual"}
            ]
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let mappings = load_mappings(&path).unwrap();
        assert_eq!(mappings.len(), 2, "both records preserved");
        assert_eq!(mappings[0].confidence, "manual");

        let (from_to, to_from) = endpoint_lookup_maps(&mappings);
        assert_eq!(from_to.len(), 1, "one unique 'from' key");
        let targets = from_to.get("probe:a/1.0/f()").unwrap();
        assert_eq!(targets.len(), 2, "both targets should be preserved");
        assert!(targets.contains(&"probe:a.lean.f".to_string()));
        assert!(targets.contains(&"probe:a.lean.g".to_string()));

        assert_eq!(to_from.len(), 2, "two unique 'to' keys");
        assert_eq!(to_from["probe:a.lean.f"], vec!["probe:a/1.0/f()"]);
        assert_eq!(to_from["probe:a.lean.g"], vec!["probe:a/1.0/f()"]);
    }

    /// P8: mapping-file endpoints are normalized at load, before any lookup.
    #[test]
    fn test_load_mappings_normalizes_endpoints() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mappings.json");
        let content = serde_json::json!({
            "schema": "probe/mappings",
            "schema-version": "3.0",
            "mappings": [
                {"from": "probe:a/1.0/f().", "to": "probe:a.lean.f.", "confidence": "manual",
                 "method": "hand-written"}
            ]
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let mappings = load_mappings(&path).unwrap();
        assert_eq!(mappings[0].from, "probe:a/1.0/f()");
        assert_eq!(mappings[0].to, "probe:a.lean.f");
        assert_eq!(mappings[0].method.as_deref(), Some("hand-written"));
    }

    /// Fail-closed at the file boundary: an out-of-vocabulary confidence is
    /// rejected at load (merge would otherwise emit records the executable
    /// schema rejects), and an empty method is canonicalized to absent (P27).
    #[test]
    fn test_load_mappings_validates_confidence_and_canonicalizes_method() {
        let dir = tempfile::tempdir().unwrap();

        let bad = dir.path().join("bad.json");
        let content = serde_json::json!({
            "schema": "probe/mappings",
            "schema-version": "3.0",
            "mappings": [
                {"from": "probe:a/1.0/f()", "to": "probe:a.lean.f", "confidence": "high"}
            ]
        });
        std::fs::write(&bad, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        let err = load_mappings(&bad).unwrap_err();
        assert!(err.contains("invalid confidence"), "{err}");
        assert!(err.contains("bad.json"), "{err}");

        let empty_method = dir.path().join("empty-method.json");
        let content = serde_json::json!({
            "schema": "probe/mappings",
            "schema-version": "3.0",
            "mappings": [
                {"from": "probe:a/1.0/f()", "to": "probe:a.lean.f", "confidence": "exact",
                 "method": ""}
            ]
        });
        std::fs::write(
            &empty_method,
            serde_json::to_string_pretty(&content).unwrap(),
        )
        .unwrap();
        let mappings = load_mappings(&empty_method).unwrap();
        assert_eq!(mappings[0].method, None, "empty method canonicalized");
    }

    // P8: `dependencies-with-locations` code-names are normalized alongside
    // keys and the dependencies set.
    #[test]
    fn test_dependencies_with_locations_normalized() {
        let mut atom = make_real_atom("f", "src/lib.rs", "rust", "exec");
        atom.extensions.insert(
            "dependencies-with-locations".to_string(),
            serde_json::json!([{"code-name": "probe:a/1.0/g().", "location": "inner"}]),
        );
        let mut atoms = BTreeMap::new();
        atoms.insert("probe:a/1.0/f()".to_string(), atom);

        let (out, _, _) = normalize_atoms(atoms).unwrap();
        let dwl = out["probe:a/1.0/f()"]
            .extensions
            .get("dependencies-with-locations")
            .unwrap();
        assert_eq!(dwl[0]["code-name"], "probe:a/1.0/g()");
    }

    // P14 at the envelope level: the same evidence presented in different
    // input orders — pre-attached record arrays reversed, mappings file
    // reordered — serializes to byte-identical full envelope JSON (fixed
    // meta). This exercises the canonicalization (record sort + triple dedup),
    // not just BTreeMap ordering.
    #[test]
    fn test_full_envelope_serialization_deterministic() {
        let build = |reverse: bool| {
            let (mut a, b, _, mut m) = law_fixtures();
            // Pre-attach two records on one atom, in order-dependent form.
            let mut records = vec![
                serde_json::json!({"target": "probe:Pkg.f", "confidence": "exact"}),
                serde_json::json!({"target": "probe:Other.f", "confidence": "manual"}),
            ];
            if reverse {
                records.reverse();
                m.reverse();
            }
            a.get_mut("probe:a/1.0/f()")
                .unwrap()
                .extensions
                .insert(MAPS_TO.to_string(), serde_json::Value::Array(records));

            let (merged, _) = merge_mapped(vec![a, b], &m);
            let envelope = MergedAtomEnvelope {
                schema: "probe/merged-atoms".to_string(),
                schema_version: "3.1".to_string(),
                tool: Tool {
                    name: "probe".to_string(),
                    version: "0.5.0".to_string(),
                    command: "merge".to_string(),
                },
                inputs: vec![InputProvenance {
                    schema: "probe-verus/atoms".to_string(),
                    source: crate::types::Source {
                        repo: "r".to_string(),
                        commit: "c".to_string(),
                        language: "rust".to_string(),
                        package: "p".to_string(),
                        package_version: "1.0".to_string(),
                        extensions: BTreeMap::new(),
                    },
                }],
                timestamp: "2026-09-29T12:00:00Z".to_string(),
                data: merged,
            };
            serde_json::to_string_pretty(&envelope).unwrap()
        };
        assert_eq!(build(false), build(true));
    }

    #[test]
    fn test_load_mappings_rejects_legacy_translations_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.json");
        let content = serde_json::json!({
            "schema": "probe/translations",
            "schema-version": "3.0",
            "mappings": [
                {"from": "probe:a/1.0/f()", "to": "probe:a.lean.f", "confidence": "manual"}
            ]
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let result = load_mappings(&path);
        assert!(result.is_err(), "legacy schema should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.contains("probe/mappings"),
            "error should mention expected schema: {err}"
        );
    }
}
