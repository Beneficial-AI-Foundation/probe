---
auditor: test-quality-auditor
date: 2026-05-12
scope: propagate-verification-status (`src/commands/propagate.rs`, `tests/propagate.rs`, fixture)
status: 0 critical, 2 warnings, 5 info
---

## Summary

Tests for `probe propagate-verification-status` align well with **P23** (reverse-BFS contamination): core semantics, trusted vs contaminating statuses, missing dependencies, missing `verification-status`, diamonds, cycles, idempotency, non-verified atoms left unchanged, and basic determinism are covered in unit tests; integration tests exercise the real binary with a rich fixture and preserve envelope fields (P1-adjacent). The main gap is **symmetric coverage for `"failed"` vs `"unverified"` as contamination seeds**, plus **no automated empty-input case**.

## P23 sub-property coverage

| P23 sub-property | Tests | Coverage | Notes |
|------------------|-------|----------|-------|
| Reverse-BFS propagation (callers inherit contamination) | `test_one_dep_unverified`, `test_transitive_chain`, `test_diamond_dependency`, integration `test_caller_of_unverified_is_local` | Full | Multi-hop and diamond covered in unit tests; fixture exercises caller→broken |
| Cycles: same scope for cycle members without SCC | `test_cycle_all_verified`, `test_cycle_with_unverified_dep`, integration `test_cycle_with_unverified_dep_all_local` | Full | Matches P23 note on cycles |
| `"transitive"` vs `"local"` labeling | `test_leaf_no_deps`, `test_all_deps_verified`, chain/diamond/cycle tests, integration chain | Full | |
| Only explicit `"unverified"` / `"failed"` contaminate | `test_missing_status_does_not_contaminate`, `test_explicit_unverified_contaminates_but_missing_does_not`, integration `test_missing_status_does_not_contaminate` | Partial | **`"failed"` never used as a dependency contaminant in any test** (see Warnings) |
| Missing `verification-status` does not contaminate | Same as above + fixture `plain_rust()` | Full | |
| `"trusted"` does not block transitive scope | `test_dep_trusted_does_not_block`, integration `test_trusted_dep_does_not_block` | Full | |
| Missing deps treated as trusted (warning logged) | `test_dep_missing_from_map`, integration `test_missing_dep_does_not_block` | Full | Warning text not asserted (see Info) |
| Non-verified atoms: no `transitive-verification-status` | `test_non_verified_atoms_untouched`, integration asserts `None` on `broken()`, `plain_rust()` | Full | Integration does not enumerate every non-verified atom in fixture but spot-checks |
| Deterministic structures / serialization | `test_deterministic_output` (atoms JSON), `BTreeMap`/`BTreeSet` in implementation | Partial | Same logical graph twice; **full pretty-printed envelope** not golden-tested (see Info) |

## Critical

None. P23 is exercised across unit and integration tests; no whole rule is untested.

## Warnings

1. **`"failed"` as contamination source (P23)** — `is_contamination_source` treats `"failed"` like `"unverified"`, but no test builds a verified atom that depends on a **`verification-status: "failed"`** dependency and asserts `transitive-verification-status: "local"`. `test_non_verified_atoms_untouched` only checks that failed atoms get no scope field. This is **partial coverage** for the explicit `"unverified"` / `"failed"` seed rule.

2. **Empty atom graph** — No unit test for `propagate_verification_status` on an empty `BTreeMap`, and no integration test for an envelope with empty `data`. Behavior is straightforward (counts zero, no writes), but **regressions** (e.g. panic on empty verified set iteration—currently safe) would go unnoticed.

## Info

1. **Property-based testing** — Random DAGs with random contamination seeds would strengthen confidence in reverse-BFS vs a hand-written suite (`proptest`/`quickcheck`), especially combined with cycle injection.

2. **Integration: stderr** — Unit tests do not capture `eprintln!` warnings for missing deps; integration could assert subprocess stderr contains the documented warning substring for a fixture with only missing deps.

3. **Determinism of CLI output** — `test_deterministic_output` compares `serde_json::to_string` of the atom map after propagation, not `serde_json::to_string_pretty` of the full envelope as written by `cmd_propagate_verification_status`. Low risk given `BTreeMap` keys, but a **golden file** or byte-stable round-trip check would lock P14 for the command output.

4. **CLI branches** — Default output path (`propagate_<stem>.json` when `-o` omitted), schema-version rejection for non-`2.x`, and missing `data` handling are not covered by `tests/propagate.rs` (only `-o` path + valid fixture).

5. **Idempotency** — Covered in unit tests only; integration could run propagate twice on tempfile output if desired (redundant if core stays stable).

## Integration vs unit

- **End-to-end CLI**: Yes — `CARGO_BIN_EXE_probe` + `propagate-verification-status` + `-o`, then reload envelope and inspect atoms (`tests/propagate.rs`).
- **Fixture quality**: `tests/fixtures/propagate_test/atoms.json` encodes chain, trusted leaf, external missing key, untracked callee, and cycle touching `broken()`; aligns with scenarios tested in `propagate.rs` unit tests.
