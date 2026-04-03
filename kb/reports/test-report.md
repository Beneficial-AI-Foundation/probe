---
auditor: test-quality-auditor
date: 2026-04-03
status: 3 critical, 9 warnings, 6 info
---

## Critical

### P5 — Merge identity

No test asserts `merge(A, empty) = A`. `merge_atom_maps` supports an empty tail (`unwrap_or_default()` only covers an empty `Vec`, not `vec![base, BTreeMap::new()]`), but there is no unit or integration test that merges a non-empty base with an empty second map and compares to the base.

### P19 — No cross-repo path dependencies

The KB calls for validation that no `Cargo.toml` uses a `path = "..."` dependency that resolves outside the repository root. This repository has **no** automated test or script (beyond manual review) that enforces P19 across probe-* crates.

### `probe query` subcommand

`src/commands/query.rs` exposes `cmd_query` (entrypoints vs verified-dependencies partition) with **no** `#[cfg(test)]` module, no integration test under `tests/`, and no CLI invocation via `CARGO_BIN_EXE_probe`. This is a functional surface with zero regression safety.

---

## Warnings

### P4 — Merge associativity

There is no test that serializes or structurally compares `merge(merge(A, B), C)` with `merge(A, merge(B, C))`. `test_recursive_merge_flattens_provenance` exercises chaining merged files with provenance flattening but does **not** assert associativity of the atom/proof/spec maps themselves.

### P11 / P12 — Translation matching (probe-aeneas)

These properties are defined for `probe-aeneas`. Neither the `probe` crate nor `probe-verus` (files reviewed) contains tests for strategy priority, `matched_rust` / `matched_lean` invariants, or 1-to-1 mapping. Coverage must be claimed in the aeneas repository, not here.

### P13 — Cross-language edges (partial)

`test_translations_add_cross_language_edges` in `merge.rs` covers adding a translated dependency when the target exists and is absent from `dependencies`. There is no dedicated test that a translated edge is **skipped** when already present (duplicate), or exhaustive checks for both directions beyond the happy path.

### P14 — Deterministic output

Merge and loaders use `BTreeMap`, but no test re-runs the same merge or compares key ordering to a golden snapshot for stability. `properties.md` also notes probe-rust ordering issues; nothing in `probe` tests guards against `HashMap` iteration leaks in sibling tools.

### P16 — Verification status (Lean branch)

`map_verification_status` and unified/proof fixtures thoroughly cover **Verus** → `verification-status`. **probe-lean** sorry / verified mapping described in P16 is **not** exercised in the `probe` or `probe-verus` test trees reviewed.

### P18 — Lean `specified` derived

No tests in the audited scope assert Lean atom behavior (no stored `specified`; inferred from `specs`). Owned by probe-lean / Lean fixtures elsewhere.

### P20 — Language from kind (incomplete + fixture drift)

- **Positive signal**: `probe-verus/src/commands/extract.rs` `test_unified_atom_serialization` asserts `foo` (`exec`) → `"language": "rust"` and `bar` (`proof`) → `"language": "verus"` after merge into unified output.
- **Gaps**: No test in that module asserts a **`spec`** atom maps to `"verus"`. The `convert_to_atoms_with_lines_internal` kind→language rule is not covered by a focused unit test on the conversion path (only indirectly via hand-built JSON in merge tests).
- **Fixture inconsistency**: `probe-verus/tests/fixtures/unified_test/atoms.json` still lists `proof` and `spec` atoms with `"language": "rust"`. `tests/unified_extract.rs` does not assert `language` for `bar` or `baz`, so P20 is **not** enforced at integration level. `probe-verus/tests/fixtures/merge_test/atoms_b.json` and `atoms_combined.json` likewise use `"language": "rust"` for `spec` / `proof` entries—consistent with “preserve whatever the files say” for merge, but **misaligned** with P20 as documentation of intended extract output.

### KB known bugs without regression tests

**C6** and **C7** (see `properties.md`) have no tests in `probe` that fail when those defects are fixed (no targeted regression). **C8** is exercised by `test_duplicate_translation_from_keys_overwrite`, which documents last-wins behavior rather than enforcing a fix.

---

## Info

### P2 — Atom identity

