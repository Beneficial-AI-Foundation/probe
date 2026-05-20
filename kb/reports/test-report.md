---
auditor: test-quality-auditor
date: 2026-05-20
scope: P23 (transitive verification status), trusted status preservation
status: 0 critical, 2 warnings, 6 info
---

## Summary

Recent changes are **docs-only** (`docs/verification-statuses.md` Transitive Verification Status subsection; `docs/SCHEMA.md` additions for `trusted`, `trusted-reason`, and `transitive-verification-status`). No new code landed in this diff, but the audit confirms that **P23** is well covered by a dual-layer test suite (16 unit tests in `src/commands/propagate.rs`, 7 integration tests in `tests/propagate.rs`) backed by a rich fixture. **`verification-status: "trusted"`** is exercised in both layers as a non-blocking dependency. **`trusted-reason`** has **no probe test that names the field** — merge relies on the generic P10 extension-preservation test only. Doc additions do not require new tests by themselves, but the gap between documented schema fields and named regression tests is worth closing.

## Recent changes vs test coverage

| Commit | Change | Tests added? | Assessment |
|--------|--------|--------------|------------|
| `991fa9f` | `feat: add probe propagate-verification-status command (P23)` | Yes — unit + integration + fixture | Covered |
| `8c6081e` | `feat: add trusted verification-status and trusted-reason field (P21, P22)` | No probe merge/propagate tests for `trusted-reason` | Partial — P23 tests use `trusted` status only |
| (current) | Docs: transitive scope, `trusted`, `trusted-reason`, `transitive-verification-status` | N/A (docs only) | Existing P23 tests align with documented behavior |

## Coverage summary

| Property / field | Tests | Coverage | Notes |
|------------------|-------|----------|-------|
| **P23** — reverse-BFS contamination | `src/commands/propagate.rs` (16 unit), `tests/propagate.rs` (7 integration) | **Full** | Chain, diamond, cycles, idempotency, counts, determinism |
| P23 — `"transitive"` vs `"local"` labeling | `test_leaf_no_deps`, `test_all_deps_verified`, integration `test_transitive_chain_gets_transitive`, `test_caller_of_unverified_is_local` | Full | |
| P23 — only `"unverified"` / `"failed"` contaminate | `test_one_dep_unverified`, `test_one_dep_failed_contaminates`, `test_explicit_unverified_contaminates_but_missing_does_not` | Full | `"failed"` covered in unit tests; not duplicated in integration fixture |
| P23 — missing `verification-status` transparent | `test_missing_status_does_not_contaminate`, integration `test_missing_status_does_not_contaminate` | Full | Fixture `plain_rust()` has no status |
| P23 — `"trusted"` does not block | `test_dep_trusted_does_not_block`, integration `test_trusted_dep_does_not_block` | Full | Fixture `axiom()` is `trusted` |
| P23 — missing deps treated as trusted | `test_dep_missing_from_map`, integration `test_missing_dep_does_not_block` | Full | Warning text not asserted |
| P23 — non-verified atoms untouched | `test_non_verified_atoms_untouched`, integration spot-checks on `broken()`, `plain_rust()` | Full | |
| P23 — cycles without SCC | `test_cycle_all_verified`, `test_cycle_with_unverified_dep`, integration `test_cycle_with_unverified_dep_all_local` | Full | |
| P14 — deterministic output | `test_deterministic_output` | Partial | Atom-map JSON only; not full pretty-printed CLI envelope |
| **`transitive-verification-status` field** | All P23 tests above | Full | Field is the direct assertion target |
| **`verification-status: "trusted"`** (propagate) | `test_dep_trusted_does_not_block`, integration `test_trusted_dep_does_not_block` | Full | Used as dependency semantics, not as labeled atom |
| **`verification-status: "trusted"`** (merge) | — | **None explicit** | No merge fixture or test asserts `trusted` survives merge |
| **`trusted-reason`** (merge / P10) | `merge.rs::test_extensions_preserved` (generic extension key) | **Partial** | Generic passthrough only; no test with `trusted-reason` |
| **`trusted-reason`** (P22 normalization) | — | **None** | `scripts/summarize_extract.py` implements `TRUST_LABELS`; no automated tests in repo |
| P10 — extensions preserved through merge | `merge.rs::test_extensions_preserved`, `tests/merge.rs` stub-replacement fixtures | Partial | Covers `dependencies-with-locations`, not verification fields |

