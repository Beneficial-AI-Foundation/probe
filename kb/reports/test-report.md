---
auditor: test-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB). Property text audited against `origin/main:kb/engineering/properties.md` (4b633e0), per common skill §2b. The working tree is on la/merge-soundness-pr3c-collision-warnings-projection (PR #83), which amends P7 (intra-input tie-break is key order, not input order) and P8 (`probe project` rejects collisions on authoritative and projected inputs; categorized dependency arrays are sets; specs/proofs collision warning names input position, both keys and the kept key). Every other property heading and body is byte-identical to origin/main. Code that conforms only to those amendments is reported as a merge-order constraint (I4), not as clean.
scope: uncommitted working-tree changeset on PR #83 (review-comment fixes) — strict `normalize_atoms` with private non-rejecting core `normalize_atoms_reporting_collisions`; sort+dedup of the five categorized dependency arrays after normalization; `normalize_generic` warning text naming input position, both original keys and the kept key; `prepare_atoms` returns Err on collision and `PrepareStats::dropped_atoms` removed; `cmd_project` projected branch uses the strict helper; roundtrip tests updated. Full property matrix.
status: 0 critical, 2 warnings, 5 info
---

Test surface: inline `#[cfg(test)]` modules (`src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` — 134 unit tests), integration suites `tests/{authority,merge,merge_laws,propagate,roundtrip,schema_validation}.rs` (19 + 13 + 4 + 13 + 12 + 16 = 77 tests, binary-level via `CARGO_BIN_EXE_probe` plus jsonschema validation), and the `probe-extract-check` crate (separate scope: 30 unit + 27 golden, 8 ignored). `cargo test --workspace` on this tree: 268 passed, 0 failed, 8 ignored. The unit count rose 133 → 134 with `test_categorized_arrays_compared_as_sets`; `test_prepare_reports_collision` was renamed to `test_prepare_rejects_collision`.

No mutation checks this run: the audit was scoped read-only on source and tests. Discrimination claims below are argued from the code and the assertions, and say so where they matter.

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures); authority.rs::project_output_uses_projected_atoms_schema_and_validates; tests/propagate.rs::test_envelope_structure_preserved; roundtrip.rs::specs_merge_same_source_dedups_and_validates | Full | Unchanged by this changeset |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural; code-name determinism is producer-side |
| P3 | merge.rs::test_is_stub; project.rs::test_stub_seeds_included | Full | |
| P4 | merge.rs::test_associativity_with_mappings_across_groupings, ::test_commutativity_disjoint_keys, ::test_mapping_compatibility_laws, ::test_intermediate_enrichment_does_not_change_selected_base_data, ::test_tied_identity_records_keep_associativity; tests/merge_laws.rs::laws_hold_over_generated_record_variants (generated); roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup | Full (example + generated + binary) | The new categorized-array sort+dedup is idempotent canonicalization inside per-input normalization, so it cannot change which atom a later conflict selects. The generated sweep does not generate categorized arrays (tests/ has no `*-dependencies` fixture), so this is argued, not exercised (I1) |
| P5 | merge.rs::test_identity_exact_on_carrier, ::test_identity_up_to_preparation_on_legacy | Full | |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added; tests/merge.rs::test_atoms_* | Full | |
| P7 | merge.rs::test_generic_last_wins_on_conflict; tests/merge.rs::test_specs_*/test_proofs_*; merge.rs::test_generic_intra_input_collision_counted and roundtrip.rs::specs_intra_input_collision_warned_and_counted (intra-input tie-break: the dotted alias `s().` is kept) | Full vs origin/main | origin/main P7 governs only cross-input order ("same code-name in multiple inputs"), which the carried tests pin. The intra-input "key order, not P7" sentence is an amendment (I4). Both intra-input tests assert `which == "dotted"`, so a switch to first-wins within an input would fail them |
| P8 | Carried suite (trailing-dot normalization, dependencies-with-locations, record targets, mapping endpoints, intra-input distinct-real rejection on merge first/subsequent/raw inputs with `input #N of M` prefixes, two-input dotted-alias evidence selection; tests/propagate.rs enrich-boundary trio). **Changed this changeset**: merge.rs::test_normalization_collision_classification (the strict `normalize_atoms` now returns Err with `distinct real atom` and the colliding key; the lossy map is checked only through the private core); propagate.rs::test_prepare_rejects_collision (library `prepare_atoms` errs, message names the `"g()." vs "g()"` pair; replaces the stats-field assertion); roundtrip.rs::project_rejects_distinct_real_normalization_collision and ::project_rejects_invalid_projected_input `collision` case (both now match the shared `distinct real atom` wording, for authoritative and projected inputs). **New**: merge.rs::test_categorized_arrays_compared_as_sets (duplicates-only and order-only alias pairs collapse benignly with canonical sorted output; a real set difference still rejects); roundtrip.rs::specs_intra_input_collision_warned_and_counted asserts the full warning string (`input #1 of 2: … 's()' and 's().' both normalize to 's()'; kept 's().' (key order, P8)`) | Full vs origin/main for merge and enrich rejection; Partial on the amended clauses | origin/main P8 names merge and enrich as rejecting boundaries. Project rejection on authoritative input follows from origin/main's "every recomputation boundary", since P23 makes project's enrich-before-trim a recomputation. Rejection on projected views, categorized arrays as sets, and the warning contents are amendment-only (I4). The warning's input position is pinned only for input #1 (W1). The set test exercises only `body-dependencies`; that is sound, because the sort runs in a loop over `CATEGORIZED_DEPENDENCY_ARRAYS` and so cannot omit one field (I5) |
| P9 | types.rs::dedup_provenance_collapses_identical_entries_only; merge.rs::test_recursive_merge_flattens_provenance, ::test_merge_atom_files_dedups_provenance; authority.rs::aeneas_composed_envelope_inventory_survives_merge; roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup, ::specs_merge_same_source_dedups_and_validates | Full | Unchanged by this changeset |
| P10 | merge.rs::test_extensions_preserved; record union tested under P27 | Full vs origin/main | Sort+dedup now rewrites the five categorized arrays, which are extensions. origin/main P8 already sanctions rewriting them (normalization), but not reordering or deduplicating them. That is sanctioned only by the P8 amendment's "sets" sentence, and is part of I4 |
| P13 | Carried attachment suite (records-not-edges, dangling target, re-application no-op, one-to-many, distinct confidence; schema wire test; CLI end-to-end) | Full | |
| P14 | merge.rs::test_full_envelope_serialization_deterministic, ::test_distinct_confidence_records_both_kept_sorted; propagate.rs::test_deterministic_output; project.rs::test_determinism; **new**: merge.rs::test_categorized_arrays_compared_as_sets asserts the canonical sorted form (`["g()", "h()"]` from `["h()", "g()"]`) | Full | The categorized arrays now satisfy P14's "array fields MUST be sorted" by construction at every normalizing boundary. The strings-before-non-strings rule is pinned by test_categorized_dependency_arrays_normalized: `arr[0] == "g()"` and `arr[1] == 42` |
| P15 | project.rs::test_categorized_arrays_trimmed_with_dependencies; merge.rs::test_categorized_dependency_arrays_normalized, **new** ::test_categorized_arrays_compared_as_sets; probe-extract-check golden/properties tests (extract side) | Full (hub mutation sites) | The new sort+dedup keeps each subset's set value, so the union equality is unaffected. As before, no test computes the union and asserts the equality, and no binary-level fixture carries categorized arrays (I1) |
| P16 | propagate.rs recomputation suite (see P23 rows) | Full (hub scope) | Producer mapping rows are producer-owned |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader; tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | — | Indirect | No path deps; manifest hygiene, review-enforced |
| P21 | — | None (cross-repo) | Implemented and tested in probe-verus/probe-rust (ADR-005 ownership); not a hub gap |
| P22 | — | None (consumer script) | `scripts/summarize_extract.py` has no tests (W2, carried) |
| P23 (authority boundary) | tests/authority.rs suite; src/authority.rs unit matrix; raw-path/status-origin rejections; roundtrip.rs::merged_output_passes_the_hubs_own_gate, ::projection_readable_by_consumers_rejected_by_recomputation, ::project_rejects_invalid_status_origin, ::project_rejects_invalid_projected_input `bogus_marker` case | Full | Unchanged by this changeset |
| P23 (recomputation/seeds) | propagate.rs plan-§3 letter suite, chain/diamond/cycle/idempotence, merge-re-enriches pair, raw-defers negative | Full | `prepare_atoms` now errs before enrichment on a collision. test_prepare_rejects_collision pins that a library caller never receives the enriched lossy graph |
| P23 (projection caveat: enrich-before-trim, inherited labels) | roundtrip.rs::projection_recomputes_enrichment_before_trimming — **extended**: re-projects its own depth-0 output and asserts `f()` stays `verified` while its blocker `bad()` is absent from the view. Recomputation over the view would promote `f()` (no reachable seed), so the assertion discriminates. Also ::projection_readable_by_consumers_rejected_by_recomputation, ::project_inherits_labels_of_legacy_format_projection; tests/authority.rs::project_reads_an_already_projected_input | Full | Label inheritance on projected inputs is now pinned in both formats by an assertion that fails under recomputation: the modern format by the extended test, the legacy format by the carried test |
| P24 | — | None (producer-side) | Validated in producer repos and probe-extract-check goldens; not a hub gap |
| P25 | — | None (producer-side) | Same ownership as P24 |
| P27 | Carried suite (union through every equal-key case, collision classification, target normalization, identity/canonical-form/fail-closed matrix, generated laws, enrichment and projection inertness); merge.rs::test_normalization_collision_classification `with_record` case (records-only difference is benign, records unioned) — now through the private core | Full | The benign-collapse half is tested on the non-rejecting core. The strict wrapper's benign path is exercised by test_categorized_arrays_compared_as_sets and tests/propagate.rs::test_enrich_accepts_identical_duplicate_and_stub_collisions |

