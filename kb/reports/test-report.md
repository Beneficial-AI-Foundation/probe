---
auditor: test-quality-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB; properties.md identical between this branch and main)
scope: branch la/merge-soundness-pr3a-authority-boundary vs main (merge-soundness PR 3a, issue #76), full property matrix
status: 0 critical, 3 warnings, 3 info
---

Test surface: inline `#[cfg(test)]` modules (`src/types.rs`, `src/authority.rs`, `src/commands/{merge,project,propagate,summary}.rs` — 79 unit tests), integration suites `tests/{merge,propagate,authority,schema_validation}.rs` (44 tests, binary-level via `CARGO_BIN_EXE_probe`), and the `probe-extract-check` crate (57 tests, separate scope: extract-vs-source validation).

## Coverage matrix

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | schema_validation.rs (all fixtures), authority.rs::project_output_uses_projected_atoms_schema_and_validates, ::aeneas_composed_envelope_inventory_survives_merge | Full | Merge and projected outputs are validated against the executable schema; `probe/summary` has no JSON schema by design (ADR-006/plan §6) |
| P2 | (BTreeMap keys by construction) | Indirect | Uniqueness is structural; determinism of code-names is producer-side |
| P3 | merge.rs::test_is_stub, project.rs::test_stub_seeds_included | Full | All three structural conditions pinned |
| P4 | — | None* | Law tests (associativity across groupings, commutativity, mapping compatibility) are scheduled in PR 3b (plan §9); μ's re-enrichment is not yet implemented |
| P5 | — | None* | Identity-on-carrier test scheduled in PR 3b |
| P6 | merge.rs::test_stub_replaced_by_real, ::test_real_vs_real_conflict_keeps_base, ::test_new_atoms_added, tests/merge.rs::test_atoms_* | Full | Unit + binary paths |
| P7 | merge.rs::test_generic_last_wins_on_conflict, tests/merge.rs::test_specs_*/test_proofs_* | Full | |
| P8 | merge.rs::test_trailing_dot_normalization, ::test_generic_trailing_dot_normalization | Partial | Keys covered; dependency-array and `dependencies-with-locations` normalization implemented but unpinned; extended arrays + unary boundaries are PR 2/3c |
| P9 | merge.rs::test_recursive_merge_flattens_provenance, ::test_generic_recursive_merge_flattens_provenance, types.rs::parse_envelope_detects_composed_shape_structurally, ::parse_envelope_rejects_malformed_inputs, authority.rs::aeneas_composed_envelope_inventory_survives_merge (test m) | Full (current scope) | Dedup clause is PR 3c and unimplemented; structural detection incl. negative case (malformed `inputs` errors) added this branch |
| P10 | merge.rs::test_extensions_preserved, project.rs::test_extensions_preserved_through_projection | Full | |
| P13 | merge.rs::test_mappings_add_cross_language_edges, ::test_mapping_target_absent_no_edge_added, … | None* | Existing tests pin the **superseded** edge-injection semantics (existence-checked); record attachment + these tests' replacement is PR 3b |
| P14 | propagate.rs::test_deterministic_output, project.rs::test_determinism, authority validator (no unordered iteration) | Partial | Still struct-level, not full-JSON comparison (W2, carried over) |
| P15 | probe-extract-check golden/properties tests (extract side) | Partial | Hub-side projection trim of categorized arrays is PR 3c and untested |
| P16 | propagate.rs status tests (verified→transitively-verified, failed/unverified untouched) | Partial | Producer mapping rows are producer-owned; recomputation semantics are PR 2 |
| P17 | merge.rs::test_category_detection, ::test_category_mismatch_detected_by_loader, tests/merge.rs::test_category_mismatch_rejected | Full | Loader + binary paths, incl. output-not-created negative |
| P19 | — | Indirect | No path deps exist (Cargo.toml unchanged); enforced by review, not test — acceptable for a manifest hygiene rule |
| P21 | — | None (cross-repo) | Implemented and tested in probe-verus/probe-rust (ADR-005 ownership); not testable from the hub |
| P22 | — | None (consumer script) | `scripts/summarize_extract.py` TRUST_LABELS has no tests; vocabulary is producer-owned |
| P23 (authority boundary) | tests/authority.rs: tests f (both projection formats × merge/enrich), l (5-producer gate matrix, legacy Verus-composer shape, post-threshold + ungated acceptance), n (summary/project gate; projections readable); src/authority.rs unit matrix (interval bounds, unparsable versions, malformed tool, `projection: null`, ReadOnly scope, non-atoms pass-through) | Full | New in this branch |
| P23 (recomputation/seeds) | propagate.rs contamination suite (chain, diamond, cycle, trusted boundary, missing-status, idempotence) | Partial* | Pins current upgrade-only behavior; recomputation, transparency chain, `status-origin` seed tests (a–e, g–k) are PR 2 |
| P24 | — | None (producer-side) | Verus/Aeneas scope semantics; validated in producer repos + probe-extract-check goldens |
| P25 | — | None (producer-side) | Same ownership as P24 |
| P27 | — | None* | Record union/inertness lands with the records themselves in PR 3b |

\* = scheduled in the merge-soundness plan §9 (PRs 2/3b/3c); the KB spec landed ahead of the code in PR 1 (#63) by design. Not regressions of this changeset.

## Impact analysis (this changeset)

Every behavior change in the branch landed with tests:

- Projection rejection → tests f (integration) + 4 unit tests incl. `projection: null` and ReadOnly readability.
- Version gate → test l (integration, all four rejection shapes + acceptance set) + unit matrix (thresholds, interval, command check, unparsable/malformed edge cases per the plan's §3d validator contract).
- Structural provenance → test m + two new types.rs unit tests (added during this audit: malformed-`inputs` error, structural composed detection).
- `probe/projected-atoms` output → integration test validating schema string, projection block, executable-schema validation, and rejection on re-feed.
- Read-only gate in summary/project → test n (+ already-projected input readable by project).
- Executable-schema rework → schema_validation.rs: composed Aeneas fixture (replacing the synthetic-`source` masking fixture, closing docs-report W0), vcvio + status-origin fixture, projected fixture incl. missing-projection-block negative, both-source-and-inputs negative.
- `cmd_enrich` schema-check reroute → covered by enrich rejection tests (unknown/non-atoms schema now errors); acceptance pinned by existing tests/propagate.rs suite on a probe-verus fixture.

## Critical

None. (P4/P5/P13/P27 have no coverage, but the properties' implementations are staged in PRs 2/3b/3c per plan §9; flagging them critical every run until then would be noise. They are tracked in the matrix with \*.)

## Warnings

### [W1] P13 tests pin superseded semantics
- **Location**: src/commands/merge.rs tests (test_mappings_add_cross_language_edges and the P13-guard tests)
- **Issue**: these pin existence-checked edge injection, which revised P13 forbids. They will be replaced wholesale in PR 3b; until then a reader may mistake them for conformance tests.
- **Recommendation**: PR 3b replaces them with record-attachment tests (already in its §9 row).

### [W2] P14 tested at struct level only (carried over)
- **Issue**: determinism tests compare BTreeMap key order, not serialized JSON.
- **Recommendation**: compare `serde_json::to_string_pretty` output (excluding timestamp) once PR 3b touches serialization anyway.

### [W3] `scripts/summarize_extract.py` untested (P22)
- **Issue**: the normalization mapping the property names has no test harness.
- **Recommendation**: low priority; consider a smoke test over a fixture extract.

## Info

### [I1] Property-based testing opportunity (carried over)
P4/P5 law tests in PR 3b are natural proptest candidates (generated maps rather than hand-picked triples); BFS invariants likewise.

### [I2] Gate thresholds asserted in one place only
The unit matrix hardcodes the ADR-006 numbers, so a threshold typo in `GATE_FLOORS` would be caught; the integration tests use near-threshold versions (0.15.x/0.20.x vs floors), which double-pins probe-lean/probe-aeneas but not leanblueprint (unit-only). Acceptable.

### [I3] Mixed valid+ghost mapping seeds untested (carried over)
`test_missing_seeds_skipped` covers all-ghost; no test mixes valid and invalid mapping keys.

## Post-audit delta (2026-09-29, review commit 9507b3c)

New coverage from the PR #77 cross-model review, all landed with the
behavior they pin:

- Ambiguous/empty provenance: types.rs
  `parse_envelope_rejects_ambiguous_and_empty_provenance` (unit) plus
  tests/authority.rs `merge_rejects_envelope_with_both_source_and_inputs`
  and `project_rejects_empty_inputs_inventory` (binary-level — the
  schema-validator fixtures alone did not exercise the production loader).
- The three projection-rejection integration tests now assert
  message-specific needles (`"projected input (schema"`,
  `"legacy projection rejected"`): the old substring needles also matched
  the fixture *filenames*, so any rejection of the file — including a pure
  version-gate one — satisfied them. Closes that attribution gap.
- Signed version components (`+0.16.0`, `0.+16.0`) added to the
  unparsable-version rejection matrix.
