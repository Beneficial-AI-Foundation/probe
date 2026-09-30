---
auditor: test-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB; this branch amends P8 — the `probe project` rejection clause and the specs/proofs counted-collision paragraph are unmerged edits, and the coverage rows for P8 judge the tests against the branch text, which is the semantics this PR implements; audit against `main:kb/engineering/properties.md` confirmed no other property text differs — merge-order constraint: PR #82 must land with these KB edits)
scope: branch la/merge-soundness-pr3c-collision-warnings-projection vs origin/main (merge-soundness PR 3c, issue #82 — counted collision warnings, provenance dedup + envelope idempotence, minItems relaxation, P8/P15 categorized-array completion, projection prepare = enrich ∘ normalize, 0.5.0 contract release, round-trip suite), full property matrix; third run, over the uncommitted delta: the I4 fix to the legacy inherited-labels test, plus two new `cmd_project` behaviors in src/commands/project.rs (status-origin validation on every input, and normalization without enrichment for already-projected inputs) with their tests in tests/roundtrip.rs; fourth run, over `project_rejects_invalid_projected_input` (targets W4) and the `write_real_projection` helper refactor
status: 0 critical, 1 warning, 3 info
---

Test surface: inline `#[cfg(test)]` modules (`src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` — 133 unit tests), integration suites `tests/{merge,merge_laws,propagate,authority,schema_validation,roundtrip}.rs` (77 tests, binary-level via `CARGO_BIN_EXE_probe` plus jsonschema validation; `roundtrip.rs` is new this branch, 12 tests over real binary outputs), and the `probe-extract-check` crate (separate scope: 30 unit + 27 golden, 8 ignored). `cargo test --workspace`: 267 passed, 0 failed, 8 ignored.

Fourth run. I re-applied the two mutations that left the suite green in the third run. Running the status-origin check only when `!loaded.projected` now fails `project_rejects_invalid_projected_input` in its `bogus_marker` case. Discarding `dropped` in the projected branch fails the same test in its `collision` case (tests/roundtrip.rs:414 in both). No other test fails. Source restored: src/commands/project.rs is byte-identical to the pre-mutation copy, and the `git diff src` hash is unchanged. The `write_real_projection` helper (tests/roundtrip.rs:279-307) runs the same merge, the same `f()`→`probe:Lean.T` mapping and the same project call that `project_inherits_labels_of_legacy_format_projection` and `project_normalizes_keys_of_projected_input` previously did inline. Their assertions are unchanged, so the refactor does not weaken them.

Mutation checks, third run. Each mutation was applied to the working tree (which already contains the intended project.rs change) and then reverted. Both source files were compared byte for byte against copies saved before the first mutation, and `git diff src` shows only the intended src/commands/project.rs change.

| Mutation | Result |
|---|---|
| Drop `\|\| meta.has_projection_field` (src/authority.rs:211) | Only `project_inherits_labels_of_legacy_format_projection` fails, at its `labels inherited` stderr assertion (tests/roundtrip.rs:330). That assertion runs first, so this mutation never reaches the label check |
| Keep the `labels inherited` message but run `prepare_atoms` (enrichment) in the projected branch (src/commands/project.rs:253) | Only the legacy test fails, at the label `assert_eq!` (tests/roundtrip.rs:335). The label assertion now discriminates on its own, which closes I4 |
| Validate status-origins over an empty map instead of the input (src/commands/project.rs:238) | Only `project_rejects_invalid_status_origin` fails (non-zero-exit assertion, tests/roundtrip.rs:433) |
| Skip normalization for projected inputs (return `loaded.atoms` unchanged, src/commands/project.rs:253) | Only `project_normalizes_keys_of_projected_input` fails (tests/roundtrip.rs:394) |
| Run the status-origin check only when `!loaded.projected` | All tests passed in the third run (W4). In the fourth run, `project_rejects_invalid_projected_input` fails (`bogus_marker` case) |
| Discard `dropped` in the projected branch, so a view's collisions are not rejected | All tests passed in the third run (W4). In the fourth run, `project_rejects_invalid_projected_input` fails (`collision` case) |

The previous runs' mutations (disabling the project collision rejection; dropping the disjunct) remain closed. The collision test still fails when its rejection block is disabled, because that block is now shared by both branches.

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures), authority.rs::project_output_uses_projected_atoms_schema_and_validates, tests/propagate.rs::test_envelope_structure_preserved; **new**: roundtrip.rs::specs_merge_same_source_dedups_and_validates (real merged envelope with a deduped single-entry inventory validates against the executable schema — pins the minItems 2→1 relaxation) | Full | |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural; code-name determinism is producer-side |
| P3 | merge.rs::test_is_stub, project.rs::test_stub_seeds_included | Full | |
| P4 | merge.rs::test_associativity_with_mappings_across_groupings, ::test_commutativity_disjoint_keys, ::test_mapping_compatibility_laws, ::test_intermediate_enrichment_does_not_change_selected_base_data, ::test_tied_identity_records_keep_associativity; tests/merge_laws.rs::laws_hold_over_generated_record_variants (generated sweep); **new**: roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup (envelope-level idempotence at the binary boundary: re-merging merged output with one of its inputs changes neither `data` nor the deduped inventory — the "modulo envelope meta / deduplicated inventory" qualifier of the laws, testable only after P9 dedup) | Full (example + generated + binary) | |
| P5 | merge.rs::test_identity_exact_on_carrier, ::test_identity_up_to_preparation_on_legacy | Full | |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added, tests/merge.rs::test_atoms_* | Full | |
| P7 | merge.rs::test_generic_last_wins_on_conflict, tests/merge.rs::test_specs_*/test_proofs_*; **new**: last-wins explicitly re-asserted under an intra-input collision (merge.rs::test_generic_intra_input_collision_counted, roundtrip.rs::specs_intra_input_collision_warned_and_counted) | Full | |
| P8 | Carried suite (trailing-dot normalization, collision classification, intra-input distinct-real rejection on merge first/subsequent/raw inputs, record targets, dependencies-with-locations, mapping endpoints, two-input dotted-alias evidence selection; propagate prepare trio; tests/propagate.rs enrich-boundary trio). **New this branch**: merge.rs::test_categorized_dependency_arrays_normalized (the five CATEGORIZED_DEPENDENCY_ARRAYS normalized like `dependencies`, non-string entries untouched — shared `normalize_atoms` helper, so the merge, enrich, and project boundaries all inherit it); merge.rs::test_generic_intra_input_collision_counted + roundtrip.rs::specs_intra_input_collision_warned_and_counted (specs/proofs collision warned + counted in `stats.conflicts`, unit and CLI, stdout `Conflicts: 1` asserted); roundtrip.rs::projection_seeds_match_normalized_keys (project boundary normalizes before seed matching — dotted-key atom found by its normalized mapping seed); roundtrip.rs::project_rejects_distinct_real_normalization_collision (a `probe-verus/atoms` extract with distinct real atoms `g()`/`g().` → `probe project` exits non-zero, stderr says `refusing to project` and names both keys, no output file written); roundtrip.rs::project_normalizes_keys_of_projected_input (a real projection with `f()` re-keyed to `f().` is re-projected; `f()` is selected by its normalized seed, so projected inputs are normalized too); roundtrip.rs::project_rejects_invalid_projected_input `collision` case (a real projection with a distinct real `f().` alias beside `f()` → non-zero exit, `refusing to project`, no output) | Full | Collision rejection is pinned at merge, enrich, and project, and at project for both authoritative and already-projected inputs. This covers the kb/tools/probe-project.md Step 2 clause "with the same collision rejection" (W4 closed, mutation-checked) |
| P9 | Carried flattening suite (test_recursive_merge_flattens_provenance, generic variant, types.rs parse matrix, authority.rs::aeneas_composed_envelope_inventory_survives_merge). **New — dedup clause now implemented and tested at all three levels**: types.rs::dedup_provenance_collapses_identical_entries_only (exact duplicates collapse first-occurrence-ordered; entries differing only in flattened `source` extensions stay distinct), merge.rs::test_merge_atom_files_dedups_provenance (library file path), roundtrip.rs::envelope_idempotence_modulo_meta_after_dedup + ::specs_merge_same_source_dedups_and_validates (cmd_merge binary path, both categories) | Full | Closes the previous report's "dedup clause is PR 3c and unimplemented" caveat |
| P10 | merge.rs::test_extensions_preserved; record union tested under P27 | Full | |
| P13 | Carried attachment suite (records-not-edges, dangling target, re-application no-op, one-to-many, distinct-confidence; schema wire test; CLI end-to-end) | Full | |
| P14 | merge.rs::test_full_envelope_serialization_deterministic, ::test_distinct_confidence_records_both_kept_sorted; propagate.rs::test_deterministic_output, project.rs::test_determinism | Full | dedup_provenance keeps first-occurrence order — deterministic given the argument order, which P6/P7 already make significant |
| P15 | **New**: project.rs::test_categorized_arrays_trimmed_with_dependencies (projection trims the categorized arrays with the same filter as `dependencies`; excluded name removed, included kept, non-string entry untouched — would fail if project trimmed `dependencies` alone, the exact break the plan's §6 P15 item names); merge.rs::test_categorized_dependency_arrays_normalized (normalization applies one rule to `dependencies` and the subsets, so the decomposition equality survives merge/enrich/project preparation); probe-extract-check golden/properties tests (extract side) | Full (hub mutation sites) | Both hub-side transformations that touch these arrays are pinned; closes the carried W1. Residual imprecision — no test asserts the union equality itself on a real merged/projected binary output — is I1 |
| P16 | propagate.rs recomputation suite (see P23 row) | Full (hub scope) | Producer mapping rows are producer-owned |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader, tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | — | Indirect | No path deps; manifest hygiene, review-enforced |
| P21 | — | None (cross-repo) | Implemented and tested in probe-verus/probe-rust (ADR-005 ownership) |
| P22 | — | None (consumer script) | `scripts/summarize_extract.py` has no tests (W3, carried) |
| P23 (authority boundary) | Carried tests/authority.rs suite + src/authority.rs unit matrix + raw-path/status-origin rejections. **New — real-binary round trips (the PR 3a review gap)**: roundtrip.rs::merged_output_passes_the_hubs_own_gate (merge → enrich/summary/project on actual output; `tool.version` pinned to `CARGO_PKG_VERSION`, i.e. the 0.5.0 contract release inside the gate interval), ::projection_readable_by_consumers_rejected_by_recomputation (real projection: summary and project read it; merge and enrich reject it with `projected input` in stderr and the test asserts the absence of `pre-contract`, attributing the rejection to the projection predicate, not the version gate); roundtrip.rs::project_rejects_invalid_status_origin (an authoritative extract with `status-origin: "bogus"` → `probe project` exits non-zero with `invalid status-origin` and writes no output); roundtrip.rs::project_rejects_invalid_projected_input `bogus_marker` case (the same check on a real projection) | Full | The `probe project` status-origin check is pinned for authoritative and projected inputs, matching kb/engineering/schema.md (`status-origin` row) and kb/tools/probe-project.md ("every input, projected or not"). W4 closed, mutation-checked |
| P23 (recomputation/seeds) | Carried plan-§3 letter suite + chain/diamond/cycle/idempotence + merge-re-enriches pair + raw-defers negative | Full | |
| P23 (projection caveat: enrich-before-trim, inherited labels) | **New**: roundtrip.rs::projection_recomputes_enrichment_before_trimming (the Verus-shaped stale-label regression: stale embedded `transitively-verified` over a `failed`-reaching chain comes out `verified` in a depth-0 view whose trimmed `dependencies` could never justify the recomputation — pins recompute-on-the-full-graph-before-trim); ::projection_readable_by_consumers_rejected_by_recomputation asserts `labels inherited` stderr when projecting a projection (marker preserved through the ReadOnly load, modern format); tests/authority.rs::project_reads_an_already_projected_input; roundtrip.rs::project_inherits_labels_of_legacy_format_projection (a real projection rewritten to the legacy shape, with schema `probe/merged-atoms` and the `projection` field kept, is re-projected; stderr must say `labels inherited` and an injected `verified` label, which recomputation would promote to `transitively-verified`, must be kept) | Full | Both disjuncts of `ValidatedAtomFile.projected` are pinned at the project boundary (the first run's W2). The label assertion now discriminates independently of the stderr line (second run's I4, mutation-checked) |
| P24 | — | None (producer-side) | Validated in producer repos + probe-extract-check goldens |
| P25 | — | None (producer-side) | Same ownership as P24 |
| P27 | Carried suite (union through every equal-key case, collision classification, target normalization, identity/canonical-form/fail-closed matrix, generated laws, enrichment inertness). **New**: project.rs::test_selection_is_dependency_only (projection BFS never traverses `maps-to`/`mapped-from`: an atom reachable only via a record is excluded, with a mixed present/absent mapping-endpoint pair — closes the carried I3, both halves: record inertness and mixed valid+ghost seeds, `seeds_found` asserted) | Full | |

ADR-006 Decision 9 (summary consumer contract): unchanged; summary.rs matrix suite still green (the `ValidatedAtomFile` refactor in cmd_summary is mechanical destructuring).

## Plan §9 row 3c obligations

| Obligation | Covered by | Verdict |
|---|---|---|
| Counted collision warnings feeding `stats.conflicts` | test_generic_intra_input_collision_counted (unit), specs_intra_input_collision_warned_and_counted (CLI: stderr warning + stdout count + last-wins) | Yes |
| Provenance dedup | dedup_provenance unit test, test_merge_atom_files_dedups_provenance, both roundtrip dedup tests | Yes |
| `minItems` relaxation validated | specs_merge_same_source_dedups_and_validates (single-entry inventory vs executable schema; only the generic merged branch changed 2→1 — verified against `main:schemas/atom-envelope.schema.json`) | Yes |
| Envelope idempotence modulo meta, after dedup | envelope_idempotence_modulo_meta_after_dedup | Yes |
| P8 extension (categorized arrays) | test_categorized_dependency_arrays_normalized | Yes |
| P15 trim + decomposition test | test_categorized_arrays_trimmed_with_dependencies (projected output), test_categorized_dependency_arrays_normalized (merge path) | Yes, unit-level (I1: no explicit union-equality assertion on binary output) |
| Dependency-only projection-selection regression (§8) | test_selection_is_dependency_only | Yes |
| Verus-shaped stale-label regression | projection_recomputes_enrichment_before_trimming | Yes |
| Dotted-key projection-seed regression | projection_seeds_match_normalized_keys | Yes |
| Projection-marker preservation through ReadOnly load | projection_readable_by_consumers_rejected_by_recomputation (modern format), project_inherits_labels_of_legacy_format_projection (legacy format); both assert `labels inherited` | Yes (the legacy test also asserts the inherited label value) |
| Round trips: merge → enrich/summary/project | merged_output_passes_the_hubs_own_gate | Yes |
| Round trips: project → summary/project | projection_readable_by_consumers_rejected_by_recomputation | Yes |
| Round trips: project → merge/enrich fails on the projection predicate specifically | same test — asserts `projected input` present and `pre-contract` absent | Yes |

Not in the row but introduced by this branch's KB edit: `probe project` collision rejection is covered by project_rejects_distinct_real_normalization_collision, which closes the previous W1.

## Impact analysis (this changeset)

- Collision counting in `normalize_generic` → unit + CLI tests, both asserting the count reaches `stats.conflicts` and last-wins is kept.
- `dedup_provenance` wired into `load_atom_inputs` and `cmd_merge` → three levels (struct, library file path, binary both categories), plus the idempotence and schema-validation consequences.
- `CATEGORIZED_DEPENDENCY_ARRAYS` normalization (merge.rs) and trim (project.rs) → one test at each mutation site; the shared-helper placement means enrich/project inherit the normalization coverage.
- `ValidatedAtomFile` (`projected` flag replacing the tuple) → compile-enforced across callers; the flag's *use* (skip preparation, inherit labels) is pinned for both formats (the legacy one by project_inherits_labels_of_legacy_format_projection).
- Projection carrier preparation (`prepare = enrich ∘ normalize` before seed matching/trimming) → the Verus-shaped and dotted-key roundtrip regressions; the collision-rejection arm is pinned by project_rejects_distinct_real_normalization_collision.
- `cmd_project` status-origin validation (uncommitted, src/commands/project.rs:238) → project_rejects_invalid_status_origin (authoritative input) and project_rejects_invalid_projected_input (projected input).
- `cmd_project` normalizes already-projected inputs without enrichment (uncommitted, src/commands/project.rs:251-255) → project_normalizes_keys_of_projected_input pins the normalization, and project_rejects_invalid_projected_input pins the collision rejection on projected inputs.
- Uncommitted KB and schema edits (a glossary entry for `carrier preparation`, kb/tools/probe-merge.md, kb/tools/probe-project.md Step 2, kb/engineering/schema.md adding `probe project` to the status-origin rejecters, and the schema `inputs` description) document the two behaviors above. The schema change is a description string only.
- 0.5.0 bump (ADR-006 Decision 7) → merged_output_passes_the_hubs_own_gate pins `tool.version == CARGO_PKG_VERSION` and its acceptance by the hub's own gate — the test that PR 3a's hypothetical-input acceptance tests deferred to the contract release.

## Critical

None.

## Warnings

### [W1] ~~`probe project` distinct-real collision rejection untested (new P8 clause)~~ — resolved
- **Resolved by**: tests/roundtrip.rs::project_rejects_distinct_real_normalization_collision. It asserts a non-zero exit, `refusing to project` in stderr, both colliding keys named, and no output file written.
- **Mutation check**: replacing the condition at src/commands/project.rs:258 with `false` fails this test and no other. Source restored.

### [W2] ~~Legacy-format projection marker unpinned at the project boundary~~ — resolved
- **Resolved by**: tests/roundtrip.rs::project_inherits_labels_of_legacy_format_projection. It takes a real projection, rewrites `schema` to `probe/merged-atoms` while keeping the `projection` field, runs `probe project` on it, and asserts `labels inherited` in stderr.
- **Mutation check**: dropping `|| meta.has_projection_field` at src/authority.rs:211 fails this test at its `labels inherited` assertion (tests/roundtrip.rs:330), and no other test. Source restored.
- **Residual**: the test's label-value assertion does not discriminate (I4).

### [W3] `scripts/summarize_extract.py` untested (P22, carried)
- **Location**: scripts/summarize_extract.py
- **Recommendation**: low priority; smoke test over a fixture extract.

### [W4] ~~`probe project`'s fail-closed checks are untested on already-projected inputs~~ — resolved
- **Resolved by**: tests/roundtrip.rs::project_rejects_invalid_projected_input. It builds a real projection with `write_real_projection`, then runs two cases: `bogus_marker` sets `status-origin: "bogus"` on `f()`, and `collision` adds a distinct real `f().` alias with status `failed`. Each case asserts a non-zero exit, the expected message (`invalid status-origin` / `refusing to project`), and no output file.
- **Mutation check**: running the status-origin check only for authoritative inputs fails the `bogus_marker` case. Discarding the projected branch's `dropped` list fails the `collision` case. No other test fails in either case. Both were reverted.

## Info

### [I1] P15 decomposition asserted per-field, not as the union equality, and not on binary output
test_categorized_arrays_trimmed_with_dependencies and test_categorized_dependency_arrays_normalized assert exact field values whose instances happen to satisfy `dependencies = union(subsets)`; neither computes the union and asserts the equality, and no roundtrip fixture carries categorized arrays through `probe merge`/`probe project` at the binary level. The mutation sites are pinned (hence the matrix's Full), but an equality-form assertion on a roundtrip fixture would make the invariant explicit and catch a future third mutation site for free.

### [I2] Gate thresholds asserted in one place only (carried from PR 3a)
Acceptable; unchanged this branch.

### [I3] Defensive non-array guards in `push_record`/`union_correspondence_records` unreachable on merge paths (carried, was I4)
Unchanged this branch; boundary validation still guarantees arrays before any union or attachment. Noted so nobody mistakes the guards for a live path with missing coverage.

### [I4] ~~Legacy inherited-labels test: its label-value assertion cannot detect recomputation, and its comment is wrong~~ — resolved
- **Resolved by**: the test now injects `"verified"` and asserts it is kept (tests/roundtrip.rs:316, 335-339). The comment now correctly says recomputation would promote the label because `f()` reaches no seed.
- **Mutation check**: keeping the `labels inherited` message but recomputing enrichment in the projected branch fails the test at the label `assert_eq!` (tests/roundtrip.rs:335). Reverted.

Closed this round: W4 (fail-closed checks on already-projected inputs), mutation-checked.

Closed in earlier rounds: I4 (legacy test label assertion), from the third run; W1 (`probe project` collision rejection on authoritative inputs) and W2 (legacy-format projection marker), from the second run; the P15 hub-side trim warning (the trim landed with its test), and the projection-side record inertness plus mixed valid/ghost seeds info finding (test_selection_is_dependency_only covers both).

Totals (`cargo test --workspace`, this tree): probe 133 unit + 77 integration; probe-extract-check 30 unit + 27 golden (8 ignored); 267 passed, 0 failed.