C-series known-issue entries: none in the properties file (origin/main or branch).

ADR-006 Decision 9 (summary consumer contract): unchanged; summary.rs matrix suite green.

## Impact analysis (this changeset)

- **Strict `normalize_atoms`.** Merge's per-input call sites lost their inline `dropped` checks. The carried first/subsequent/raw rejection tests (including `input #2 of 2` at merge.rs:1660 and the tests/merge_laws.rs:237/303 position prefixes) still pass, which shows that `input_context` wrapping survived the move. The strict/core split is pinned by test_normalization_collision_classification.
- **`prepare_atoms` returns Err; `PrepareStats::dropped_atoms` removed.** Pinned by test_prepare_rejects_collision (library) and tests/propagate.rs::test_enrich_rejects_normalization_collision (CLI: non-zero exit, both keys named, pre-existing output untouched). The CLI test's needle is the key pair, not the removed `refusing to enrich` text, so the wording change needed no test edit. Removing a `pub` struct field is a library-API change; that is not a test concern, but the auditors for code quality and changelog should see it.
- **`cmd_project` projected branch uses the strict helper; the copied rejection block is removed.** Pinned by project_rejects_invalid_projected_input `collision` case, now matching `distinct real atom`. The authoritative branch is pinned by project_rejects_distinct_real_normalization_collision. With the copied block gone, a lossy map can only reach projection if the strict helper stops rejecting, which test_normalization_collision_classification catches.
- **Categorized arrays sorted+deduplicated.** Added test: test_categorized_arrays_compared_as_sets (positive and negative). It is unit-level only, and every merge/enrich/project output's wire shape now changes for atoms that carry these arrays (I1).
- **`normalize_generic` warning text and tie-break wording.** Added assertions: the CLI test pins the exact string for input #1, and both tests pin the kept value. The `i + 1` position for later inputs is not pinned (W1).
- **KB edits to P7/P8 and kb/tools/probe-merge.md.** These document the above. The probe-merge.md example warning (`input #1 of 2: normalization collision: 's()' and 's().' both normalize to 's()'; kept 's().' (key order, P8)`) matches the format string at merge.rs:642 and the roundtrip needle.

