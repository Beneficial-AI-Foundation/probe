---
auditor: test-quality-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB; properties.md identical between this branch and main)
scope: branch la/merge-soundness-pr2-enrich-recomputation vs main (merge-soundness PR 2, issue #78), full property matrix
status: 0 critical, 3 warnings, 3 info
---

Test surface: inline `#[cfg(test)]` modules (`src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` — 97 unit tests), integration suites `tests/{merge,propagate,authority,schema_validation}.rs` (52 tests, binary-level via `CARGO_BIN_EXE_probe`), and the `probe-extract-check` crate (separate scope: extract-vs-source validation).

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures), authority.rs::project_output_uses_projected_atoms_schema_and_validates, ::aeneas_composed_envelope_inventory_survives_merge; tests/propagate.rs::test_envelope_structure_preserved | Full | `probe/summary` has no JSON schema by design (ADR-006/plan §6); its wire field names are pinned by summary::tests::test_summary_result_wire_field_names (new) |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural; determinism of code-names is producer-side |
| P3 | merge.rs::test_is_stub, project.rs::test_stub_seeds_included | Full | All three structural conditions pinned |
| P4 | — | None* | Law tests (associativity across groupings, commutativity, mapping compatibility) and μ's re-enrichment are PR 3b (plan §9) |
| P5 | — | None* | Identity-on-carrier test scheduled in PR 3b |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added, tests/merge.rs::test_atoms_* | Full | Unit + binary paths |
| P7 | merge.rs::test_generic_last_wins_on_conflict, tests/merge.rs::test_specs_*/test_proofs_* | Full | |
| P8 | merge.rs::test_trailing_dot_normalization, ::test_generic_trailing_dot_normalization; propagate.rs::test_prepare_normalizes_before_enrichment, ::test_prepare_normalizes_keys; tests/propagate.rs::test_dotted_alias_contamination_end_to_end | Partial | Keys + dependency arrays now pinned at the enrich boundary (new); `dependencies-with-locations` normalization still unpinned; extended arrays + project boundary are PR 3c |
| P9 | merge.rs::test_recursive_merge_flattens_provenance, ::test_generic_recursive_merge_flattens_provenance, types.rs::parse_envelope_detects_composed_shape_structurally, ::parse_envelope_rejects_malformed_inputs, ::parse_envelope_rejects_ambiguous_and_empty_provenance, authority.rs::aeneas_composed_envelope_inventory_survives_merge (test m) | Full (current scope) | Dedup clause is PR 3c and unimplemented |
| P10 | merge.rs::test_extensions_preserved, project.rs::test_extensions_preserved_through_projection; enrichment writes only `verification-status` (pinned indirectly by every status-origin test — the marker survives recomputation) | Full | |
| P13 | merge.rs::test_mappings_add_cross_language_edges, ::test_mapping_target_absent_no_edge_added, … | None* | Existing tests pin the **superseded** edge-injection semantics; record attachment + replacement tests are PR 3b |
| P14 | propagate.rs::test_deterministic_output, project.rs::test_determinism; summary lists inherit BTreeMap order | Partial | Still struct-level, not full-JSON comparison (W2, carried over); test_deterministic_output does compare serialized maps |
| P15 | probe-extract-check golden/properties tests (extract side) | Partial | Hub-side projection trim of categorized arrays is PR 3c and untested |
| P16 | propagate.rs recomputation suite (see P23 row) | Full (hub scope) | The hub enrichment note (recomputation, blocker seeds) is now implemented and pinned; producer mapping rows are producer-owned |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader, tests/merge.rs::test_category_mismatch_rejected | Full | |
| P19 | — | Indirect | No path deps (Cargo.toml unchanged); manifest hygiene, review-enforced |
| P21 | — | None (cross-repo) | Implemented and tested in probe-verus/probe-rust (ADR-005 ownership) |
| P22 | — | None (consumer script) | `scripts/summarize_extract.py` has no tests (W3, carried over) |
| P23 (authority boundary) | tests/authority.rs: tests f, l, n; src/authority.rs unit matrix | Full | Unchanged from PR 3a audit |
| P23 (recomputation/seeds) | propagate.rs: test_contaminated_transitively_verified_is_downgraded (a), test_contamination_flows_through_missing_status (b), test_trusted_boundary_blocks_contamination incl. spec-position variant (c), test_maps_to_records_do_not_contaminate (d), test_translation_origin_never_promoted (g), test_translation_origin_blocks_caller_promotion (h), test_translation_origin_transitively_verified_rewritten (i), test_copied_trusted_is_seed_not_boundary (j), test_kernel_taint_replicas (k), test_marked_status_less_atom_is_seed (quantifier edge), plus the retained chain/diamond/cycle/idempotence/missing-status suite; end-to-end: tests/propagate.rs::{test_stale_transitive_label_is_downgraded, test_status_origin_blocks_promotion_end_to_end, test_dotted_alias_contamination_end_to_end} | Full | Library + CLI paths; negative halves covered (plain trusted boundary, missing-status non-seed, non-candidates untouched, seeds keep base status). "Merge re-enriches" clause is PR 3b (test e in plan §3) |
| P24 | — | None (producer-side) | Validated in producer repos + probe-extract-check goldens |
| P25 | — | None (producer-side) | Same ownership as P24 |
| P27 | propagate.rs::test_maps_to_records_do_not_contaminate (inertness to enrichment) | Partial* | Union-through-conflicts and projection inertness land with the records in PR 3b/3c |

