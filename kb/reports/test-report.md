---
auditor: test-quality-auditor
date: 2026-10-20
repo: probe (hub)
kb: kb/ (own KB). Branch la/keep-translated-labels. Property text compared against `origin/main:kb/engineering/properties.md` (82ec129) per common skill §2b. The heading set is identical. The working tree amends P23 and the P16 enrichment paragraph. In P23, the definition gets an exception clause. The recomputation paragraph and the `status-origin` key rule name the translation, malformed-marker and blueprint cases. A new "Copied labels are snapshots" scope caveat is added, and the extractor note changes. The new behaviour conforms only to that amendment and is reported as a merge-order constraint (I1).
scope: Round 2. Uncommitted working-tree changeset vs origin/main implementing hub issue #92 (src/commands/{propagate,merge,summary}.rs, src/types.rs, src/main.rs, tests/merge_laws.rs, tests/roundtrip.rs, KB edits, CHANGELOG, version 0.6.0). Untracked scratch files and the other two reports ignored. Full property matrix.
status: 0 critical, 1 warning, 3 info
---

Test surface: inline `#[cfg(test)]` modules in `src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` (145 unit tests), integration suites `tests/{authority,merge,merge_laws,propagate,roundtrip,schema_validation}.rs` (19 + 13 + 6 + 13 + 14 + 16 = 81, binary-level via `CARGO_BIN_EXE_probe` plus jsonschema validation), and `probe-extract-check` (30 unit + 27 golden, 8 ignored). `cargo test --workspace` on this tree: 283 passed, 0 failed, 8 ignored.

## What the #92 change does, and what discriminates it

`enrich_verification_status` now skips atoms that satisfy `is_kept_copied_label` (propagate.rs:30) as relabel candidates. That predicate is `has_translation_origin` (types.rs:173, exact string `"translation"`) on a non-blueprint atom. Seeding stays presence-based (`has_status_origin`, propagate.rs:20). Kept labels are added to the returned counts by the label they keep. Argued from the code (no source mutation run, per the edit constraint):

- **Reverting the exemption** (old unconditional rewrite) fails `test_translation_origin_label_kept` (propagate.rs:1027), `test_caller_of_translated_transitively_verified_stays_verified` (:1060), `merge.rs::test_stub_resolution_keeps_copied_label` (merge.rs:1031) and `roundtrip.rs::copied_label_survives_merge_project_summary` (roundtrip.rs:965).
- **Dropping translated atoms from the seed set** fails :1060 (the caller `f` becomes `transitively-verified`), `test_translation_origin_blocks_caller_promotion` (:992), merge.rs:1031 and roundtrip.rs:965 (on `c()`).
- **Promoting a translated `verified`** fails `test_translation_origin_never_promoted` (:963).
- **Widening the exemption to `kernel-taint`** fails `test_kernel_taint_transitively_verified_still_demoted` (:1084).
- **Widening the exemption to any present marker** fails `test_malformed_marker_transitively_verified_demoted` (:1100) on `null`, `42` and `"Translation"`. This closes round-1 W1.
- **Dropping the blueprint guard** fails `test_blueprint_translation_marker_still_relabeled` (:1122).
- **Dropping or misbucketing the kept-label count loop** fails `test_counts_include_kept_copied_labels` (:1136). It expects `(1, 2)`. Without the loop the result is `(0, 1)`.

