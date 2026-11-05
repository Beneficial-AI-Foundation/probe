---
auditor: test-quality-auditor
date: 2026-10-20
repo: probe (hub)
kb: kb/ (own KB). Property text read from the working tree and compared with `origin/main:kb/engineering/properties.md` (c159a50) per common skill §2b. The two are identical, so this audit has no merge-order constraint.
scope: Branch test/categorized-array-laws at c159a50 (equal to origin/main, version 0.6.0) plus the uncommitted working-tree changes to tests/merge_laws.rs and tests/roundtrip.rs. Round 2: the changes add two tests, extend two sweeps and one roundtrip test, and touch no file under src/. Untracked scratch files and the probe-aeneas reports are ignored. Full property matrix, re-derived from the current properties file.
status: 0 critical, 1 warning, 0 info
---

Test surface: inline `#[cfg(test)]` modules in `src/types.rs`, `src/authority.rs` and `src/commands/{merge,project,propagate,summary}.rs` (145 unit tests). Integration suites `tests/{authority,merge,merge_laws,propagate,roundtrip,schema_validation}.rs` hold 19 + 13 + 8 + 13 + 14 + 16 = 83 tests. They run the binary through `CARGO_BIN_EXE_probe` and validate output with jsonschema. The `probe-extract-check` crate adds 30 unit and 27 golden tests (8 ignored). `cargo test --workspace` on this tree: 285 passed, 0 failed, 8 ignored. `cargo clippy --all-targets --all-features -- -D warnings` is clean.

## What the working-tree change adds

Both tests live in tests/merge_laws.rs and target the 2026-09-30 report's I4. That finding said two things. First, set canonicalization of the categorized arrays was tested on three hand-picked pairs over one field, and the generated law sweep had no categorized arrays. Second, the three-alias specs/proofs warning text was never asserted.

- **`categorized_arrays_collapse_iff_equal_as_sets`** (tests/merge_laws.rs:639). The sweep covers 12 arrays, every ordered pair, all five `CATEGORIZED_DEPENDENCY_ARRAYS` fields, four stub/real combinations and both the raw and the enriched path. That is 5760 merge calls. An independent oracle (`oracle_set`, `oracle_canonical`, :578-607) predicts the outcome. Two real aliases with different normalized sets must reject with `input #1 of 1: ` and `distinct real atom`. Every other case must collapse to one atom that carries the canonical array of the expected survivor.
- **`laws_hold_over_generated_categorized_variants`** (tests/merge_laws.rs:708). It sweeps 22 variants on both paths: 18 one-key inputs and 4 two-key alias inputs. Three alias inputs hold the same set spelled differently, and one pairs a stub with a real atom. The test asserts associativity over every triple, mapping compatibility and disjoint-key commutativity over every pair, and self-merge idempotence, mapping idempotence and the fixed point for every variant. All five fields take part, and one record shape rides along, so the record union meets the set comparison.

Discrimination, argued from the code at src/commands/merge.rs:367-388 and :398-410. This analysis agrees with the caller's mutation runs. I did not re-run the mutations, because this audit edits no source file.

- **Remove `arr.dedup()`.** `["g()", "g().."]` must become `["g()"]`, so the survivor assertion fails.
- **Remove the sort.** `["h()", "g()", "h()."]` must become `["g()", "h()"]`, so the survivor assertion fails.
- **Remove name normalization in the arrays.** `["g()"]` and `["g()."]` then compare as different sets, so the two-real case rejects where the oracle expects a collapse.
- **Sort strings by JSON text.** The `"k\""` entry sorts before `"k#"` by value, but after it by JSON text, because `\` follows `#`. The array `["k#", "k\"."]` exists to catch this.
- **Mix non-strings into the string sort.** `[42, "g().", 42]` must become `["g()", 42]`, so the survivor assertion fails.
- **Make collision equality ignore extensions.** Two reals that differ only in one categorized array then collapse, and the test expects a rejection.