## P23 sub-property detail

Implementation in `src/commands/propagate.rs` matches P23: reverse dependency index, contamination seeded from explicit `"unverified"` / `"failed"`, reverse-BFS through verified callers, labeling only `verification-status: "verified"` atoms, `BTreeMap`/`BTreeSet` for determinism.

**Unit tests** (`src/commands/propagate.rs::tests`): leaf, full chain, failed/unverified contamination, trusted non-blocking, missing dep, transitive chain with deep unverified, diamond, cycle-all-verified, cycle-with-unverified, missing status, mixed explicit-unverified + missing-status, non-verified untouched, idempotency, deterministic serialization, return counts.

**Integration tests** (`tests/propagate.rs`): end-to-end via `CARGO_BIN_EXE_probe`, fixture `tests/fixtures/propagate_test/atoms.json` encodes verified chain, caller→unverified, trusted axiom, external missing key, untracked callee, cycle touching unverified, envelope preservation (`test_envelope_structure_preserved`).

## Critical

None. P23 and `transitive-verification-status` have substantive unit and integration coverage; no whole rule is untested.

## Warnings

1. **`trusted-reason` not named in any test** — Documented in `docs/SCHEMA.md` and `kb/engineering/schema.md` as a probe-verus/probe-lean field. Probe merge structurally preserves extensions (`test_extensions_preserved`), but there is no regression test that merges atoms carrying `verification-status: "trusted"` **and** `trusted-reason: "axiom"` (or any probe-verus value) and asserts both fields survive stub replacement or multi-file merge. A stub→real replacement scenario would lock P6 + P10 for trust-base atoms.

2. **`verification-status: "trusted"` not tested through merge** — Propagate tests cover trusted as a dependency that does not contaminate. No merge integration test verifies that a real atom with `trusted` status (from probe-verus/lean output) is retained unchanged in merged output. Low runtime risk given serde flatten + P10, but docs now prominently document the value with no merge-level regression anchor.

## Info

1. **Property-based testing** — Random DAGs with random contamination seeds (`proptest`/`quickcheck`) would complement the hand-written P23 suite, especially with cycle injection.

2. **Integration: `"failed"` contamination** — Unit test `test_one_dep_failed_contaminates` covers this; the propagate integration fixture has `broken()` as `"unverified"` only. Adding a `"failed"` atom to the fixture would mirror unit coverage at CLI level.

3. **Integration: stderr warnings** — Missing-dep warnings are printed (`eprintln!`) but integration tests do not assert subprocess stderr contains the documented substring.

4. **CLI edge branches** — Default output path, schema-version rejection for non-`2.x`, and missing `data` field are not covered by `tests/propagate.rs`.

5. **Schema validation** — `tests/schema_validation.rs` validates generic extensions (`dependencies-with-locations`) but has no case with `verification-status: "trusted"`, `trusted-reason`, or `transitive-verification-status`. JSON Schema uses passthrough for optional atom fields, so this is low risk but would document the new fields in validation fixtures.

6. **P22 consumer script** — `scripts/summarize_extract.py` maps tool-specific `trusted-reason` values via `TRUST_LABELS`; no pytest or golden-output test exists in this repo. Normalization logic is untested here (may live in probe-verus/probe-lean repos).

## Integration vs unit

- **End-to-end CLI for P23**: Yes — `tests/propagate.rs` runs the real binary, writes output, reloads envelope.
- **End-to-end CLI for trusted/trusted-reason preservation**: No — `tests/merge.rs` fixtures do not include trust-base fields.
- **Fixture quality (propagate)**: `tests/fixtures/propagate_test/atoms.json` covers the main P23 scenarios; `axiom()` has `trusted` status but omits `trusted-reason` (optional per schema).