## Critical

None.

## Warnings

### [W1] Specs/proofs collision warning's input position pinned only for the first input
- **Location**: src/commands/merge.rs:674 (`normalize_generic(incoming, i + 1, total)`); tests/roundtrip.rs:598-665; src/commands/merge.rs:2338-2358
- **Issue**: The P8 amendment requires the warning to name the input position. The first input passes the literal `1` (merge.rs:666). Later inputs compute `i + 1` from a 0-based `enumerate`. The only test that checks the text puts the collision in input #1 of 2, and the unit test uses a single input without inspecting stderr. An off-by-one at line 674 (passing `i`) would label a collision in the second input as `input #1 of 2` and every test would still pass. The atoms side does not have this gap: `input #2 of 2` is asserted at merge.rs:1660 and tests/merge_laws.rs:303.
- **Evidence**: `rg 'input #\d' src tests` finds only `input #1 of 2` on the specs/proofs path (tests/roundtrip.rs:649).
- **Recommendation**: In specs_intra_input_collision_warned_and_counted, also put an aliased pair in the second (probe-lean) input, or add a third input. Then assert `input #2 of 2` (or `#3 of 3`) in stderr and `Conflicts: 2`. Impact is diagnostic-only; the kept value and count are already pinned.

### [W2] `scripts/summarize_extract.py` untested (P22, carried)
- **Location**: scripts/summarize_extract.py
- **Issue**: The P22 consumer-side normalization (`TRUST_LABELS`, `TOOL_CONFIG`) has no tests.
- **Recommendation**: Low priority: a smoke test over a fixture extract that covers one value per canonical category.