The two generated law tests pass under the old behaviour too. They cover P4 and P5, not P23 discrimination.

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures), authority.rs::project_output_uses_projected_atoms_schema_and_validates, tests/propagate.rs::test_envelope_structure_preserved, roundtrip.rs::specs_merge_same_source_dedups_and_validates | Full | Unchanged by #92 |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural. Code-name determinism is producer-side |
| P3 | merge.rs::test_is_stub, project.rs::test_stub_seeds_included, merge_laws.rs stub variants in both #92 sweeps | Full | |
| P4 | merge.rs law suite (associativity with mappings, commutativity, mapping compatibility, intermediate enrichment, tied identity), merge_laws.rs::laws_hold_over_generated_record_variants, **new** merge_laws.rs::laws_hold_with_translated_labels (:397, 49 maps, 49^3 triples) and ::laws_hold_with_translated_labels_and_trust (:476, 30 maps, 30^3 triples), roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup | Full | Both sweeps feed merged intermediates back as inputs, so a kept label that changed with grouping fails them. The trust sweep has no caller of `r` (I2) |
| P5 | merge.rs::test_identity_exact_on_carrier, ::test_identity_up_to_preparation_on_legacy, **new** fixed-point assertion in both #92 sweeps | Full | |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added, **new** merge.rs::test_stub_resolution_keeps_copied_label, tests/merge.rs::test_atoms_* | Full | |
| P7 | merge.rs::test_generic_last_wins_on_conflict, ::test_generic_intra_input_collision_counted, tests/merge.rs::test_specs_*/test_proofs_*, roundtrip.rs::specs_intra_input_collision_warned_and_counted | Full | Unchanged by #92 |
| P8 | merge.rs normalization/collision suite, propagate.rs::test_prepare_normalizes_*, ::test_prepare_rejects_collision, project.rs::test_prepare_* (both branches), roundtrip.rs projection normalization and collision tests, ::p15_decomposition_survives_merge_then_project | Full | Unchanged by #92 |
| P9 | types.rs::dedup_provenance_collapses_identical_entries_only, merge.rs::test_recursive_merge_flattens_provenance, ::test_merge_atom_files_dedups_provenance, authority.rs::aeneas_composed_envelope_inventory_survives_merge, roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup | Full | |
| P10 | merge.rs::test_extensions_preserved, roundtrip.rs::p15_decomposition_survives_merge_then_project, P27 union suite, **new** roundtrip.rs:965 (`status-origin` must survive merge and project for summary to list `r()` as imported) | Full | |
| P13 | Carried attachment suite (records-not-edges, dangling target, re-application no-op, one-to-many, distinct confidence, schema wire test, CLI end-to-end) | Full | |
| P14 | merge.rs::test_full_envelope_serialization_deterministic, propagate.rs::test_deterministic_output, project.rs::test_determinism, **new** propagate.rs::test_translation_origin_label_kept_idempotent and both #92 sweeps (serialized JSON comparison) | Full | #92 adds no ordering-sensitive code. The count loop iterates the same BTreeMap |
| P15 | project.rs::test_categorized_arrays_trimmed_with_dependencies, merge.rs::test_categorized_dependency_arrays_normalized, ::test_categorized_arrays_compared_as_sets, roundtrip.rs::p15_decomposition_survives_merge_then_project, probe-extract-check goldens | Full | |
| P16 | propagate.rs recomputation suite | Full (hub scope) | The amended enrichment paragraph points to P23. Producer mapping rows are producer-owned |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader, tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | (none) | Indirect | No path deps. Manifest hygiene, review-enforced |
| P21 | (none) | None (cross-repo) | Owned and tested in probe-verus/probe-rust (ADR-005). Not a hub gap |
| P22 | (none) | None (consumer script) | `scripts/summarize_extract.py` has no tests (W1, carried, outside #92) |
| P23 (authority boundary, fail-closed load) | tests/authority.rs suite, src/authority.rs unit matrix, types.rs status-origin enum test, tests/propagate.rs::test_enrich_rejects_invalid_status_origin, merge.rs file-level `graph-taint` rejection, roundtrip.rs::project_rejects_invalid_status_origin, ::project_rejects_invalid_projected_input `bogus_marker`, project.rs::test_prepare_rejects_invalid_input_on_both_branches, authority.rs summary out-of-enum rejection | Full | Unchanged by #92 |
| P23 (recomputation/seeds, as amended) | propagate.rs plan-§3 suite (:963, :992, copied-trusted :1183, kernel-taint replicas, status-less marked seed, non-string marker seed and boundary :1325), **new** :1027, :1060, :1084, :1100, :1122, :1136, :1157, merge.rs:1031 | Full | Every clause of the amended rule has a discriminating test: kept translated label (clean and contaminated), translated seed blocks callers, kernel-taint and malformed demotion, blueprint relabel, snapshot after stub resolution, counts |
| P23 (projection caveat, summary consumer) | roundtrip.rs::projection_recomputes_enrichment_before_trimming, ::project_inherits_labels_of_legacy_format_projection, project.rs::test_prepare_*, summary.rs::test_status_origin_matrix, **new** roundtrip.rs::copied_label_survives_merge_project_summary | Full | The new test runs `probe merge`, then `probe project` on the merged file (authoritative branch, recomputes before trimming), then `probe summary`. It asserts `r()` stays `transitively-verified` in both outputs, the caller `c()` stays `verified`, and `r()` is listed only in `imported_verified`. Closes round-1 I3. `probe enrich` reaches the same `prepare_atoms` path and is not run separately, which is acceptable |
| P24 | (none) | None (producer-side) | Checked in producer repos and probe-extract-check goldens |
| P25 | (none) | None (producer-side) | Same ownership as P24 |
| P27 | Carried suite (union, collision classification, target normalization, fail-closed matrix, generated laws, enrichment and projection inertness), merge_laws.rs::cli_boundaries_reject_malformed_records_before_writing, roundtrip.rs::project_rejects_invalid_projected_input `malformed_record`, schema_validation.rs::malformed_correspondence_records_are_rejected | Full | Unchanged by #92 |

C-series known-issue entries: none in the properties file (origin/main or branch).

## Impact analysis

- **src/commands/propagate.rs (#92 behaviour change, blueprint guard, kept-label counts).** Touches P23, and through re-enrichment P4, P5 and P14. Each behaviour has a test added alongside (see the discrimination list).
- **src/types.rs, src/commands/summary.rs.** `has_translation_origin` is extracted and shared. Summary behaviour is unchanged and pinned by summary.rs::test_status_origin_matrix and roundtrip.rs:965. See I3 for a doc-comment side effect.
- **src/main.rs.** Help text only.
- **KB, CHANGELOG, Cargo.toml (0.6.0).** No test impact. The P16/P23 amendments must ship with the code (I1).
- **Recent commits (82ec129 and earlier, PRs #85, #88, #91).** KB and docs only. No behaviour change.

## Critical

None.

## Warnings

### [W1] `scripts/summarize_extract.py` untested (P22, carried, outside #92)
- **Location**: scripts/summarize_extract.py
- **Issue**: The P22 consumer-side normalization has no tests. Nothing under tests/ references the script. This predates #92.
- **Recommendation**: Low priority: a smoke test over a fixture extract covering one value per canonical category.

## Info

### [I1] Merge-order constraint: P16/P23 amendments and code must ship together
`origin/main` P23 says a `status-origin`-bearing atom arriving as `transitively-verified` is "rewritten to `verified` **unconditionally**". The new code and the new tests (propagate.rs:1027, :1060, :1122, :1136, :1157, merge.rs:1031, roundtrip.rs:965) conform only to the uncommitted amendment in kb/engineering/properties.md and the matching glossary, schema and ADR-006 edits. Keep the KB edits and src/ changes in the same PR for #92. The amended P23 extractor note and the CHANGELOG also depend on the probe-aeneas release that refreshes copied labels (0.22.0, probe-aeneas#78).

### [I2] The trust law sweep cannot observe copied-trusted or translated-seed blocking
- **Location**: tests/merge_laws.rs:476-560
- **Issue**: The only edge is `r → s`, so nothing depends on `r`. The copied-`trusted` and translated variants of `r` are never in a position to shield or block a caller. Only plain `trusted` `s` behind unmarked `verified` `r` is observable. The doc comment says the sweep covers "copied `trusted` (a seed, never a boundary)", but under P4 that role is not exercised. The unit test propagate.rs:1183 does cover it for P23.
- **Recommendation**: Add a third key `c` that depends on `r`. Give it a small vocabulary: absent, stub, unmarked `verified`. Or narrow the doc comment to what the sweep exercises. Check the runtime, because the two #92 sweeps already take about 6.6 s together.

### [I3] `validate_status_origins` lost its doc comment (code quality, not test coverage)
- **Location**: src/types.rs:159-180
- **Issue**: `has_translation_origin` was inserted between the `validate_status_origins` doc comment and the function. The `///` block that starts "Validate every atom's `status-origin` marker" now attaches to `has_translation_origin`. The public `validate_status_origins` is left undocumented.
- **Recommendation**: Move `has_translation_origin` (with its own doc lines and `@kb` annotation) above the `// @kb: kb/engineering/schema.md#common-optional-fields` line.

## Verified clean

P1, P3, P4, P5, P6, P7, P8, P9, P10, P13, P14, P15, P16 (hub scope), P17, P23 (all three facets, as amended) and P27 are covered. For #92, P23 is covered on every clause of the amended rule, including the malformed-marker fail-closed half, the blueprint guard and the projection and summary legs at binary level. P4, P5 and P14 are covered by two generated sweeps and the idempotence test. P2 and P19 are Indirect by construction or review. P21, P24 and P25 are owned by producers or other repos and are not hub gaps. P22 is W1.

Closed since round 1: W1 (malformed-marker demotion, by propagate.rs:1100), I3 (no binary-level test, by roundtrip.rs:965). Round-1 I2 is partly closed by merge_laws.rs:476: unmarked `verified` and plain `trusted` are now swept, and the remaining gap is the new I2. Carried from 2026-09-30 and unchanged by #92, not re-listed: the leanblueprint gate floor pinned at unit level only, unreachable defensive guards in `push_record`/`union_correspondence_records`, and set canonicalization exercised only on hand-picked instances.