\* = scheduled in the merge-soundness plan §9 (PRs 3b/3c); the KB spec landed ahead of the code in PR 1 (#63) by design. Not regressions of this changeset.

ADR-006 Decision 9 (summary consumer contract, no property number): summary.rs::test_status_origin_matrix (2 origins × 5 statuses), ::test_translation_origin_excluded_from_lemmas, ::test_blueprint_atoms_excluded_from_partitions (plan test o), ::test_blueprint_deps_do_not_alter_entrypoints, ::test_summary_result_wire_field_names — Full: membership conjunction, no-list negative half, kernel-taint locality, blueprint exclusion from both partitions and entrypoint classification, and the wire field names.

## Impact analysis (this changeset)

Every behavior change landed with tests:

- Enrichment recomputation → the full plan §3 letter suite (a–d, g–k) at unit level plus three binary-level regressions; the pre-existing upgrade-only-pinning tests were re-validated against the new semantics (all still pass because their expectations were already recomputation-consistent).
- Trusted-boundary precedence → test j (both directions) + test h (marked seed behind a boundary).
- Carrier preparation at cmd_enrich → two prepare unit tests + the dotted-alias CLI regression.
- Summary contract → status matrix + blueprint pair + wire-contract test (added during this audit).
- status-origin schema enum → positive fixture (pre-existing) + new negative fixture (out-of-enum value rejected).
- Seed-quantifier edge (marked, status-less atom) → test added during this audit.

## Critical

None. (P4/P5/P13 and P27's union half have no coverage, but their implementations are staged in PRs 3b/3c per plan §9; tracked in the matrix with \*.)

## Warnings

### [W1] P13 tests pin superseded semantics (carried over)
- **Location**: src/commands/merge.rs tests (test_mappings_add_cross_language_edges and the P13-guard tests)
- **Issue**: these pin existence-checked edge injection, which revised P13 forbids. Replaced wholesale in PR 3b.

### [W2] P14 tested at struct level only (carried over)
- **Issue**: determinism tests compare serialized maps of atoms, not full envelope JSON.
- **Recommendation**: compare full serialized output (excluding timestamp) once PR 3b touches serialization anyway.

### [W3] `scripts/summarize_extract.py` untested (P22, carried over)
- **Recommendation**: low priority; smoke test over a fixture extract.

## Info

### [I1] Property-based testing opportunity (carried over)
P4/P5 law tests in PR 3b are natural proptest candidates; the BFS invariant ("label is a pure function of graph + base statuses") likewise.

### [I2] Gate thresholds asserted in one place only (carried over from PR 3a)
Acceptable; unchanged this branch.

### [I3] Mixed valid+ghost mapping seeds untested (carried over)
`test_missing_seeds_skipped` covers all-ghost; no test mixes valid and invalid mapping keys. Projection-side; natural home is PR 3c's seed-matching work.

## Post-audit delta (2026-09-29, review commit e6a376e)

New coverage from the PR #79 cross-model review, all landed with the
behavior it pins:

- **P23 presence-based predicates** — propagate.rs unit tests
  `test_non_string_status_origin_is_seed` (a `null` marker on a verified
  leaf blocks its caller) and
  `test_non_string_status_origin_disables_trusted_boundary` (a trusted
  atom bearing a non-string marker no longer shields callers). Library
  path; the CLI path can no longer reach the BFS with such input because
  of the boundary rejection below, which is itself pinned.
- **status-origin enum enforcement** — types.rs unit matrix
  (`validate_status_origins_enforces_the_two_value_enum`: both values and
  the absent marker pass; unknown string, empty string, `null`, number,
  object reject with the atom named) plus two binary-level rejections:
  tests/propagate.rs `test_enrich_rejects_invalid_status_origin`
  (non-string) and tests/authority.rs
  `summary_rejects_out_of_enum_status_origin` (unknown string) — both
  assert no output file is written. Both malformed classes × both
  boundaries covered across the four tests.
- **P8 collision surfacing** — merge.rs
  `test_normalization_collision_reporting`: distinct-real collision
  reported as `(discarded key, surviving key)`, stub and
  identical-duplicate collisions silent, and the `conflicts` count
  asserted through `merge_atom_maps` (probe-merge.md Phase 2); propagate.rs
  `test_prepare_reports_collision` (PrepareStats surface); tests/propagate.rs
  `test_enrich_rejects_normalization_collision` (CLI rejection names both
  keys, writes no output). Remaining gap, info-level: the merge-side
  stderr warning text is unpinned (behavior is pinned via the stats
  count); merge's first-wins output on collision is implied by the unit
  merged-map assertions.
- Updated totals: 102 unit tests, 55 binary-level integration tests
  (tests/{merge,propagate,authority,schema_validation}.rs).

## Verification pass (2026-09-29, targeted codex re-review of the fix diff)

Codex's test-sufficiency gaps, all closed:

- Library-boundary rejection: summary.rs
  `test_summarize_atoms_rejects_invalid_status_origin` (unknown string
  and non-string, direct `summarize_atoms` call).
- P8 fixed point: merge.rs `test_repeated_trailing_dots_normalize_in_one_pass`
  (`g()..` collides with `g()` in one pass, reported as dropped).
- Conflict counting in subsequent inputs and the benign zero-count:
  extended merge.rs `test_normalization_collision_reporting`.
- Rejection preserves pre-existing output: sentinel-content assertion in
  tests/propagate.rs `test_enrich_rejects_normalization_collision`.
- Benign collision acceptance end to end: tests/propagate.rs
  `test_enrich_accepts_identical_duplicate_and_stub_collisions`
  (identical duplicate collapses, stub alias absorbed).

Updated totals: 104 unit tests, 56 binary-level integration tests.