## Info

### [I1] Categorized arrays never reach a binary-level fixture; P15 union equality never asserted directly (carried, extended)
No file under tests/ contains a `*-dependencies` array (`rg 'body-dependencies|requires-dependencies|CATEGORIZED' tests` → no matches). So the new sort+dedup wire change, the P15 trim, and the decomposition equality are pinned only at the helper level, and the generated merge-law sweep never exercises them. A roundtrip fixture carrying an unsorted, duplicated, dotted `body-dependencies` through `probe merge` → `probe project`, asserting `dependencies == union(subsets)` and sorted output, would cover all three at once.

### [I2] Gate thresholds asserted in one place only (carried from PR 3a)
Acceptable; unchanged.

### [I3] Defensive non-array guards in `push_record`/`union_correspondence_records` unreachable on merge paths (carried)
Unchanged; boundary validation guarantees arrays before any union or attachment.

### [I4] Merge-order constraint: P7/P8 amendments and the code ship together in PR #83
The following behaviours conform only to the PR #83 amendment text, not to `origin/main`: rejecting a distinct-real collision on an already-projected `probe project` input; sorting and deduplicating the categorized arrays (which also rewrites P10-preserved extensions beyond plain normalization); the specs/proofs warning contents; and the key-order intra-input tie-break wording. None of these contradicts origin/main text. The KB edits in kb/engineering/properties.md and kb/tools/probe-merge.md must not be split from the src/ changes, so they land together in PR #83.

### [I5] Set canonicalization and tie-break each tested on one instance; property-based opportunity
test_categorized_arrays_compared_as_sets uses one field (`body-dependencies`). The KB's three-alias tie-break example (`f()`, `f().`, `f()..` → most-dotted kept, 2 conflicts counted) has no test. "Aliases differing only in order/duplicates collapse, any set difference rejects" is a natural generated-input property: permute and duplicate a random set across aliases, and compare the collapse/reject outcome with set equality.

## Verified clean

P1, P3, P4, P5, P6, P9, P13, P14, P16 (hub scope), P17, P23 (all three facets) and P27 were checked and found covered against the origin/main text. The changeset left the carried coverage intact, and where it touched those properties the tests were updated alongside. P7, P8 and P10 are covered against origin/main, with the amendment-only parts flagged in W1, I4 and I5. P2 and P19 are Indirect by construction or review. P21, P24 and P25 are producer- or cross-repo-owned and are not hub gaps. P22 is W2.

Closed in earlier rounds (not re-listed): `probe project` collision rejection on authoritative inputs, the legacy-format projection marker, fail-closed checks on already-projected inputs, the legacy inherited-labels discriminating assertion, the P15 hub-side trim, and projection-side record inertness with mixed valid/ghost seeds.

Totals (`cargo test --workspace`, this tree): probe 134 unit + 77 integration; probe-extract-check 30 unit + 27 golden (8 ignored); 268 passed, 0 failed.