Coverage is implicit (maps keyed by code-name; merge fixtures assume unique keys). A property-based or fuzz test could reject duplicate keys on deserialize if that becomes a hard requirement.

### P8 — Normalization

`test_trailing_dot_normalization` (atoms) and `test_generic_trailing_dot_normalization` (specs) cover trailing-dot stripping. `dependencies-with-locations` normalization is exercised through `normalize_atoms` in merge but could use a dedicated assertion that nested `code-name` fields normalize.

### P20 — End-to-end extract

Golden / `verus_micro` workflows (`extract_check`, `extract_backward_compat`) can assert P20 on **real** extractor output when those tests run with tooling installed; they are optional/skipped in minimal environments. Strengthening golden JSON (`tests/fixtures/extract_golden/golden.json`) to include at least one `spec` with `"language": "verus"` would lock the contract without relying on skipped tests.

### `probe query` — Recommended tests

1. **Unit** (in `query.rs` or `tests/` with `load_atom_file`): verified + non-stub + `language == "rust"` + `kind == "exec"` + not appearing in any `dependencies` → entrypoint; same but listed as a dependency → verified_deps; stub verified → verified_deps; `kind == "spec"` / `proof` or `language == "verus"` → never entrypoint; `code_module` / `display_name` containing `"test"` → excluded from entrypoints; partition size `entrypoints.len() + verified_deps.len()` equals count of verified atoms.
2. **Integration**: `CARGO_BIN_EXE_probe query` on a small fixture file, assert stdout JSON schema for `entrypoints` / `verified_dependencies` and optional `-o` file write.
3. **P20 linkage**: entrypoint detection **depends** on exec atoms carrying `language: "rust"` (see `query.rs`); a regression test with a verified exec incorrectly tagged `verus` should not list it as an entrypoint—documents interaction between P20 and query behavior.

### Property-based testing

P4 (associativity), P5 (identity), and P6 (first-wins / stub replacement) are good candidates for `proptest`/`quickcheck` over small synthetic atom maps.

---

## Coverage Summary

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | `tests/schema_validation.rs` (multiple envelopes); merge integration tests check `schema` / `schema-version` / tool fields | Full | Envelope shape + merged outputs validated |
| P2 | Implicit via `BTreeMap` keys in merge tests | Partial | No explicit “duplicate key” rejection test |
| P3 | `merge.rs` `test_is_stub`; `tests/merge.rs` `test_atoms_stubs_replaced` | Full | Structural stub aligned with `Atom::is_stub()` |
| P4 | — | None | No algebraic associativity test |
| P5 | — | None | No `merge(A, ∅)` test |
| P6 | `merge.rs` stub/real/conflict/new; `tests/merge.rs` atom scenarios | Full | First-wins + stub replacement covered |
| P7 | `merge.rs` `test_generic_last_wins_on_conflict`; `tests/merge.rs` specs/proofs | Full | Last-wins for generic maps + integration |
| P8 | `merge.rs` atom + generic normalization tests | Full | Keys and atom deps; extensions path partial |
| P9 | `merge.rs` `test_recursive_merge_flattens_provenance` (atoms + generic); integration `test_*_provenance_recorded` | Full | Recursive flattening + 2-file provenance |
| P10 | `merge.rs` `test_extensions_preserved` | Full | Flattened extension survives merge |
| P11 | — | None | probe-aeneas scope; not in probe/probe-verus tests reviewed |
| P12 | — | None | Same as P11 |
| P13 | `merge.rs` `test_translations_add_cross_language_edges` | Partial | Happy path; duplicate/skip cases thin |
| P14 | — | Partial | BTreeMap usage; no golden / double-run stability test |
| P15 | `extract.rs` `test_dep_categorization_with_locations` | Full | Union equals categorized subsets (probe-verus) |
| P16 | `extract.rs` `test_status_mapping_all_values`; unified proof fixtures | Partial | Verus strong; Lean branch untested here |
| P17 | `tests/merge.rs` `test_category_mismatch_rejected`; `merge.rs` category tests | Full | Mixing categories fails / detection |
| P18 | — | None | Lean-specific; outside audited tests |
| P19 | — | None | No automated manifest validation |
| P20 | `extract.rs` `test_unified_atom_serialization` (exec rust, proof verus) | Partial | Missing explicit `spec`→`verus`; integration fixtures omit language assertions |
