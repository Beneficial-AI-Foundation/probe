---
auditor: test-quality-auditor
date: 2026-03-27
status: 0 critical, 5 warnings, 4 info
scope: is-public visibility feature (probe-rust + probe-aeneas)
---

## Critical

None. Core happy-path behavior for single-candidate matching and aeneas enrichment is unit-tested.

## Warnings

### [W1] Multi-candidate disambiguation not tested for `is_public`
- **Location**: `probe-rust/src/charon_names.rs` enrich_atoms_with_charon_names
- **Issue**: Three match paths exist (single candidate, span disambiguation, heuristic fallback). Only single candidate is tested for `is_public` propagation.
- **Recommendation**: Add test with two atoms sharing a match-key, different spans and visibilities. Assert span winner's `is_public`.

### [W2] P10 (`is-public` through merge) not specifically tested
- **Location**: `probe/src/commands/merge.rs` tests
- **Issue**: Generic `test_extensions_preserved` covers a different extension key. No test sets `is-public` on a Rust Atom, runs `merge_atom_maps`, and asserts survival (including stub→real replacement).
- **Recommendation**: Add merge test with `is-public` extension.

### [W3] Serialization roundtrip not tested
- **Location**: `probe-rust/src/lib.rs` AtomWithLines
- **Issue**: No test verifies `"is-public"` JSON key name, that `None` is omitted on serialize, and that deserialize roundtrips work.
- **Recommendation**: Add serde roundtrip test for AtomWithLines with and without `is_public`.

### [W4] Integration/fixture contract gap
- **Location**: `probe-rust/tests/extract_check.rs`, `probe-aeneas/tests/extract_check.rs`, `examples/`
- **Issue**: Integration tests do not mention `is-public`. Example JSON files have no `is-public` entries.
- **Recommendation**: Once examples are regenerated with Charon, assert Rust atoms include `is-public`.

### [W5] External stubs and Charon-off path not tested
- **Location**: `probe-rust/src/charon_names.rs`
- **Issue**: No test verifies external stubs (empty `code_path`) remain `is_public: None` after enrichment. No test for Charon enrichment skipped scenario.
- **Recommendation**: Add targeted tests for both paths.

## Info

### [I1] P11 (translation 1-to-1)
Not affected by `is-public`. No additional test requirement.

### [I2] P14 (deterministic output)
No test asserts stable `is-public` across runs. Determinism is plausible but not evidenced.

### [I3] probe-aeneas enrichment test models merged world correctly
`enrich_defaults_is_public_false_for_rust_atoms` correctly models Atom.extensions without `is-public`.

### [I4] Generic extension preservation provides weak P10 support
`test_extensions_preserved` in probe covers flattened extension fields generically, providing non-zero support for `is-public`.

## Coverage Table

| Area | Tests | Coverage | Notes |
|------|-------|----------|-------|
| `build_fun_span_map` reads visibility | `test_build_fun_span_map_extracts_visibility` | Full | public + private |
| `parse_llbc_names` carries visibility | `test_parse_llbc_names_carries_visibility` | Full | public + private |
| `enrich_atoms_with_charon_names` (1 candidate) | `test_enrich_propagates_visibility` | Partial | Multi-candidate not tested |
| probe-aeneas: default `false` | `enrich_defaults_is_public_false_for_rust_atoms` | Full | |
| probe-aeneas: preserve `true` | `enrich_preserves_existing_is_public_true` | Full | |
| probe-aeneas: Lean untouched | `enrich_does_not_add_is_public_to_lean_atoms` | Full | |
| P10: merge preservation | `test_extensions_preserved` (generic) | Partial | Not specific to `is-public` |
| Serialization roundtrip | — | None | |
| External stubs | — | None | |
| Charon failure / off | — | None | |