The law sweep compares `BTreeMap<String, Atom>` values. `Value` arrays compare in order, so any grouping-dependent order or duplicate fails it.

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures), authority.rs::project_output_uses_projected_atoms_schema_and_validates, tests/propagate.rs::test_envelope_structure_preserved, roundtrip.rs::specs_merge_same_source_dedups_and_validates, ::merged_output_passes_the_hubs_own_gate | Full | |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness holds by construction. Code-name determinism is a producer duty |
| P3 | merge.rs::test_is_stub, project.rs::test_stub_seeds_included, stub variants in every merge_laws.rs sweep, **new** stub/real combinations in categorized_arrays_collapse_iff_equal_as_sets | Full | The new sweep asserts `is_stub()` on the survivor for all four combinations |
| P4 | merge.rs law suite (associativity with mappings, commutativity, mapping compatibility, intermediate enrichment, tied identity), merge_laws.rs::laws_hold_over_generated_record_variants, ::laws_hold_with_translated_labels, ::laws_hold_with_translated_labels_and_trust, **new** ::laws_hold_over_generated_categorized_variants, roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup | Full | All three laws are now generated: associativity, commutativity on disjoint keys (via `rekey`) and both halves of law 3. The fixture tests in merge.rs remain |
| P5 | merge.rs::test_identity_exact_on_carrier, ::test_identity_up_to_preparation_on_legacy, fixed-point assertions in the translated-label sweeps, **new** fixed-point and self-merge assertions in the categorized sweep | Full | Categorized arrays now reach the fixed-point assertion, so an unsorted array that survives one merge fails it |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added, ::test_stub_resolution_keeps_copied_label, tests/merge.rs::test_atoms_* | Full | |
| P7 | merge.rs::test_generic_last_wins_on_conflict, ::test_generic_intra_input_collision_counted, tests/merge.rs::test_specs_* and test_proofs_*, roundtrip.rs::specs_intra_input_collision_warned_and_counted | Full | The two-alias and three-alias warning lines are both asserted in full at binary level |
| P8 | merge.rs normalization and collision suite, propagate.rs::test_prepare_rejects_collision, tests/propagate.rs::test_enrich_rejects_normalization_collision, project.rs::test_prepare_* (both branches), roundtrip.rs::project_rejects_distinct_real_normalization_collision, ::p15_decomposition_survives_merge_then_project, ::specs_intra_input_collision_warned_and_counted (two and three aliases), merge.rs::test_categorized_dependency_arrays_normalized, ::test_categorized_arrays_compared_as_sets, **new** categorized_arrays_collapse_iff_equal_as_sets | Full | The set clause is now tested against an oracle over all five fields, both merge paths and the stub cases. `probe enrich` and `probe project` reach the same `normalize_atoms`, and their own collision tests pin that call |
| P9 | types.rs::dedup_provenance_collapses_identical_entries_only, merge.rs::test_recursive_merge_flattens_provenance, ::test_merge_atom_files_dedups_provenance, authority.rs::aeneas_composed_envelope_inventory_survives_merge, roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup | Full | |
| P10 | merge.rs::test_extensions_preserved, roundtrip.rs::p15_decomposition_survives_merge_then_project, ::copied_label_survives_merge_project_summary, P27 union suite, **new** survivor-array assertion in the categorized sweep | Full | The new sweep pins "preserved up to canonical order" for every categorized field |
| P13 | merge.rs attachment suite (records not edges, dangling target, re-application no-op, one-to-many, distinct confidence), schema wire test, CLI end-to-end | Full | |
| P14 | merge.rs::test_full_envelope_serialization_deterministic, propagate.rs::test_deterministic_output, project.rs::test_determinism, serialized-JSON comparison in the translated-label sweeps, **new** canonical-order assertions in both categorized tests | Full | Strings-first order and the value-not-JSON-text string order are now generated, not hand-picked |
| P15 | project.rs::test_categorized_arrays_trimmed_with_dependencies, merge.rs::test_categorized_dependency_arrays_normalized, ::test_categorized_arrays_compared_as_sets, roundtrip.rs::p15_decomposition_survives_merge_then_project, **new** categorized_arrays_collapse_iff_equal_as_sets, probe-extract-check goldens | Full | The roundtrip test asserts the union equality in both directions after merge and after the project trim |
| P16 | propagate.rs recomputation suite | Full (hub scope) | Producer mapping rows belong to the producers |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader, tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | (none) | Indirect | No path dependency exists. Review enforces manifest hygiene |
| P21 | (none) | None (cross-repo) | probe-verus and probe-rust own and test it (ADR-005). Not a hub gap |
| P22 | (none) | None (consumer script) | `scripts/summarize_extract.py` has no tests (W1) |
| P23 (authority boundary, fail-closed load) | tests/authority.rs suite, src/authority.rs unit matrix, tests/propagate.rs::test_enrich_rejects_invalid_status_origin, roundtrip.rs::project_rejects_invalid_status_origin, project.rs::test_prepare_rejects_invalid_input_on_both_branches, merge.rs::test_file_paths_reject_invalid_status_origin | Full | |
| P23 (recomputation, seeds, kept copied labels) | propagate.rs suite, including test_translation_origin_never_promoted, ::test_translation_origin_label_kept, ::test_kernel_taint_transitively_verified_still_demoted, ::test_malformed_marker_transitively_verified_demoted, ::test_blueprint_translation_marker_still_relabeled, ::test_counts_include_kept_copied_labels, merge.rs::test_stub_resolution_keeps_copied_label, both translated-label sweeps | Full | Every clause of the rule has a discriminating test. The trust sweep now includes a caller `c -> r` (tests/merge_laws.rs:496-548), so a copied-trusted or translated `r` must block it |
| P23 (projection caveat, summary consumer) | roundtrip.rs::projection_recomputes_enrichment_before_trimming, ::project_inherits_labels_of_legacy_format_projection, ::copied_label_survives_merge_project_summary, project.rs::test_prepare_*, summary.rs::test_status_origin_matrix | Full | |
| P24 | (none) | None (producer-side) | Producer repos and probe-extract-check goldens test it |
| P25 | (none) | None (producer-side) | Same ownership as P24 |
| P27 | merge.rs union, identity, canonical-form and fail-closed suites, merge_laws.rs::runtime_acceptance_matches_schema_validity_at_every_boundary, ::cli_boundaries_reject_malformed_records_before_writing, roundtrip.rs::project_rejects_invalid_projected_input, schema_validation.rs::malformed_correspondence_records_are_rejected, **new** record-carrying alias inputs in the categorized sweep | Full | |

