// @kb: kb/engineering/properties.md#p14-deterministic-output
// @kb: kb/engineering/properties.md#p23-transitive-verification

use crate::types::Atom;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

fn get_verification_status(atom: &Atom) -> Option<&str> {
    atom.extensions
        .get("verification-status")
        .and_then(|v| v.as_str())
}

/// Presence-based on purpose: P23 quantifies seeds over "any atom *carrying*
/// `status-origin`", so a malformed non-string marker must fail closed as a
/// seed rather than silently read as absent. Value validation happens at the
/// load boundary ([`crate::types::validate_status_origins`]); this guards the
/// bare-map API too.
fn has_status_origin(atom: &Atom) -> bool {
    atom.extensions.contains_key("status-origin")
}

fn is_verified(atom: &Atom) -> bool {
    get_verification_status(atom).is_some_and(|s| s == "verified" || s == "transitively-verified")
}

// @kb: kb/engineering/schema.md#common-optional-fields — status-origin marker
/// Blocker/contamination seeds (P23, ADR-006): atoms with explicit
/// "unverified" or "failed" status, plus every `status-origin`-bearing atom
/// (imported or graph-inexpressible evidence — never promotable). Atoms with
/// missing status are transparent, not seeds; plain "trusted" atoms are
/// boundaries, not seeds.
fn is_seed(atom: &Atom) -> bool {
    matches!(get_verification_status(atom), Some("unverified" | "failed"))
        || has_status_origin(atom)
}

/// A "trusted" atom is a trust boundary only when it carries no
/// `status-origin`: copied trust must not shield callers (ADR-006). The
/// boundary covers the atom's entire dependency closure (whole-atom trust).
fn is_trusted_boundary(atom: &Atom) -> bool {
    get_verification_status(atom) == Some("trusted") && !has_status_origin(atom)
}

/// Kinds whose members (enum constructors, struct fields/projections, class
/// fields) are referenced as dependencies but are not emitted as standalone
/// atoms.
fn is_type_definition(kind: &str) -> bool {
    matches!(kind, "inductive" | "structure" | "class")
}

/// A dependency is a benign "type member" when its parent path segment names an
/// extracted type atom (an `inductive`/`structure`). Such references — e.g. an
/// enum variant `Error.StateDecode` or a struct field/projection — have no
/// verification status of their own, so treating them as trusted is correct and
/// they should not be surfaced as missing. Anything else (a reference whose
/// parent is absent, or whose parent is a `def`/`theorem`/etc.) is a genuine
/// orphan worth reporting.
fn is_extracted_type_member(dep: &str, atoms: &BTreeMap<String, Atom>) -> bool {
    dep.rsplit_once('.')
        .and_then(|(parent, _member)| atoms.get(parent))
        .is_some_and(|parent| is_type_definition(&parent.kind))
}

