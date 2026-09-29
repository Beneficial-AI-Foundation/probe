---
auditor: test-quality-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB; this branch amends P8 ¶2 and P27 bullet 1 — the coverage rows for those two judge the tests against the branch text, which is the semantics this PR implements; the quality report records the merge-order constraint)
scope: branch la/merge-soundness-pr3b-maps-to-records vs origin/main (merge-soundness PR 3b — correspondence records, merge re-enrichment, raw staging, P8 reconciliation), full property matrix
status: 0 critical, 2 warnings, 3 info
---

Test surface: inline `#[cfg(test)]` modules (`src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` — 120 unit tests), integration suites `tests/{merge,propagate,authority,schema_validation}.rs` (58 tests, binary-level via `CARGO_BIN_EXE_probe` plus jsonschema validation), and the `probe-extract-check` crate (separate scope: extract-vs-source validation).

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures incl. the new merged-with-records envelope), authority.rs::project_output_uses_projected_atoms_schema_and_validates; tests/propagate.rs::test_envelope_structure_preserved | Full | |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural; code-name determinism is producer-side |
| P3 | merge.rs::test_is_stub, project.rs::test_stub_seeds_included | Full | |
| P4 | merge.rs::test_associativity_with_mappings_across_groupings (both nestings vs flat 3-way, with mappings, over stub replacement + real-vs-real + dangling-then-resolved targets), ::test_commutativity_disjoint_keys, ::test_mapping_compatibility_laws (F_M idempotence + F_M(μ(A,B)) = μ(F_M(A),F_M(B))), ::test_intermediate_enrichment_does_not_change_selected_base_data (the exact associativity argument: label rewrite in an intermediate does not change P6 selection) | Full (example-based) | Laws run against `merge_atom_maps` itself, the μ the property defines. Hand-picked fixtures, not generated inputs (I1) |
| P5 | merge.rs::test_identity_exact_on_carrier (both argument positions, records included), ::test_identity_up_to_preparation_on_legacy (compared against the real `prepare_atoms`, not a reimplementation) | Full | |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added, tests/merge.rs::test_atoms_* | Full | Unit + binary paths |
| P7 | merge.rs::test_generic_last_wins_on_conflict, tests/merge.rs::test_specs_*/test_proofs_* | Full | |
| P8 | merge.rs::test_trailing_dot_normalization, ::test_normalization_collision_classification (distinct-real vs stub vs identical vs identical-modulo-records), ::test_intra_input_distinct_real_collision_rejected (Err on merge — first input, subsequent input, and raw path), ::test_repeated_trailing_dots_normalize_in_one_pass, ::test_record_targets_normalized, ::test_dependencies_with_locations_normalized (new), ::test_load_mappings_normalizes_endpoints, ::test_two_input_dotted_alias_evidence_selection (per-input-before-conflict-resolution ordering — the case single-input tests cannot catch); propagate.rs::test_prepare_normalizes_before_enrichment, ::test_prepare_normalizes_keys, ::test_prepare_reports_collision; tests/propagate.rs::{test_dotted_alias_contamination_end_to_end, test_enrich_rejects_normalization_collision, test_enrich_accepts_identical_duplicate_and_stub_collisions} | Partial | Both boundaries (merge per-input, enrich) covered for the rejection, positive and negative halves. Remaining gap is the PR 3c scope: extended arrays (`requires-dependencies` etc.) and the project boundary are not yet normalized, so not yet testable |
| P9 | merge.rs::test_recursive_merge_flattens_provenance, ::test_generic_recursive_merge_flattens_provenance, types.rs::parse_envelope_* matrix, authority.rs::aeneas_composed_envelope_inventory_survives_merge | Full (current scope) | Dedup clause is PR 3c and unimplemented |
| P10 | merge.rs::test_extensions_preserved; enrichment writes only `verification-status` (pinned indirectly by every status-origin test) | Full | Record union is the P27 carve-out, tested there |
| P13 | merge.rs::test_mappings_attach_records_not_edges (records attached, `dependencies` asserted unchanged on every atom), ::test_dangling_target_still_attaches (the inversion of the retired existence-check guard), ::test_reapplication_is_noop (set-like), ::test_one_to_many_mapping_produces_multiple_records, ::test_distinct_confidence_records_both_kept_sorted; schema_validation.rs::merged_envelope_with_correspondence_records_is_valid (wire shape from real merge output) | Full | The PR-2 audit's W1 (tests pinning superseded edge-injection) is resolved: those tests were replaced wholesale |
| P14 | merge.rs::test_full_envelope_serialization_deterministic (new — full envelope JSON, fixed meta, two independent builds byte-identical), ::test_distinct_confidence_records_both_kept_sorted (record sort order pinned exactly); propagate.rs::test_deterministic_output, project.rs::test_determinism | Full | Closes the carried W2 (struct-level-only determinism) |
| P15 | probe-extract-check golden/properties tests (extract side) | Partial | Hub-side projection trim of categorized arrays is PR 3c and untested |
| P16 | propagate.rs recomputation suite (see P23 row) | Full (hub scope) | Producer mapping rows are producer-owned |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader, tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | — | Indirect | No path deps; manifest hygiene, review-enforced |
| P21 | — | None (cross-repo) | Implemented and tested in probe-verus/probe-rust (ADR-005 ownership) |
| P22 | — | None (consumer script) | `scripts/summarize_extract.py` has no tests (W2, carried) |
| P23 (authority boundary) | tests/authority.rs suite; src/authority.rs unit matrix; **new**: merge.rs::test_raw_path_rejects_both_projection_formats (raw staging entry point), ::test_file_paths_reject_invalid_status_origin (merge is now a recomputation boundary: both file-level paths fail closed on out-of-enum markers) | Full | |
| P23 (recomputation/seeds) | propagate.rs plan-§3 letter suite (a–d, g–k) + retained chain/diamond/cycle/idempotence suite; tests/propagate.rs end-to-end trio; **new** "merge re-enriches" clause (plan test e): merge.rs::test_merge_recomputes_enrichment (stale downgrade after stub resolution + clean-leaf promotion, enrichment counts in stats), ::test_raw_path_defers_enrichment (negative half: raw output carries stale statuses) | Full | Library + CLI paths; both halves |
| P24 | — | None (producer-side) | Validated in producer repos + probe-extract-check goldens |
| P25 | — | None (producer-side) | Same ownership as P24 |
| P27 | merge.rs::test_records_preserved_through_every_equal_key_case (one test walking all five resolutions: stub replacement, real-vs-real, stub-vs-stub, real-vs-stub, benign intra-input collision), ::test_normalization_collision_classification (identical-modulo-records collapse unions), ::test_record_targets_normalized, ::test_distinct_confidence_records_both_kept_sorted (identity triple, absent-method ordering, distinct assertions kept); propagate.rs::test_maps_to_records_do_not_contaminate (inert to enrichment); schema_validation.rs::malformed_correspondence_records_are_rejected (wire constraints: enum confidence, required target, no extra fields) | Full (current scope) | Projection-BFS inertness is structurally true today (projection walks `dependencies` only) but unpinned; natural home is PR 3c's projection work (I3) |