C-series known-issue entries: none in the properties file.

## Impact analysis

- **Working tree, tests/merge_laws.rs and tests/roundtrip.rs (test-only).** The change touches P4, P5, P7, P8, P10, P14 and P15 and adds tests for them. It changes no behaviour. No version bump or CHANGELOG entry is needed.
- **79d37cc, feat: keep copied Lean labels (#93).** This commit changed P23 behaviour. It shipped with discriminating unit, sweep and binary tests (P23 rows). The KB amendment is now on main, so the earlier merge-order constraint is closed.
- **60da78f, 5548433, 4f70d5b and earlier docs commits.** KB text and slide decks only. They have no test impact.

## Critical

None.

## Warnings

### [W1] `scripts/summarize_extract.py` has no tests (P22, carried)
- **Location**: scripts/summarize_extract.py
- **Issue**: P22 names this script as the place where consumers normalize `trusted-reason` values. No file under tests/ or probe-extract-check/ references it. The caller keeps it out of scope for this pull request.
- **Evidence**: `grep -rn summarize_extract tests/ probe-extract-check/` returns nothing.
- **Recommendation**: Add a smoke test over a fixture extract with one value per canonical category. Priority is low.

## Info

None.

## Resolved in round 2

- **I1 (stale module comment, `is_stub()` assertion without a message).** The module comment at tests/merge_laws.rs:1-13 no longer gives a test count or a report reference, and it names the categorized arrays. The `is_stub()` assertion (:682-686) now passes the `case` message.
- **I2 (two laws tested on fixtures only).** Both generated sweeps now assert `F_M(F_M(A)) = F_M(A)` for every variant (tests/merge_laws.rs:118-119 and :762-763). They also assert `μ(A, B) = μ(B, A)` for every pair after the new `rekey` helper (:43-48) renames the second input. The variant keys are `f`, `f.` and `f..`, so the renamed keys `g`, `g.` and `g..` share no normalized key with the first input. The atoms in both sweeps have empty `dependencies`, so renaming changes no edge. Record targets stay on `f`, so they dangle in the renamed input, which P13 permits.
- **I3 (three-alias warning text).** tests/roundtrip.rs:640-642 gives input #2 the aliases `probe:Lean.spec`, `probe:Lean.spec.` and `probe:Lean.spec..`. The test asserts the full `all normalize` warning line with input position, every key and the kept key (:665-673). It asserts `Conflicts: 3` (:675-678) and that the most-dotted alias wins (:684-688). The two-alias assertion for input #1 is unchanged. This closes the last part of the 2026-09-30 I4.

## Status of the 2026-09-30 I4

I4 is fully resolved. The oracle sweep generates the set property over all five fields, both merge paths and the stub cases. The property is "collapse iff equal as sets, reject on any set difference". The categorized law sweep puts categorized arrays and intra-input alias collapses inside every associativity, commutativity, mapping-compatibility and fixed-point assertion. The three-alias warning text is now asserted at binary level.

Three findings of the 2026-10-20 #92 report are also closed. The P16/P23 amendment is on main, so the merge-order constraint is gone. The trust sweep now has a caller `c` of `r`. The `validate_status_origins` doc comment is back in place at src/types.rs:170.

## Verified clean

P1, P3, P4, P5, P6, P7, P8, P9, P10, P13, P14, P15, P16 (hub scope), P17, P23 (all three facets) and P27 are covered. P2 and P19 are Indirect by construction or by review. P21, P24 and P25 belong to producers or other repos and are not hub gaps. P22 is W1.