/// Recompute verification labels through the dependency graph (P23).
///
/// This is a **recomputation**, not an upgrade pass: one reverse BFS from one
/// seed set — explicit `"failed"`/`"unverified"` atoms plus every
/// `status-origin`-bearing atom — then every atom whose status is
/// `"verified"` or `"transitively-verified"` has its label set fresh:
/// reaches a seed along a path with no trusted boundary, or is itself a
/// seed → `"verified"`; otherwise → `"transitively-verified"`. A contaminated
/// atom arriving as `"transitively-verified"` is downgraded, and a
/// `status-origin`-bearing `"transitively-verified"` is rewritten to
/// `"verified"` unconditionally. The result is a function of the final graph
/// and base statuses only.
///
/// Contamination flows through missing-status atoms (transparent by
/// construction) and stops only at trusted boundaries (`"trusted"` with no
/// `status-origin`). Seeds keep their own base status — blocking never
/// downgrades below `"verified"`, and `"failed"`/`"unverified"`/`"trusted"`
/// atoms are never rewritten.
///
/// Returns `(transitive_count, local_count, missing_deps)` for reporting.
/// `missing_deps` lists only *genuine orphans* — dependency code-names absent
/// from the atom map that are not members of an extracted type. References to
/// constructors/fields of an extracted `inductive`/`structure` are benign (the
/// type is extracted, its members are not standalone atoms) and are excluded so
/// real gaps are not lost in the noise.
// @kb: kb/engineering/schema.md#common-optional-fields — verification-status values
pub fn enrich_verification_status(
    atoms: &mut BTreeMap<String, Atom>,
) -> (usize, usize, Vec<String>) {
    // Use owned Strings throughout to avoid borrow-checker conflicts
    // between reading atoms (for the graph) and writing back results.

    // 1. Build reverse dependency index: for each dep, who depends on it?
    //    Uses BTreeMap/BTreeSet for deterministic iteration (P14).
    let mut reverse_deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut candidates: BTreeSet<String> = BTreeSet::new();
    // BTreeSet keeps iteration sorted + deduped for deterministic output (P14).
    let mut all_missing: BTreeSet<String> = BTreeSet::new();

    for (code_name, atom) in atoms.iter() {
        if is_verified(atom) {
            candidates.insert(code_name.clone());
        }
        for dep in &atom.dependencies {
            if !atoms.contains_key(dep.as_str()) {
                all_missing.insert(dep.clone());
            }
            reverse_deps
                .entry(dep.clone())
                .or_default()
                .insert(code_name.clone());
        }
    }

    // Split missing deps: benign type members (enum variants / struct fields of
    // an extracted type) vs. genuine orphans. Only the latter are surfaced so
    // real gaps stay visible instead of being drowned out.
    let mut missing_type_members: usize = 0;
    let mut missing_deps: Vec<String> = Vec::new();
    for dep in all_missing {
        if is_extracted_type_member(&dep, atoms) {
            missing_type_members += 1;
        } else {
            missing_deps.push(dep);
        }
    }

    for missing in &missing_deps {
        eprintln!("Warning: dependency {missing:?} not found in atom map (treated as trusted)");
    }
    if missing_type_members > 0 {
        eprintln!(
            "Note: {missing_type_members} reference(s) to constructors/fields of extracted types treated as trusted"
        );
    }

    // 2. One seed set (P23): explicit "unverified"/"failed" atoms plus every
    //    status-origin-bearing atom. A zero-length path counts — a seed is
    //    never promoted — so seeds start out blocked.
    let mut blocked: BTreeSet<String> = BTreeSet::new();
    let mut worklist: VecDeque<String> = VecDeque::new();

    for (code_name, atom) in atoms.iter() {
        if is_seed(atom) {
            blocked.insert(code_name.clone());
            worklist.push_back(code_name.clone());
        }
    }

    // 3. One reverse BFS. Contamination flows through every caller that is
    //    not a trusted boundary — including missing-status atoms, which are
    //    transparent by construction — and stops at trusted boundaries.
    while let Some(atom_name) = worklist.pop_front() {
        if let Some(callers) = reverse_deps.get(&atom_name) {
            for caller in callers {
                if blocked.contains(caller) || is_trusted_boundary(&atoms[caller]) {
                    continue;
                }
                blocked.insert(caller.clone());
                worklist.push_back(caller.clone());
            }
        }
    }

    // 4. Set every candidate's label fresh: blocked (reaches a seed along a
    //    non-trusted path, or is itself a seed) → "verified"; otherwise →
    //    "transitively-verified". This downgrades a contaminated
    //    "transitively-verified" and rewrites a status-origin-bearing
    //    "transitively-verified" to "verified" unconditionally.
    let mut transitive_count = 0;
    let mut local_count = 0;

    for code_name in &candidates {
        let label = if blocked.contains(code_name) {
            local_count += 1;
            "verified"
        } else {
            transitive_count += 1;
            "transitively-verified"
        };
        atoms.get_mut(code_name).unwrap().extensions.insert(
            "verification-status".to_string(),
            serde_json::Value::String(label.to_string()),
        );
    }

    (transitive_count, local_count, missing_deps)
}

/// Statistics from carrier preparation (`prepare = enrich ∘ normalize`).
pub struct PrepareStats {
    pub keys_normalized: usize,
    pub transitive: usize,
    pub local: usize,
    pub missing_deps: Vec<String>,
    /// Post-normalization collisions that discarded a distinct real atom, as
    /// `(discarded original key, surviving normalized key)` pairs. Discarding
    /// evidence silently can launder contamination, so the `probe enrich`
    /// boundary rejects the input when this is non-empty (P8).
    pub dropped_atoms: Vec<(String, String)>,
}