ADR-006 Decision 9 (summary consumer contract): unchanged from the PR 2 audit — summary.rs matrix suite still green.

## Impact analysis (this changeset)

Every behavior change landed with tests:

- Correspondence-record attachment replacing edge injection → the P13 row's five attachment tests plus the envelope-level wire test; the superseded edge-injection tests were replaced, not left pinning dead semantics.
- Record union on every equal-key case → single walking test + the shared-helper collision cases (P27 row).
- Merge re-enrichment (μ on the carrier) → test e pair (recompute + raw-defers), the four law tests, and the identity pair.
- `load_mappings` signature change (full records, normalized endpoints) → loader tests rewritten (`test_duplicate_from_keys_preserved` now also covers `endpoint_lookup_maps`, which `probe project` consumes; `test_load_mappings_normalizes_endpoints` new).
- P8 reconciliation (merge rejects intra-input distinct-real collisions) → rejection tested in the first input, a subsequent input, and on the raw path; the benign-collapse and zero-conflict negatives retained.
- Raw staging primitives → projection-rejection and status-origin rejection tests on `merge_atom_files_raw`; enrichment-deferral negative.
- Merged envelopes stamped 3.1 → tests/merge.rs schema-version assertions updated (they would have caught a silent stamp change; they did — three failed until updated deliberately).

Unpinned interim state, deliberate: the projection-seed regression window (normalized mapping endpoints vs still-exact-key project seed matching) is not tested because it is scheduled to be *fixed* in PR 3c, and a test pinning the wrong interim behavior would have to be deleted there (quality report W2).

## Critical

None. (P4/P5/P13/P27 — previously the starred no-coverage rows — are now covered; remaining gaps are the PR 3c-scheduled clauses, tracked per-row.)

## Warnings

### [W1] P15 hub-side projection trim untested (carried; PR 3c scope)
- **Location**: src/commands/project.rs (categorized arrays cloned untrimmed)
- **Issue**: the P15 decomposition break in projection output has no regression test; the fix and test are the plan's 3c row.

### [W2] `scripts/summarize_extract.py` untested (P22, carried)
- **Recommendation**: low priority; smoke test over a fixture extract.

## Info

### [I1] Property-based testing opportunity (carried, now concrete)
The P4/P5 law tests are example-based over one fixture family (`law_fixtures`). A proptest generating random atom maps + mapping sets and asserting the four laws would generalize them; the fixtures already encode the tricky shapes (stub replacement, conflicts, dangling targets), so this is an upgrade, not a gap.

### [I2] Gate thresholds asserted in one place only (carried from PR 3a)
Acceptable; unchanged this branch.

### [I3] Projection-side record inertness and mixed valid+ghost seeds untested (carried, projection scope)
Both belong to PR 3c's projection work (normalize-then-enrich-then-trim, seed matching over normalized keys).

Totals: 120 unit tests, 58 binary/integration tests, all green; clippy `-D warnings` clean.
