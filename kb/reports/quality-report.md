---
auditor: code-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB; branch amends P8/P27 wording — those audited against BOTH the branch text and origin/main's, see W1; all other property text is identical to origin/main)
scope: branch la/merge-soundness-pr3b-maps-to-records vs origin/main (merge-soundness PR 3b — correspondence records, merge re-enrichment, raw staging primitives, P8 intra-input collision reconciliation), plus the working-tree review-fix delta (record canonical form + triple dedup, fail-closed record/mapping validation, prepare_atoms Result, records_attached rename — cross-model review of PR #81)
status: 0 critical, 2 warnings, 1 info
---

## Critical

None.

## Warnings

### [W1] P8/P27 amendments are a merge-order constraint (deliberate, plan-authorized)
- **Location**: kb/engineering/properties.md P8 ¶2 and P27 bullets 1–4 (this branch) vs `git show origin/main:kb/engineering/properties.md`
- **Issue**: against origin/main's P8, merge on a distinct-real intra-input collision "warns and keeps first-wins"; this branch's code **rejects** it (src/commands/merge.rs `merge_atom_maps_raw`, `collision_error`) and amends P8/P27 to match. The review-fix delta adds two further P27 amendments the code conforms to only on this branch: the canonical-form clause (`method: ""` ≡ absent; dedup by the identity triple) and the fail-closed record-shape/mapping-confidence validation clause. Code conforms only to the unmerged amendments.
- **Evidence**: the P8 amendment is the reconciliation the plan's §9 row 3b explicitly mandates ("extend rejection to merge's per-input normalization or record the asymmetry in §10") — commit 4d1dbb3 recorded the obligation; the branch records the decision in ADR-006 Decision 3, plan §10, and P8. Rationale: once μ re-enriches internally, warn-and-count lets first-wins-selected evidence feed enrichment inside merge — the laundering path main's own P8 clause condemns at the unary boundary. The P27 canonical-form/validation amendments resolve the PR #81 cross-model review's two P1 findings (both reproduced as failing tests before the fix: tied-identity records broke associativity and set-dedup; a malformed non-array record field lost valid records grouping-dependently) — human-authorized, recorded in ADR-006 Decision 1 and plan §10.
- **Recommendation**: none beyond merging this PR as one unit (spec amendments + implementation + tests land together). Reviewers should confirm the rejection choice over the recorded-asymmetry alternative and the canonical-form decision (`""` ≡ absent).

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

## Review-fix delta (2026-09-30, PR #81 cross-model review)

Checked in addition to the base changeset below; spec (P27, schema.md, ADR-006
Decision 1, probe-merge.md Phase 2, glossary) and code moved together under
explicit human authorization (see W1):

- **P27 canonical form** — `record_key`/`sort_dedup_records` dedup by the
  identity triple (`dedup_by`, first occurrence wins in sorted order);
  `method: ""` canonicalized to absent in `validate_and_canonicalize_records`
  (input atoms), `push_record` (attachment), and `load_mappings` (file load);
  the executable schema's `correspondenceRecord.method` gained `minLength: 1`.
  Regressions: `test_empty_method_canonicalized_to_absent` (all three paths +
  self-merge idempotence), `test_tied_identity_records_keep_associativity`
  (the review's P4 counterexample, now associative),
  `test_records_equal_after_target_normalization_collapse`.
- **P27 fail-closed validation** — `validate_and_canonicalize_records` runs
  inside `normalize_atoms`, so every recomputation boundary gets it: merge per
  input (`merge_atom_maps_raw`, both first and subsequent inputs), enrich via
  `prepare_atoms` (now `Result`-returning; `cmd_enrich` rejects). Rejected
  shapes: non-array field, non-object entry, unexpected extra field, missing/
  non-string `target`, out-of-vocabulary `confidence`, non-string `method`
  (`test_malformed_record_shapes_rejected` — one case per arm — and
  propagate's `test_prepare_rejects_malformed_correspondence_records`). This
  closes the review's grouping-dependent evidence-loss counterexample (a
  malformed non-array field can no longer swallow a union).
- **Mapping confidence validation** — `MAPPING_CONFIDENCE_VALUES`
  (src/types.rs) validated in `load_mappings` (file boundary, error names the
  file) and `merge_atom_maps_raw` (in-memory `Mapping` values,
  `validate_mappings`); pinned against the executable schema enum by
  `confidence_vocabulary_matches_executable_schema`. CLI rejection covered
  end-to-end (`test_merge_rejects_invalid_mapping_confidence_via_cli`, exits
  non-zero, no output written).
- **CLI `--mappings` end-to-end** — new binary-level test
  (`test_merge_with_mappings_attaches_records_via_cli`): records attached both
  directions, endpoints normalized at load, dangling target attached anyway,
  `dependencies` byte-equal to the input's.
- **P14** — the envelope-determinism test now varies record-array and
  mappings-file input order (`test_full_envelope_serialization_deterministic`),
  exercising canonicalization rather than only BTreeMap ordering.
- **Per-input error context** — merge rejections are prefixed `input #N of M`
  (`merge_atom_maps_raw::input_context`; asserted in
  `test_malformed_record_shapes_rejected` and the collision tests), matching
  probe-merge.md Phase 2.
- **Rename** — `MergeStats.mappings_applied` → `records_attached` (doc comment
  states the 0–2-per-mapping semantics); no KB or docs surface named the old
  field (checked by grep; the stats-table label "Records attached" was already
  the printed form).
- **Docs accuracy** — schema.md § Correspondence records (canonical form, "no
  other fields"), § Mappings file format (load-boundary validation), P27,
  probe-merge.md Phase 2, glossary `correspondence record`, ADR-006 Decision 1
  consequences, and CHANGELOG all state the implemented behavior; KB link check
  and enum drift guard pass.

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

Suite: 249 tests green (`cargo test --workspace`, review-fix delta included), clippy clean with `-D warnings`, `cargo fmt --check` clean.