// @kb: kb/engineering/properties.md#p8-code-name-normalization
// @kb: kb/engineering/properties.md#p23-transitive-verification
/// Carrier preparation for unary recomputation boundaries (ADR-006):
/// normalize code-names (P8), then recompute enrichment (P23), matching
/// merge's per-input normalize-first ordering. Enrichment over an
/// unnormalized map can resolve names differently (a dotted-alias dependency
/// dangles instead of reaching its contamination source), so normalization
/// must run first.
///
/// Normalization is not injective, so it can collide keys even within a
/// single input; a collision that would discard a distinct real atom is
/// reported in [`PrepareStats::dropped_atoms`] (the CLI boundary rejects it
/// fail-closed — with one input there is no second source to arbitrate the
/// evidence).
pub fn prepare_atoms(atoms: BTreeMap<String, Atom>) -> (BTreeMap<String, Atom>, PrepareStats) {
    let (mut atoms, keys_normalized, dropped_atoms) =
        crate::commands::merge::normalize_atoms(atoms);
    let (transitive, local, missing_deps) = enrich_verification_status(&mut atoms);
    (
        atoms,
        PrepareStats {
            keys_normalized,
            transitive,
            local,
            missing_deps,
            dropped_atoms,
        },
    )
}

/// CLI entry point: load atom file, enrich verification status, write JSON.
///
/// The input passes the envelope schema check and the shared authority
/// validator (projection rejection + version gate, ADR-006) before any
/// recomputation, then carrier preparation (`prepare = enrich ∘ normalize`)
/// recomputes the labels. The output preserves the input envelope structure
/// exactly, up to normalized code-names.
// @kb: kb/engineering/schema.md#authority-validation-and-re-enrichment
pub fn cmd_enrich(input: &Path, output: Option<&Path>) {
    let content = std::fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", input.display());
        std::process::exit(1);
    });

    let mut raw: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing JSON in {}: {e}", input.display());
        std::process::exit(1);
    });

    let origin = input.display().to_string();
    let meta = crate::types::parse_envelope(&raw, &origin).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });
    if meta.category != crate::types::SchemaCategory::Atoms {
        eprintln!(
            "Error: {origin}: expected atoms schema, got {} (\"{}\")",
            meta.category, meta.schema
        );
        std::process::exit(1);
    }
    if let Err(e) = crate::authority::validate_authority(
        &meta,
        &origin,
        crate::authority::AuthorityScope::Recompute,
    ) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    let atoms: BTreeMap<String, Atom> =
        serde_json::from_value(meta.data_value).unwrap_or_else(|e| {
            eprintln!("Error deserializing atoms from {}: {e}", input.display());
            std::process::exit(1);
        });

    // Fail closed on out-of-enum status-origin markers (ADR-006 Decision 2):
    // the runtime does not schema-validate, and a malformed marker must not
    // silently read as absent.
    if let Err(e) = crate::types::validate_status_origins(&atoms) {
        eprintln!("Error: {origin}: {e}");
        std::process::exit(1);
    }

    let (atoms, stats) = prepare_atoms(atoms);

    // A normalization collision that discarded a distinct real atom is
    // producer error at a unary boundary: silently selecting one atom's
    // evidence can launder contamination (P8). Reject rather than write.
    if !stats.dropped_atoms.is_empty() {
        for (discarded, kept) in &stats.dropped_atoms {
            eprintln!(
                "Error: normalization collision: atom {discarded:?} would be discarded \
                 (a distinct atom already occupies {kept:?})"
            );
        }
        eprintln!(
            "Error: {origin}: refusing to enrich — normalization collided {} distinct \
             atom(s); fix the producer aliases and regenerate (P8)",
            stats.dropped_atoms.len()
        );
        std::process::exit(1);
    }

    let not_verified = atoms.len() - stats.transitive - stats.local;

    if stats.keys_normalized > 0 {
        eprintln!("Keys normalized: {}", stats.keys_normalized);
    }
    eprintln!(
        "Transitively verified: {}  |  Locally-scoped verified: {}  |  Not verified: {not_verified}",
        stats.transitive, stats.local
    );

    let enriched_data = serde_json::to_value(&atoms).expect("failed to serialize atoms");
    raw.as_object_mut()
        .expect("envelope is not a JSON object")
        .insert("data".to_string(), enriched_data);

    let json = serde_json::to_string_pretty(&raw).expect("failed to serialize output");

    let default_name;
    let out_path = match output {
        Some(p) => p,
        None => {
            default_name = default_output_name(input);
            Path::new(&default_name)
        }
    };

    std::fs::write(out_path, &json).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {e}", out_path.display());
        std::process::exit(1);
    });
    eprintln!("Wrote {}", out_path.display());
}

