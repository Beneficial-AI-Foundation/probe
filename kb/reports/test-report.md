---
auditor: test-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB). Property text audited against `origin/main:kb/engineering/properties.md` (4b633e0), per common skill §2b. The tree is on la/merge-soundness-pr3c-collision-warnings-projection (PR #83), which amends five property bodies — P7 (intra-input tie-break is key order, not input order), P8 (`probe project` rejects collisions on any input; categorized dependency arrays are sets in canonical order; specs/proofs collision warning names input position, both keys and the kept key), P10 (preservation is "up to P8 normalization", including canonical ordering of categorized arrays), P15 (categorized subsets are sets, canonically sorted) and P27 (fail-closed record validation extends to `probe project` on any input). The property heading set is identical to origin/main. Code that conforms only to those amendments is reported as a merge-order constraint (I3), not as clean.
scope: PR #83 at 903c1b8 plus the uncommitted working-tree edits — new binary-level test `tests/roundtrip.rs::p15_decomposition_survives_merge_then_project` (with helpers `string_set` and `assert_p15_decomposition`); `tests/merge_laws.rs::cli_merge_and_enrich_reject_malformed_records_before_writing` renamed to `cli_boundaries_reject_malformed_records_before_writing` and extended to `probe project`; a `malformed_record` case in `tests/roundtrip.rs::project_rejects_invalid_projected_input`; a CHANGELOG.md wording fix (no test impact). Untracked pr-*-review-comments.md files ignored. Full property matrix.
status: 0 critical, 1 warning, 4 info
---

Test surface: inline `#[cfg(test)]` modules (`src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` — 134 unit tests), integration suites `tests/{authority,merge,merge_laws,propagate,roundtrip,schema_validation}.rs` (19 + 13 + 4 + 13 + 13 + 16 = 78 tests, binary-level via `CARGO_BIN_EXE_probe` plus jsonschema validation), and the `probe-extract-check` crate (separate scope: 30 unit + 27 golden, 8 ignored). `cargo test --workspace` on this tree: 269 passed, 0 failed, 8 ignored. The roundtrip count rose 12 → 13 with the new P15 test; the P27 project-boundary additions extend existing tests and do not change the counts (re-checked with `cargo test -q --test merge_laws --test roundtrip`: 4 and 13 passed).

Mutation evidence for the new P15 test was supplied by its author, not re-run in this audit: disabling the categorized-array trim loop in `src/commands/project.rs:134-145`, or the categorized-array normalization loop in `src/commands/merge.rs:367-386`, each makes it fail. That is consistent with the assertions: merged `f()`'s `requires-dependencies` starts as `["probe:app/1.0/g()."]` while its `dependencies` normalizes to `g()`, so an un-normalized subset breaks the union equality, and an untrimmed `g()` keeps `k()` in `ensures-`/`body-dependencies` while `dependencies` is trimmed to `[]`.

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures); authority.rs::project_output_uses_projected_atoms_schema_and_validates; tests/propagate.rs::test_envelope_structure_preserved; roundtrip.rs::specs_merge_same_source_dedups_and_validates | Full | |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural; code-name determinism is producer-side |
| P3 | merge.rs::test_is_stub; project.rs::test_stub_seeds_included | Full | |
| P4 | merge.rs::test_associativity_with_mappings_across_groupings, ::test_commutativity_disjoint_keys, ::test_mapping_compatibility_laws, ::test_intermediate_enrichment_does_not_change_selected_base_data, ::test_tied_identity_records_keep_associativity; tests/merge_laws.rs::laws_hold_over_generated_record_variants (generated); roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup | Full (example + generated + binary) | Categorized-array canonicalization is idempotent and per-input, so it cannot change which atom a later conflict selects. The generated sweep does not generate categorized arrays (tests/merge_laws.rs has none), so that is argued, not exercised (I4) |
| P5 | merge.rs::test_identity_exact_on_carrier, ::test_identity_up_to_preparation_on_legacy | Full | |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added; tests/merge.rs::test_atoms_* | Full | |
| P7 | merge.rs::test_generic_last_wins_on_conflict; tests/merge.rs::test_specs_*/test_proofs_*; merge.rs::test_generic_intra_input_collision_counted (two- and three-alias cases, merge.rs:2346-2373); roundtrip.rs::specs_intra_input_collision_warned_and_counted | Full vs origin/main | origin/main P7 governs cross-input order, which the carried tests pin. The intra-input key-order sentence is an amendment (I3). Both intra-input tests assert the dotted/most-dotted alias is kept, so a switch to first-wins within an input fails them |
| P8 | Carried suite (trailing-dot normalization, dependencies-with-locations, record targets, mapping endpoints, intra-input distinct-real rejection on merge first/subsequent/raw inputs with `input #N of M` prefixes, two-input dotted-alias evidence selection; tests/propagate.rs::test_enrich_rejects_normalization_collision and ::test_enrich_accepts_identical_duplicate_and_stub_collisions); merge.rs::test_normalization_collision_classification; propagate.rs::test_prepare_rejects_collision; roundtrip.rs::project_rejects_distinct_real_normalization_collision, ::project_rejects_invalid_projected_input `collision` case, ::project_normalizes_keys_of_projected_input, ::projection_seeds_match_normalized_keys; merge.rs::test_categorized_dependency_arrays_normalized, ::test_categorized_arrays_compared_as_sets; roundtrip.rs::specs_intra_input_collision_warned_and_counted (full warning string for input #1 at :656-663, `input #2 of 2` at :664-667, `Conflicts: 2` at :669-672); **new** roundtrip.rs::p15_decomposition_survives_merge_then_project (dotted, duplicated, unsorted categorized entries normalized through the `probe merge` binary; sorted+deduped `body-dependencies` asserted at :918-922) | Full vs origin/main and on the amended clauses | Project rejection on projected views, categorized arrays as sets, and the warning contents are amendment-only (I3). The three-alias warning's `all … normalize` wording (merge.rs:651) is not asserted; the count and kept key are (I4) |
| P9 | types.rs::dedup_provenance_collapses_identical_entries_only; merge.rs::test_recursive_merge_flattens_provenance, ::test_merge_atom_files_dedups_provenance; authority.rs::aeneas_composed_envelope_inventory_survives_merge; roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup, ::specs_merge_same_source_dedups_and_validates | Full | |
| P10 | merge.rs::test_extensions_preserved; record union tested under P27; **new** roundtrip.rs::p15_decomposition_survives_merge_then_project (all five categorized extension fields must be present as arrays after `probe merge` and `probe project`, since `string_set` panics on a missing or non-array field) | Full vs origin/main | Canonical reordering/dedup of categorized extensions is sanctioned only by the P10/P8 amendments (I3) |
| P13 | Carried attachment suite (records-not-edges, dangling target, re-application no-op, one-to-many, distinct confidence; schema wire test; CLI end-to-end) | Full | |
| P14 | merge.rs::test_full_envelope_serialization_deterministic, ::test_distinct_confidence_records_both_kept_sorted, ::test_categorized_arrays_compared_as_sets; propagate.rs::test_deterministic_output; project.rs::test_determinism; **new** roundtrip.rs::p15_decomposition_survives_merge_then_project asserts the written categorized array is sorted and deduplicated | Full | Strings-before-non-strings ordering pinned by test_categorized_dependency_arrays_normalized (`arr[0] == "g()"`, `arr[1] == 42`) |
| P15 | project.rs::test_categorized_arrays_trimmed_with_dependencies (trim, non-string untouched); merge.rs::test_categorized_dependency_arrays_normalized, ::test_categorized_arrays_compared_as_sets; **new** roundtrip.rs::p15_decomposition_survives_merge_then_project; probe-extract-check golden/properties tests (extract side) | Full | The new test computes `⋃ subsets` and asserts equality with `dependencies` (both directions — never superset or subset) for Verus-shaped (`requires/ensures/body`) and Lean-shaped (`type/term`) atoms, after merge normalization and again after the `--forward-depth 1` trim, with an excluded callee on each side (`k()`, `Lean.C`) asserted absent (:942-947). Both hub mutation sites are covered at binary level |
| P16 | propagate.rs recomputation suite (see P23 rows) | Full (hub scope) | Producer mapping rows are producer-owned |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader; tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | — | Indirect | No path deps; manifest hygiene, review-enforced |
| P21 | — | None (cross-repo) | Implemented and tested in probe-verus/probe-rust (ADR-005 ownership); not a hub gap |
| P22 | — | None (consumer script) | `scripts/summarize_extract.py` has no tests (W1) |
| P23 (authority boundary) | tests/authority.rs suite; src/authority.rs unit matrix; raw-path/status-origin rejections; roundtrip.rs::merged_output_passes_the_hubs_own_gate, ::projection_readable_by_consumers_rejected_by_recomputation, ::project_rejects_invalid_status_origin, ::project_rejects_invalid_projected_input `bogus_marker` case | Full | |
| P23 (recomputation/seeds) | propagate.rs plan-§3 letter suite, chain/diamond/cycle/idempotence, merge-re-enriches pair, raw-defers negative; propagate.rs::test_prepare_rejects_collision | Full | |
| P23 (projection caveat: enrich-before-trim, inherited labels) | roundtrip.rs::projection_recomputes_enrichment_before_trimming (re-projects its own view and asserts `f()` stays `verified` with its blocker absent), ::projection_readable_by_consumers_rejected_by_recomputation, ::project_inherits_labels_of_legacy_format_projection; tests/authority.rs::project_reads_an_already_projected_input | Full | Both projection formats pinned by assertions that fail under recomputation |
| P24 | — | None (producer-side) | Validated in producer repos and probe-extract-check goldens; not a hub gap |
| P25 | — | None (producer-side) | Same ownership as P24 |
| P27 | Carried suite (union through every equal-key case, collision classification, target normalization, identity/canonical-form/fail-closed matrix, generated laws, enrichment and projection inertness); merge.rs::test_malformed_record_shapes_rejected; propagate.rs::test_prepare_rejects_malformed_correspondence_records; tests/merge_laws.rs::cli_boundaries_reject_malformed_records_before_writing (merge, enrich and project × six malformed shapes × both fields: non-zero exit, no output, field named on stderr); roundtrip.rs::project_rejects_invalid_projected_input `malformed_record` case (out-of-vocabulary `confidence` in `maps-to` on an already-projected view); schema_validation.rs::malformed_correspondence_records_are_rejected | Full, including the amended clause | Both `probe project` branches are now pinned at binary level: the authoritative branch (`prepare_atoms`) by the merge_laws loop over an extract fixture, the projected branch (`normalize_atoms`) by the roundtrip case over a real projection. The `probe project` clause is amendment-only (I3) |

C-series known-issue entries: none in the properties file (origin/main or branch).

ADR-006 Decision 9 (summary consumer contract): unchanged; summary.rs matrix suite green.

## Impact analysis

- **5623ae9, b33d9d0, 903c1b8 (PR #83 commits).** Each touches src/ and adds or updates tests/roundtrip.rs alongside (529, 227 and 58 changed lines respectively), plus unit tests in merge.rs/project.rs/propagate.rs. The behaviours they introduce — collision warnings, provenance dedup, enrich-then-trim projection, project validating and normalizing every input, strict `normalize_atoms`, categorized arrays as sets — are each pinned by a test listed in the matrix.
- **Working tree: `p15_decomposition_survives_merge_then_project`.** Test-only. Categorized arrays now reach binary-level fixtures, and the P15 union equality is asserted directly after both transformations the preservation clause names (merge and project).
- **Working tree: P27 project-boundary tests.** Test-only. `cli_boundaries_reject_malformed_records_before_writing` (tests/merge_laws.rs:268-318) adds `"project"` to its command loop with `--mappings` pointing at an empty mappings file; `project_rejects_invalid_projected_input` gains `malformed_record` (tests/roundtrip.rs:394-396, :405). Closes this audit's earlier W1.
- **Working tree: CHANGELOG.md wording.** No test impact.
- **KB example.** kb/tools/probe-merge.md:51 still matches the format string at merge.rs:652-656 and the roundtrip needle at tests/roundtrip.rs:657-661.

## Critical

None.

## Warnings

### [W1] `scripts/summarize_extract.py` untested (P22, carried)
- **Location**: scripts/summarize_extract.py
- **Issue**: The P22 consumer-side normalization (`TRUST_LABELS`, `TOOL_CONFIG`) has no tests; nothing under tests/ or scripts/ references it.
- **Recommendation**: Low priority: a smoke test over a fixture extract that covers one value per canonical category.

## Info

### [I1] probe-leanblueprint gate floor pinned at unit level only (carried)
src/authority.rs::gated_producers_below_threshold_rejected_at_threshold_accepted (authority.rs:315) hardcodes every `GATE_FLOORS` entry, so a typo in the table is caught. The binary suite in tests/authority.rs double-pins probe-lean, probe-aeneas, probe-vcvio (:249, :296) and the probe interval, but has no probe-leanblueprint case. Acceptable as is.

### [I2] Defensive non-array guards in `push_record`/`union_correspondence_records` unreachable on merge paths (carried)
merge.rs:166-170 and merge.rs:215-221. Per-input validation guarantees arrays before any union or attachment, so these branches are untested by design; they exist for direct library callers.

### [I3] Merge-order constraint: the P7/P8/P10/P15/P27 amendments and the code ship together in PR #83 (carried, extended)
These behaviours conform only to the PR #83 amendment text, not to `origin/main`: rejecting a distinct-real collision on an already-projected `probe project` input (P8); sorting and deduplicating the categorized arrays, which rewrites P10-preserved extensions beyond plain normalization (P8, P10, P15); the specs/proofs warning contents (P8); the key-order intra-input tie-break (P7, P8); and rejecting malformed records at `probe project` (P27). The new tests' sorted-output assertion (tests/roundtrip.rs:918-922) and `probe project` record rejections (tests/merge_laws.rs:297-311, tests/roundtrip.rs:405) also pin amendment-only behaviour. None of these contradicts origin/main text. The KB edits (kb/engineering/properties.md, kb/tools/probe-merge.md, kb/tools/probe-project.md, kb/engineering/schema.md, kb/engineering/glossary.md) and the src/ changes are all in PR #83 and must not be split. The checked-in report listed only the P7/P8 amendments; P10, P15 and P27 are also amended on this branch (`git diff origin/main -- kb/engineering/properties.md`).

### [I4] Set canonicalization is exercised on hand-picked instances; property-based opportunity (carried, narrowed)
"Aliases whose categorized arrays differ only in order or duplicates collapse; any set difference rejects" is tested on three hand-picked pairs over one field (merge.rs:2756-2796), and the generated merge-law sweep (tests/merge_laws.rs) never generates categorized arrays. This is a natural generated-input property: permute and duplicate a random set across aliases and compare the collapse/reject outcome with set equality, over all five `CATEGORIZED_DEPENDENCY_ARRAYS`. Separately, the three-alias warning path (`all … normalize`, merge.rs:651) runs in merge.rs:2366-2373 but its stderr text is never asserted; diagnostic-only.

## Resolved since the checked-in report

- **Checked-in W1 (specs/proofs warning position pinned only for input #1)** — resolved. The second input of specs_intra_input_collision_warned_and_counted carries its own aliased pair (tests/roundtrip.rs:639-642), and the test asserts `input #2 of 2: normalization collision: 'probe:Lean.spec' and` (tests/roundtrip.rs:664-667) plus `Conflicts: 2` (:669-672). An off-by-one at merge.rs:681 would now fail it.
- **Checked-in I1 (categorized arrays never reach a binary-level fixture; P15 union never asserted)** — resolved by tests/roundtrip.rs:827-959 (working tree), as detailed in the P15 row.
- **Checked-in I5, three-alias clause** — dropped as incorrect: the KB's `f()`, `f().`, `f()..` example is tested at merge.rs:2366-2373 (2 conflicts, most-dotted kept). The remainder is carried as I4.
- **This audit's earlier W1 (P27 fail-closed record validation at `probe project` had no project-boundary test)** — resolved in the working tree. tests/merge_laws.rs:297 now loops over `["merge", "enrich", "project"]`, passing `--mappings` for project (:304-306), and asserts non-zero exit, no output and the field name on stderr for all six malformed shapes in both `maps-to` and `mapped-from` (:308-311); this drives the authoritative `prepare_atoms` branch. tests/roundtrip.rs:394-396 builds a real projection with `maps-to: [{target: probe:Lean.f, confidence: bad}]` and :405 expects rejection naming `maps-to` with no output, which drives the projected `normalize_atoms` branch. Both suites pass.

## Verified clean

P1, P3, P4, P5, P6, P9, P13, P14, P15, P16 (hub scope), P17, P23 (all three facets) and P27 were checked and found covered against the origin/main text. P7, P8, P10 and P27 are also covered on their amended clauses, with the amendment-only parts flagged in I3. P2 and P19 are Indirect by construction or review. P21, P24 and P25 are producer- or cross-repo-owned and are not hub gaps. P22 is W1.

Closed in earlier rounds (not re-listed): `probe project` collision rejection on authoritative inputs, the legacy-format projection marker, fail-closed checks on already-projected inputs, the legacy inherited-labels discriminating assertion, the P15 hub-side trim, projection-side record inertness with mixed valid/ghost seeds.

Totals (`cargo test --workspace`, this tree): probe 134 unit + 78 integration; probe-extract-check 30 unit + 27 golden (8 ignored); 269 passed, 0 failed.
