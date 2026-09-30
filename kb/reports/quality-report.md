---
auditor: code-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB). Property text audited at origin/main (per common skill §2b). The working tree amends P7 (one "last = input order" sentence) and P8 (projected-input rejection, categorized arrays as sets, specs/proofs tie-break); both amendments are part of PR #83 itself, so every dependency on them is a merge-order constraint internal to that PR, reported below as W1, W2 and I1.
scope: uncommitted working-tree changeset on branch la/merge-soundness-pr3c-collision-warnings-projection (PR #83 review fixes) — src/commands/merge.rs (strict `normalize_atoms`, private `normalize_atoms_reporting_collisions`, categorized-array sort+dedup, specs/proofs collision warning), src/commands/propagate.rs (`prepare_atoms` strict, `PrepareStats::dropped_atoms` removed, `cmd_enrich` rejection block removed), src/commands/project.rs (both branches strict, rejection block removed), tests (merge.rs, propagate.rs, tests/roundtrip.rs), kb/engineering/properties.md, kb/tools/probe-merge.md, CHANGELOG.md — plus the code and docs they touch (tests/propagate.rs, tests/merge_laws.rs, glossary, schema.md, probe-project.md, ADR-006, probe-verus/probe-lean producers of the categorized arrays)
status: 0 critical, 2 warnings, 4 info
---

## Critical

None.

## Warnings

### [W1] Categorized-array canonicalization conforms to P5 only under the amended P8 (merge-order constraint, PR #83)
- **Location**: src/commands/merge.rs:381-385 (sort + dedup inside `normalize_atoms_reporting_collisions`); kb/engineering/properties.md P8, new "categorized dependency arrays … are sets" paragraph (working tree)
- **Issue**: `normalize_atoms` now reorders and deduplicates `requires-`/`ensures-`/`body-`/`type-`/`term-dependencies` on every boundary (merge per input, enrich, project). origin/main's P5 says `μ(A, ∅) = A` *exactly* on the carrier, where the carrier is "normalized, enrichment-consistent atom maps", and origin/main's P8 defines normalization as trailing-dot stripping only. Under that text, an atom map whose categorized arrays are unsorted or contain duplicates is on the carrier, yet merging it with nothing changes it — a literal P5 violation. Only the amended P8 (normalization includes set canonicalization, so such a map is off-carrier and `μ(A, ∅) = enrich(normalize(A))` applies) makes the code conformant. The same holds for P10 ("extensions preserved"), which on main does not license reordering an extension value.
- **Evidence**: `arr.sort_by_cached_key(|entry| match entry.as_str() { Some(name) => (false, name.to_string()), None => (true, entry.to_string()) }); arr.dedup();` — origin/main P8: "Normalization strips trailing `.` characters"; no set semantics for the categorized arrays anywhere in origin/main's properties.md. Practical impact today is nil: probe-verus emits these fields from `BTreeSet<String>` (probe-verus/src/lib.rs:327,339) and probe-lean from `sortDedupNames`/`sortByName` (probe-lean/ProbeLean/Analysis.lean:195,991-993, string order), so current producer output is already canonical.
- **Recommendation**: Land the code and the P8 amendment together (same PR #83; do not split the working-tree commit so that merge.rs lands without properties.md). Optionally make the carrier definition in P4/P5 say explicitly that "normalized" includes categorized-array canonicalization, so P5 does not depend on a reader following P8.

### [W2] `probe project` rejecting collisions on already-projected inputs has its normative basis only in the P8 amendment (merge-order constraint, PR #83)
- **Location**: src/commands/project.rs:252-255 (projected branch now calls strict `normalize_atoms`); kb/engineering/properties.md P8 rejection sentence (working tree)
- **Issue**: origin/main's P8 requires fail-closed rejection "at every recomputation boundary" and names `probe merge` and `probe enrich`. An already-projected input is explicitly *not* recomputed (labels inherited), so main's text neither requires nor names rejection there; the committed HEAD text named only project's *authoritative* input. The working-tree amendment ("`probe project` rejects any input, authoritative or projected") is what the code now implements. The behavior itself is sound and stricter-only, but it is a new fail-closed CLI behavior whose spec exists only in this PR. ADR-006 (kb/decisions/006-correspondence-records.md:167-172) still describes the uniform-rejection policy in terms of merge and enrich only — not a contradiction, but the ADR is not the place a reader will find the view case.
- **Evidence**: `crate::commands::merge::normalize_atoms(loaded.atoms).map(|(atoms, keys_normalized)| (atoms, keys_normalized, None))` in the `loaded.projected` branch; tests/roundtrip.rs:400 (`("collision", collision, "distinct real atom")` in `project_rejects_invalid_projected_input`). The glossary `carrier preparation` entry and kb/tools/probe-project.md:38 already describe the view-side rejection consistently.
- **Recommendation**: Keep the P8 amendment in the same commit/PR as the project.rs change. No code change needed.

## Info

### [I1] Specs/proofs intra-input tie-break (key order) is described only by the P7/P8 amendment (merge-order constraint, PR #83)
- **Location**: src/commands/merge.rs:626-653 (`normalize_generic`); properties.md P7 new sentence, P8 specs/proofs paragraph. Behavior (alias sorting last wins) is unchanged from origin/main, which is silent on intra-input specs collisions; the committed HEAD text's "keeps P7 last-wins" was the misleading part, and the amendment fixes it. Land together.

### [I2] With three or more aliases, earlier warnings name a "kept" key that is later superseded
- **Location**: src/commands/merge.rs:641-644. For `f()`, `f().`, `f()..` the first warning says "kept 'f().'" and the second "kept 'f()..'"; only the last is true. P8 (amended) asks the warning to name the kept key. Consider emitting after the loop, or wording it as "replaced". Not exercised by any test (both tests use two aliases).

### [I3] Removal of public `PrepareStats::dropped_atoms` is logged under "Changed", not "Removed"
- **Location**: CHANGELOG.md:25. The field is public API in v0.4.0 (origin/main src/commands/propagate.rs:205); removing it and making `prepare_atoms` error on collisions is a breaking library change (covered by the 0.4→0.5 minor bump under 0.x semver). No consumers in sibling repos (`rg dropped_atoms|PrepareStats|prepare_atoms` over probe-aeneas/rust/verus/lean/leanblueprint/vcvio: none). Consider a `### Removed` line.

### [I4] schema.md's normalization section does not mention categorized-array canonicalization
- **Location**: kb/engineering/schema.md:310. P8 and kb/tools/probe-merge.md:49 now state that the five arrays are sorted and deduplicated; schema.md still describes normalization as trailing-dot stripping of keys and code-name arrays. Not contradictory, but it is the doc a producer reads for the wire contract.

## Verified clean

Checked against origin/main property text unless noted.

- **P2** (code-name uniqueness): output maps remain `BTreeMap` keyed by normalized name; strict `normalize_atoms` never returns a map where a distinct atom was silently dropped (merge.rs:301-309).
- **P3** (structural stub): collision classification still uses `Atom::is_stub()` only (merge.rs:393-406).
- **P4** (per-input normalization before conflict resolution): merge.rs:476 and :480 normalize each input and prefix errors with the input position before the equal-key loop; the strict helper keeps the ordering intact.
- **P5**: holds on the carrier under the amended P8 (see W1); `tests/merge_laws.rs` still passes (identity/associativity sweeps).
- **P6** (first-wins with stub replacement): merge.rs:486-507 unchanged.
- **P7** (cross-input last-wins): `merge_generic_maps` cross-input override unchanged; the `enumerate()` refactor only threads position into the warning.
- **P8** (normalization, fail-closed at every boundary): all three boundaries now route through the single strict helper — merge (merge.rs:476/480), enrich (propagate.rs:224 via `prepare_atoms`, CLI propagates at :291), project authoritative (project.rs:258) and projected (project.rs:255). The lossy core is private (`fn normalize_atoms_reporting_collisions`), so no library caller can obtain the lossy map; the duplicated CLI rejection blocks are gone and error text is the shared `collision_error`. Keys, `dependencies`, `dependencies-with-locations` code-names, the categorized arrays, record targets and mapping endpoints are all still normalized. Regression coverage: `test_prepare_rejects_collision`, `test_normalization_collision_classification`, `test_intra_input_distinct_real_collision_rejected`, `test_categorized_arrays_compared_as_sets`, tests/propagate.rs `test_enrich_rejects_normalization_collision` (CLI, names both keys, no clobber), tests/roundtrip.rs `project_rejects_distinct_real_normalization_collision` and the projected-input matrix.
- **P10** (extensions preserved): only the five categorized arrays are rewritten, and only canonicalized (see W1 for the main-text caveat); all other extensions untouched.
- **P14** (determinism): categorized arrays now have a fixed order (strings byte-sorted, non-strings after by JSON text) — an improvement.
- **P15** (dependency completeness): sort/dedup does not change set membership, so `dependencies` = union of subsets is preserved; project trimming unchanged.
- **P23** (enrichment): `prepare_atoms` still normalizes before enriching and now errors before enrichment on a collision; re-projection of a view inherits labels (new roundtrip assertion in `projection_recomputes_enrichment_before_trimming`).
- **P27** (records unioned and inert): records validated/canonicalized before the collision comparison (merge.rs:391); benign collapses still union records; distinctness still judged modulo records.
- **P1, P9, P13, P16, P17, P19, P21, P22, P24, P25**: not touched by this changeset.
- **Known issues**: properties.md carries no `C<n>` entries.
- **Docs**: no remaining references to `dropped_atoms`, "refusing to enrich/project", or "keeping last" outside archived plans; kb/tools/probe-merge.md (Phase 2 text, stats table), probe-project.md:38 and glossary `carrier preparation` match the code. Doc comments on `normalize_atoms`, `normalize_atoms_reporting_collisions`, `normalize_generic`, `prepare_atoms`, `merge_atom_maps_raw` and the project.rs preparation comment are current.
- **`@kb:` annotations**: `./scripts/check-kb-links.sh` → "All KB links OK."; `normalize_atoms` gained a P8 annotation.
- **Build**: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` (all suites) pass.