fn default_output_name(input: &Path) -> String {
    let stem = input
        .file_stem()
        .map_or("atoms", |s| s.to_str().unwrap_or("atoms"));
    format!("enriched_{stem}.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CodeText;

    fn make_atom(display_name: &str) -> Atom {
        Atom {
            display_name: display_name.to_string(),
            dependencies: BTreeSet::new(),
            code_module: String::new(),
            code_path: "src/lib.rs".to_string(),
            code_text: CodeText {
                lines_start: 1,
                lines_end: 10,
            },
            kind: "exec".to_string(),
            language: "rust".to_string(),
            extensions: BTreeMap::new(),
        }
    }

    fn set_status(atom: &mut Atom, status: &str) {
        atom.extensions.insert(
            "verification-status".to_string(),
            serde_json::Value::String(status.to_string()),
        );
    }

    fn set_verified(atom: &mut Atom) {
        set_status(atom, "verified");
    }

    fn set_trusted(atom: &mut Atom) {
        set_status(atom, "trusted");
    }

    fn add_dep(atom: &mut Atom, dep: &str) {
        atom.dependencies.insert(dep.to_string());
    }

    fn get_vs(atom: &Atom) -> Option<&str> {
        atom.extensions
            .get("verification-status")
            .and_then(|v| v.as_str())
    }

    #[test]
    fn test_leaf_no_deps() {
        let mut atoms = BTreeMap::new();
        let mut a = make_atom("a");
        set_verified(&mut a);
        atoms.insert("a".to_string(), a);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_all_deps_verified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_verified(&mut c);
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(
            get_vs(atoms.get("b").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(
            get_vs(atoms.get("c").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_one_dep_failed_contaminates() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "failed");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("failed"));
    }

    #[test]
    fn test_one_dep_unverified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_dep_trusted_does_not_block() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_trusted(&mut b);
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_dep_missing_from_map() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "nonexistent");
        atoms.insert("a".to_string(), a);

        let (_t, _l, missing) = enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(missing, vec!["nonexistent"]);
    }

    #[test]
    fn test_type_member_dep_not_reported_missing() {
        let mut atoms = BTreeMap::new();

        // A verified function referencing an enum variant `MyEnum.VariantA`
        // and a genuine orphan `totally_unknown`.
        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "MyEnum.VariantA");
        add_dep(&mut a, "totally_unknown");
        atoms.insert("a".to_string(), a);

        // The enum type itself IS extracted (as an inductive atom); its
        // variants are not separate atoms.
        let mut e = make_atom("MyEnum");
        e.kind = "inductive".to_string();
        atoms.insert("MyEnum".to_string(), e);

        let (_t, _l, missing) = enrich_verification_status(&mut atoms);

        // The enum-variant reference is a benign type member -> not reported;
        // the genuine orphan still is.
        assert_eq!(missing, vec!["totally_unknown"]);
        // Neither missing dep blocks transitive verification.
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_struct_field_dep_not_reported_missing() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "MyStruct.field");
        atoms.insert("a".to_string(), a);

        let mut s = make_atom("MyStruct");
        s.kind = "structure".to_string();
        atoms.insert("MyStruct".to_string(), s);

        let (_t, _l, missing) = enrich_verification_status(&mut atoms);
        assert!(missing.is_empty(), "struct field ref should be suppressed");
    }

    #[test]
    fn test_member_of_non_type_parent_still_reported() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        // Parent `some_fn` is an `exec` def (make_atom default), not a type,
        // so a missing member of it is a genuine gap worth surfacing.
        add_dep(&mut a, "some_fn.inner");
        atoms.insert("a".to_string(), a);

        let f = make_atom("some_fn");
        atoms.insert("some_fn".to_string(), f);

        let (_t, _l, missing) = enrich_verification_status(&mut atoms);
        assert_eq!(missing, vec!["some_fn.inner"]);
    }

    #[test]
    fn test_transitive_chain() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_status(&mut c, "unverified");
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_diamond_dependency() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        add_dep(&mut a, "c");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "d");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_verified(&mut c);
        add_dep(&mut c, "d");
        atoms.insert("c".to_string(), c);

        let mut d = make_atom("d");
        set_status(&mut d, "unverified");
        atoms.insert("d".to_string(), d);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("d").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_cycle_all_verified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "a");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(
            get_vs(atoms.get("b").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_cycle_with_unverified_dep() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        add_dep(&mut b, "d");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_verified(&mut c);
        add_dep(&mut c, "a");
        atoms.insert("c".to_string(), c);

        let mut d = make_atom("d");
        set_status(&mut d, "unverified");
        atoms.insert("d".to_string(), d);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("d").unwrap()), Some("unverified"));
    }

    #[test]
    fn test_missing_status_does_not_contaminate() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        // b has no verification-status at all (untracked/Grey)
        let b = make_atom("b");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_explicit_unverified_contaminates_but_missing_does_not() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        add_dep(&mut a, "c");
        atoms.insert("a".to_string(), a);

        // b is explicitly unverified — contaminates
        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        // c has no status — does NOT contaminate
        let c = make_atom("c");
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
    }

    #[test]
    fn test_non_verified_atoms_untouched() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_status(&mut a, "unverified");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "failed");
        atoms.insert("b".to_string(), b);

        let c = make_atom("c");
        atoms.insert("c".to_string(), c);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("unverified"));
        assert_eq!(get_vs(atoms.get("b").unwrap()), Some("failed"));
        assert_eq!(get_vs(atoms.get("c").unwrap()), None);
    }

    #[test]
    fn test_idempotency() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        let first = get_vs(atoms.get("a").unwrap()).unwrap().to_string();

        enrich_verification_status(&mut atoms);
        let second = get_vs(atoms.get("a").unwrap()).unwrap().to_string();

        assert_eq!(first, second);
        assert_eq!(first, "verified");
    }

    #[test]
    fn test_idempotency_transitively_verified() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        atoms.insert("a".to_string(), a);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );

        // Running again should not change the result (already upgraded)
        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    #[test]
    fn test_deterministic_output() {
        let build = || {
            let mut atoms = BTreeMap::new();
            let mut a = make_atom("a");
            set_verified(&mut a);
            add_dep(&mut a, "b");
            add_dep(&mut a, "c");
            atoms.insert("a".to_string(), a);

            let mut b = make_atom("b");
            set_verified(&mut b);
            atoms.insert("b".to_string(), b);

            let mut c = make_atom("c");
            set_status(&mut c, "unverified");
            atoms.insert("c".to_string(), c);
            atoms
        };

        let mut atoms1 = build();
        let mut atoms2 = build();

        enrich_verification_status(&mut atoms1);
        enrich_verification_status(&mut atoms2);

        let json1 = serde_json::to_string(&atoms1).unwrap();
        let json2 = serde_json::to_string(&atoms2).unwrap();
        assert_eq!(json1, json2);
    }

    fn set_origin(atom: &mut Atom, origin: &str) {
        atom.extensions.insert(
            "status-origin".to_string(),
            serde_json::Value::String(origin.to_string()),
        );
    }

    // Plan §3 test (a): recomputation downgrades a stale label.
    #[test]
    fn test_contaminated_transitively_verified_is_downgraded() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_status(&mut a, "transitively-verified");
        add_dep(&mut a, "b");
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_status(&mut b, "unverified");
        atoms.insert("b".to_string(), b);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("a").unwrap()), Some("verified"));
    }

    // Plan §3 test (b): missing-status atoms are transparent — contamination
    // flows through them (P23).
    #[test]
    fn test_contamination_flows_through_missing_status() {
        let mut atoms = BTreeMap::new();

        let mut v = make_atom("v");
        set_verified(&mut v);
        add_dep(&mut v, "helper");
        atoms.insert("v".to_string(), v);

        let mut helper = make_atom("helper");
        add_dep(&mut helper, "bad");
        atoms.insert("helper".to_string(), helper);

        let mut bad = make_atom("bad");
        set_status(&mut bad, "unverified");
        atoms.insert("bad".to_string(), bad);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("v").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("helper").unwrap()), None);
    }

    // Plan §3 test (c): a trusted atom is a whole-atom boundary — nothing
    // behind it contaminates, including a dep listed in its spec-position
    // extension arrays (enrichment traverses the unified `dependencies` set).
    #[test]
    fn test_trusted_boundary_blocks_contamination() {
        let mut atoms = BTreeMap::new();

        let mut v = make_atom("v");
        set_verified(&mut v);
        add_dep(&mut v, "t");
        atoms.insert("v".to_string(), v);

        let mut t = make_atom("t");
        set_trusted(&mut t);
        add_dep(&mut t, "bad");
        t.extensions.insert(
            "requires-dependencies".to_string(),
            serde_json::json!(["bad"]),
        );
        atoms.insert("t".to_string(), t);

        let mut bad = make_atom("bad");
        set_status(&mut bad, "failed");
        atoms.insert("bad".to_string(), bad);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("v").unwrap()),
            Some("transitively-verified")
        );
        assert_eq!(get_vs(atoms.get("t").unwrap()), Some("trusted"));
    }

    // Plan §3 test (d): correspondence records are not dependencies and never
    // participate in the BFS (P27).
    #[test]
    fn test_maps_to_records_do_not_contaminate() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        a.extensions.insert(
            "maps-to".to_string(),
            serde_json::json!([{ "target": "bad", "confidence": "exact" }]),
        );
        a.extensions.insert(
            "mapped-from".to_string(),
            serde_json::json!([{ "target": "bad", "confidence": "exact" }]),
        );
        atoms.insert("a".to_string(), a);

        let mut bad = make_atom("bad");
        set_status(&mut bad, "unverified");
        atoms.insert("bad".to_string(), bad);

        enrich_verification_status(&mut atoms);
        assert_eq!(
            get_vs(atoms.get("a").unwrap()),
            Some("transitively-verified")
        );
    }

    // Plan §3 test (g): a translation-origin atom is never promoted; its own
    // base status survives contamination unchanged (seeds keep their status).
    #[test]
    fn test_translation_origin_never_promoted() {
        let mut atoms = BTreeMap::new();

        // Leaf with no deps — unmarked it would be promoted.
        let mut g = make_atom("g");
        set_verified(&mut g);
        set_origin(&mut g, "translation");
        atoms.insert("g".to_string(), g);

        // Marked atom over a contaminated dep stays at its base status.
        let mut h = make_atom("h");
        set_verified(&mut h);
        set_origin(&mut h, "translation");
        add_dep(&mut h, "bad");
        atoms.insert("h".to_string(), h);

        let mut bad = make_atom("bad");
        set_status(&mut bad, "failed");
        atoms.insert("bad".to_string(), bad);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("g").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("h").unwrap()), Some("verified"));
    }

    // Plan §3 test (h): translation-origin atoms are promotion blockers — a
    // locally verified caller is not promoted; the same caller behind a
    // trusted boundary is.
    #[test]
    fn test_translation_origin_blocks_caller_promotion() {
        let mut atoms = BTreeMap::new();

        let mut f = make_atom("f");
        set_verified(&mut f);
        add_dep(&mut f, "g");
        atoms.insert("f".to_string(), f);

        let mut g = make_atom("g");
        set_verified(&mut g);
        set_origin(&mut g, "translation");
        atoms.insert("g".to_string(), g);

        // f2 reaches the marked atom only through a plain trusted boundary.
        let mut f2 = make_atom("f2");
        set_verified(&mut f2);
        add_dep(&mut f2, "t");
        atoms.insert("f2".to_string(), f2);

        let mut t = make_atom("t");
        set_trusted(&mut t);
        add_dep(&mut t, "g");
        atoms.insert("t".to_string(), t);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("f").unwrap()), Some("verified"));
        assert_eq!(
            get_vs(atoms.get("f2").unwrap()),
            Some("transitively-verified")
        );
    }

    // Plan §3 test (i): an imported "transitively-verified" is rewritten to
    // "verified" unconditionally, not only when contaminated.
    #[test]
    fn test_translation_origin_transitively_verified_rewritten() {
        let mut atoms = BTreeMap::new();

        let mut g = make_atom("g");
        set_status(&mut g, "transitively-verified");
        set_origin(&mut g, "translation");
        atoms.insert("g".to_string(), g);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("g").unwrap()), Some("verified"));
    }

    // Plan §3 test (j): trusted-boundary precedence — copied trust
    // (trusted + status-origin) seeds and must not shield its callers;
    // plain local trusted is a boundary.
    #[test]
    fn test_copied_trusted_is_seed_not_boundary() {
        let mut atoms = BTreeMap::new();

        let mut caller1 = make_atom("caller1");
        set_verified(&mut caller1);
        add_dep(&mut caller1, "copied_trusted");
        atoms.insert("caller1".to_string(), caller1);

        let mut copied = make_atom("copied_trusted");
        set_trusted(&mut copied);
        set_origin(&mut copied, "translation");
        atoms.insert("copied_trusted".to_string(), copied);

        let mut caller2 = make_atom("caller2");
        set_verified(&mut caller2);
        add_dep(&mut caller2, "local_trusted");
        atoms.insert("caller2".to_string(), caller2);

        let mut local = make_atom("local_trusted");
        set_trusted(&mut local);
        atoms.insert("local_trusted".to_string(), local);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("caller1").unwrap()), Some("verified"));
        assert_eq!(
            get_vs(atoms.get("caller2").unwrap()),
            Some("transitively-verified")
        );
        // Seeds keep their own base status.
        assert_eq!(
            get_vs(atoms.get("copied_trusted").unwrap()),
            Some("trusted")
        );
    }

    // Plan §3 test (k): kernel-taint synthetic replicas of the probe-lean
    // aux-fold fixtures — a marked "verified" with empty deps (ownSorry) or
    // with its taint path through a non-emitted node (viaNoRange) stays
    // "verified", and a locally verified caller of either is not promoted.
    #[test]
    fn test_kernel_taint_replicas() {
        let mut atoms = BTreeMap::new();

        // ownSorry replica: verified, empty dependency list, marked.
        let mut own_sorry = make_atom("ownSorry");
        set_verified(&mut own_sorry);
        set_origin(&mut own_sorry, "kernel-taint");
        atoms.insert("ownSorry".to_string(), own_sorry);

        // viaNoRange replica: verified, taint dep absent from the map.
        let mut via = make_atom("viaNoRange");
        set_verified(&mut via);
        set_origin(&mut via, "kernel-taint");
        add_dep(&mut via, "nonEmittedAux");
        atoms.insert("viaNoRange".to_string(), via);

        let mut caller = make_atom("caller");
        set_verified(&mut caller);
        add_dep(&mut caller, "ownSorry");
        atoms.insert("caller".to_string(), caller);

        let mut caller2 = make_atom("caller2");
        set_verified(&mut caller2);
        add_dep(&mut caller2, "viaNoRange");
        atoms.insert("caller2".to_string(), caller2);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("ownSorry").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("viaNoRange").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("caller").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("caller2").unwrap()), Some("verified"));
    }

    // P23 quantifies seeds over "any atom carrying status-origin": a marked
    // atom with no verification-status at all is still a seed — it blocks its
    // callers and is never labelled itself.
    #[test]
    fn test_marked_status_less_atom_is_seed() {
        let mut atoms = BTreeMap::new();

        let mut caller = make_atom("caller");
        set_verified(&mut caller);
        add_dep(&mut caller, "marked");
        atoms.insert("caller".to_string(), caller);

        let mut marked = make_atom("marked");
        set_origin(&mut marked, "translation");
        atoms.insert("marked".to_string(), marked);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("caller").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("marked").unwrap()), None);
    }

    // Carrier preparation (ADR-006): the dotted-alias contamination
    // regression — without P8 normalization the dep "g." dangles (treated as
    // trusted) and f is wrongly promoted; prepared, it reaches g [failed].
    #[test]
    fn test_prepare_normalizes_before_enrichment() {
        let mut atoms = BTreeMap::new();

        let mut f = make_atom("f");
        set_verified(&mut f);
        add_dep(&mut f, "g.");
        atoms.insert("f".to_string(), f);

        let mut g = make_atom("g");
        set_status(&mut g, "failed");
        atoms.insert("g".to_string(), g);

        let (prepared, stats) = prepare_atoms(atoms);
        assert_eq!(get_vs(prepared.get("f").unwrap()), Some("verified"));
        assert_eq!(stats.local, 1);
        assert_eq!(stats.transitive, 0);
        assert!(stats.missing_deps.is_empty());
    }

    // Carrier preparation also normalizes dotted keys (P8), so a legacy
    // "g()."-keyed atom is reachable by its normalized name.
    #[test]
    fn test_prepare_normalizes_keys() {
        let mut atoms = BTreeMap::new();

        let mut f = make_atom("f");
        set_verified(&mut f);
        add_dep(&mut f, "g()");
        atoms.insert("f".to_string(), f);

        let mut g = make_atom("g");
        set_status(&mut g, "unverified");
        atoms.insert("g().".to_string(), g);

        let (prepared, stats) = prepare_atoms(atoms);
        assert_eq!(stats.keys_normalized, 1);
        assert!(prepared.contains_key("g()"));
        assert_eq!(get_vs(prepared.get("f").unwrap()), Some("verified"));
    }

    // P23 quantifies seeds over "any atom *carrying* status-origin": the
    // check is presence-based, so a malformed non-string marker fails closed
    // as a seed instead of silently reading as absent.
    #[test]
    fn test_non_string_status_origin_is_seed() {
        let mut atoms = BTreeMap::new();

        let mut caller = make_atom("caller");
        set_verified(&mut caller);
        add_dep(&mut caller, "g");
        atoms.insert("caller".to_string(), caller);

        let mut g = make_atom("g");
        set_verified(&mut g);
        g.extensions
            .insert("status-origin".to_string(), serde_json::Value::Null);
        atoms.insert("g".to_string(), g);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("caller").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("g").unwrap()), Some("verified"));
    }

    // The same fail-closed rule on the boundary side: a trusted atom bearing
    // a malformed marker is a seed, not a boundary — it must not shield its
    // callers.
    #[test]
    fn test_non_string_status_origin_disables_trusted_boundary() {
        let mut atoms = BTreeMap::new();

        let mut caller = make_atom("caller");
        set_verified(&mut caller);
        add_dep(&mut caller, "t");
        atoms.insert("caller".to_string(), caller);

        let mut t = make_atom("t");
        set_trusted(&mut t);
        t.extensions
            .insert("status-origin".to_string(), serde_json::json!(42));
        atoms.insert("t".to_string(), t);

        enrich_verification_status(&mut atoms);
        assert_eq!(get_vs(atoms.get("caller").unwrap()), Some("verified"));
        assert_eq!(get_vs(atoms.get("t").unwrap()), Some("trusted"));
    }

    // P8 at the unary boundary: a normalization collision that would discard
    // a distinct real atom is reported in PrepareStats (the CLI rejects it).
    #[test]
    fn test_prepare_reports_collision() {
        let mut atoms = BTreeMap::new();

        let mut g = make_atom("g");
        set_verified(&mut g);
        atoms.insert("g()".to_string(), g);

        let mut g_alias = make_atom("g_alias");
        set_status(&mut g_alias, "failed");
        atoms.insert("g().".to_string(), g_alias);

        let (_, stats) = prepare_atoms(atoms);
        assert_eq!(
            stats.dropped_atoms,
            vec![("g().".to_string(), "g()".to_string())]
        );
    }

    #[test]
    fn test_counts_are_correct() {
        let mut atoms = BTreeMap::new();

        let mut a = make_atom("a");
        set_verified(&mut a);
        atoms.insert("a".to_string(), a);

        let mut b = make_atom("b");
        set_verified(&mut b);
        add_dep(&mut b, "c");
        atoms.insert("b".to_string(), b);

        let mut c = make_atom("c");
        set_status(&mut c, "unverified");
        atoms.insert("c".to_string(), c);

        let (transitive, local, missing) = enrich_verification_status(&mut atoms);
        assert_eq!(transitive, 1);
        assert_eq!(local, 1);
        assert!(missing.is_empty());
    }
}
