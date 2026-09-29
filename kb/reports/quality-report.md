---
auditor: code-quality-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB; branch amends P8/P27 wording — those two audited against BOTH the branch text and origin/main's, see W1; all other property text is identical to origin/main)
scope: branch la/merge-soundness-pr3b-maps-to-records vs origin/main (merge-soundness PR 3b — correspondence records, merge re-enrichment, raw staging primitives, P8 intra-input collision reconciliation)
status: 0 critical, 2 warnings, 1 info
---

## Critical

None.

## Warnings

### [W1] P8/P27 amendment is a merge-order constraint (deliberate, plan-authorized)
- **Location**: kb/engineering/properties.md P8 ¶2 and P27 bullet 1 (this branch) vs `git show origin/main:kb/engineering/properties.md`
- **Issue**: against origin/main's P8, merge on a distinct-real intra-input collision "warns and keeps first-wins"; this branch's code **rejects** it (src/commands/merge.rs `merge_atom_maps_raw`, `collision_error`) and amends P8/P27 to match. Code conforms only to the unmerged amendment.
- **Evidence**: the amendment is the reconciliation the plan's §9 row 3b explicitly mandates ("extend rejection to merge's per-input normalization or record the asymmetry in §10") — commit 4d1dbb3 recorded the obligation; the branch records the decision in ADR-006 Decision 3, plan §10, and P8. Rationale: once μ re-enriches internally, warn-and-count lets first-wins-selected evidence feed enrichment inside merge — the laundering path main's own P8 clause condemns at the unary boundary.
- **Recommendation**: none beyond merging this PR as one unit (spec amendment + implementation + tests land together). Reviewers should confirm the rejection choice over the recorded-asymmetry alternative.

### [W2] Staged KB clauses scheduled for PR 3c (carried from the PR 2 audit, narrowed)
- **Location**: kb/engineering/properties.md P8/P9/P15 vs src/commands/{merge,project}.rs
- **Issue**: PR 3b closes the PR 2 audit's P23/P4 "merge re-enriches", P13/P27 records, P5 identity, and raw-staging items. Still scheduled for PR 3c per merge-soundness-fix-plan.md §9:
  - P8 extended array coverage (`requires-dependencies`, `ensures-dependencies`, `body-dependencies`, `type-dependencies`, `term-dependencies` are not yet normalized; keys, `dependencies`, `dependencies-with-locations`, record targets, and mapping endpoints are).
  - P9 provenance dedup (merge still extends without dedup, src/commands/merge.rs `cmd_merge`; envelope idempotence modulo meta therefore not yet testable).
  - P15 categorized-array trim in `probe project`; project's normalize-then-enrich-then-trim (project still exact-key seed-matches — note the known interim regression: `load_mappings` now normalizes endpoints, so a dotted-*key* legacy atom no longer seed-matches until 3c lands; plan §3d records this ordering).
  - Counted collision warnings feeding `stats.conflicts` for the specs/proofs path (`normalize_generic` still last-wins silently within an input; issue 6).
- **Evidence**: plan §9 rows 3b/3c; the 3c row explicitly owns each item.
- **Recommendation**: none for this PR — tracked by the plan; land 3c promptly so the projection-seed interim regression window stays short.

## Info

### [I1] Transition-window usability: hub output self-gated until 0.5.0 (carried from PR 3a/PR 2)
- **Location**: src/authority.rs (probe gate interval 0.5.0 ≤ v < 1.0.0) vs Cargo.toml (0.4.0)
- **Issue**: until the 0.5.0 contract release (after PR 3c, ADR-006 Decision 7), output of this build's `probe merge` (tool.version 0.4.0, now stamped schema-version 3.1) is rejected on re-input by merge/enrich/summary/project. Specced behavior, recorded so it isn't read as a bug; the 3c row's "contract-release round-trip tests over real binary outputs" close the loop at 0.5.0.

## Fixed during this audit

- kb/engineering/schema.md merged-envelope example: `schema-version` 3.0 → 3.1 and `tool.version` 0.1.0 → 0.5.0 (the hub now stamps 3.1 on merged output; a 0.1.0 tool example would be rejected by the gate the same page specifies).

## Verified clean

Checked on the changeset (commit 90c50cc), at these locations:

- **P3** stub detection structural — unchanged (`Atom::is_stub`, src/types.rs); merge/normalize use it for stub-vs-real classification only.
- **P4** μ factoring and laws — `merge_atom_maps` = per-input `normalize_atoms` → P6 conflict loop → `attach_correspondence_records` → one `enrich_verification_status` after all inputs combine (src/commands/merge.rs). Laws tested against `merge_atom_maps` itself: `test_associativity_with_mappings_across_groupings`, `test_commutativity_disjoint_keys`, `test_mapping_compatibility_laws` (F_M idempotence + `F_M(μ(A,B)) = μ(F_M(A),F_M(B))`), `test_intermediate_enrichment_does_not_change_selected_base_data` (the exact argument P4 gives for associativity).
- **P5** identity — `test_identity_exact_on_carrier` (exact, both argument positions) and `test_identity_up_to_preparation_on_legacy` (`μ(A,∅) = enrich(normalize(A))`, compared against `prepare_atoms` — the same shared primitive, so the equation is checked against the real carrier preparation, not a reimplementation).
- **P6** first-wins with stub replacement — conflict loop unchanged in semantics; every equal-key arm now also unions records (P27 carve-out), whole-atom selection otherwise intact (`test_real_vs_real_conflict_keeps_base`, `test_stub_replaced_by_real`).
- **P7** specs/proofs last-wins — untouched (`merge_generic_maps`).
- **P8** (branch text; see W1 for main) — per-input, pre-conflict-resolution normalization preserved; record targets normalized (`test_record_targets_normalized`); mapping endpoints normalized at load (`load_mappings`, src/types.rs) and re-normalized in `attach_correspondence_records` for bare-map callers; distinct-real intra-input collision rejected on merge, raw, and enrich paths with distinctness modulo records (`test_intra_input_distinct_real_collision_rejected`, `test_normalization_collision_classification`, propagate's `test_prepare_reports_collision`); all-trailing-dots fixed point retained (`test_repeated_trailing_dots_normalize_in_one_pass`). Two-input dotted-alias evidence-selection regression present (`test_two_input_dotted_alias_evidence_selection`) — the case the plan says single-input tests cannot catch.
- **P9** — structural composed detection and flatten unchanged (`test_recursive_merge_flattens_provenance` still green); dedup deferred to 3c (W2).
- **P10** — whole-atom extension preservation intact; the union carve-out touches only `maps-to`/`mapped-from` (`test_extensions_preserved`).
- **P13** — attachment unconditional (`test_dangling_target_still_attaches` — the inversion of the old existence-check test), key-local (lookup by the atom's own code-name only), set-like (`test_reapplication_is_noop`), 1-to-many (`test_one_to_many_mapping_produces_multiple_records`), `dependencies` never modified (`test_mappings_attach_records_not_edges` asserts the dependency set unchanged). Edge-injection block removed; no `dependencies.insert` remains on the mappings path.
- **P14** — record arrays sorted by the `(target, confidence, method)` triple with absent method as `""` (`sort_dedup_records`; `test_distinct_confidence_records_both_kept_sorted` pins the order); BTreeMap containers throughout.
- **P17** — category detection and same-category enforcement unchanged in `cmd_merge`.
- **P23** — merge is now a recomputation boundary: shared `enrich_verification_status` runs once post-combination (`test_merge_recomputes_enrichment` covers both the stale-downgrade and clean-promotion directions); raw staging defers recomputation only (`test_raw_path_defers_enrichment`) and still validates authority (`test_raw_path_rejects_both_projection_formats`) and `status-origin` (`test_file_paths_reject_invalid_status_origin`); records are inert to the BFS (propagate's `test_maps_to_records_do_not_contaminate`).
- **P27** — union through every equal-key case: stub replacement, real-vs-real, stub-vs-stub, real-vs-stub, and benign intra-input collisions, one test walking all five (`test_records_preserved_through_every_equal_key_case`); helper (`union_correspondence_records`) shared between the merge conflict loop and `normalize_atoms` as the plan requires.
- **Executable schema** — `correspondenceRecord` defs added (`target` required, `confidence` enum, optional `method`, `additionalProperties: false`); envelope-level validation runs over *real* `merge_atom_maps` output (`merged_envelope_with_correspondence_records_is_valid`) plus a negative test (`malformed_correspondence_records_are_rejected`).
- **Docs/architecture** — architecture.md:31 (post-merge enrichment recomputation, record attachment) now describes implemented behavior; kb/tools/probe-merge.md phases 2/4/5, stats table, and key-files row updated; schema.md normalization and status-origin enforcement rows updated; CLI help (src/main.rs) already stated record semantics; `./scripts/check-kb-links.sh` and the enum drift guard pass.
- **P1, P2, P15, P16, P19, P21, P22, P24, P25** — not touched by this changeset; spot-checked that no changed file affects them (P15's known project-side gap is W2).

Suite: 206 tests green (`cargo test --workspace`), clippy clean with `-D warnings`, `cargo fmt --check` clean.
