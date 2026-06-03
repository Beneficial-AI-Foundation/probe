---
auditor: test-quality-auditor
date: 2026-06-03
scope: P11–P13 (mappings rename, 1-to-many / C8 fix), probe merge, probe-aeneas, scip-callgraph
status: 0 critical, 5 warnings, 6 info
---

## Summary

Recent changes rename **translation → mapping**, fix **C8** (duplicate `from` keys now collect into `Vec` instead of last-wins), and extend **P13** merge logic for 1-to-many targets. **P11** (probe-aeneas 1-to-1 generation) and **P12** (strategy priority) remain well covered in `probe-aeneas/src/translate.rs` and were not weakened by the hub changes. **P13** core happy paths are covered by three unit tests in `merge.rs`, including a new 1-to-many edge test and a flipped C8 loader test — but **P13 guard branches** (missing target, duplicate dep skip, explicit to→from path) have **no dedicated tests**, **`load_mappings()` schema rejection** of legacy `"probe/translations"` is untested, and **`tests/merge.rs` has no CLI integration test** for `--mappings`. scip-callgraph UI tests were updated for `showMappingLinks` / `'mapping'` link type.

## Recent changes vs test coverage

| Area | Change | Tests added/updated? | Assessment |
|------|--------|----------------------|------------|
| `probe/src/types.rs` | `load_mappings()` → `HashMap<String, Vec<String>>`; schema `"probe/mappings"` | `test_duplicate_from_keys_preserved` (loader only) | Partial — C8 fixed at parse layer; no schema-rejection test |
| `probe/src/commands/merge.rs` | 1-to-many in `merge_atom_maps`; mapping rename | `test_mappings_add_cross_language_edges` (renamed), `test_duplicate_from_keys_preserved`, `test_one_to_many_mapping_produces_multiple_edges` | Partial — happy path + stats; guards untested |
| `probe-aeneas/src/extract.rs` | `MappingMaps` Vec-based; `setup_translation` helper | Existing enrich tests updated (still 1-to-1) | OK — P11 scope unchanged |
| `scip-callgraph/web/src/*.test.ts` | `showMappingLinks`, `'mapping'` link type | Updated snapshots/filters | UI layer only; no 1-to-many graph fixture |
| `tests/merge.rs` | — | None | Gap — no `--mappings` end-to-end |

## Coverage summary

| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| **P11** — mapping generation 1-to-1 (probe-aeneas) | `probe-aeneas/src/translate.rs`: `test_one_to_one_primary_wins`, `test_does_not_double_claim_lean`, `test_no_duplicate_mappings` | **Full** | Hub merge correctly accepts 1-to-many; generation invariant tested in aeneas only |
| **P12** — strategy priority (RQN → file+name → file+lines) | `test_strategy_rust_qualified_name`, `test_strategy_file_display_name`, `test_strategy_file_line_overlap`, `test_no_duplicate_mappings` | **Partial** | Each strategy tested in isolation; strategy-1-over-2/3 via `test_no_duplicate_mappings`; no explicit strategy-2-over-3 fixture |
| **P13** — cross-language edges require existence (core) | `merge.rs::test_mappings_add_cross_language_edges`, `test_one_to_many_mapping_produces_multiple_edges` | **Partial** | Both tests exercise `merge_atom_maps` with existing targets; 1-to-many adds two edges and asserts `mappings_applied == 2` |
| P13 — 1-to-many per-target independence | `test_one_to_many_mapping_produces_multiple_edges` | **Partial** | Both targets exist; no case where one target missing and one present |
| P13 — target absent from merged keys → edge NOT added | — | **None** | `key_set.contains(target)` guard has zero regression tests |
| P13 — dep already present → edge NOT added | — | **None** | `!atom.dependencies.contains(target)` guard untested |
| P13 — both directions (from→to and to→from) | `test_mappings_add_cross_language_edges`, `test_one_to_many_mapping_produces_multiple_edges` | **Partial** | Both maps populated in fixtures; assertions only on from→to caller path (Rust dep → Lean targets) |
| **C8** — duplicate `from` keys preserved | `test_duplicate_from_keys_preserved` | **Partial** | Asserts `from_to` Vec has 2 targets; `_to_from` discarded; no merge assertion from file load |
| C8 — end-to-end via `load_mappings` + merge | — | **None** | `test_one_to_many` builds maps in-memory, bypassing loader |
| `load_mappings()` — rejects `"probe/translations"` | — | **None** | Schema gate in `types.rs:327` untested |
| `load_mappings()` — builds bidirectional `to_from` | `test_duplicate_from_keys_preserved` | **None** | `to_from` map never asserted |
| CLI `probe merge --mappings` | — | **None** | `tests/merge.rs` covers merge without mappings flag |
| scip-callgraph mapping links | `query.test.ts`, `query.integration.test.ts`, `filters.test.ts` | **Partial** | Filter toggle and link type; integration assumes fixture has mapping links |

## Specific audit questions

### Does `test_duplicate_from_keys_preserved` properly assert 1-to-many behavior?

**At the loader layer, yes.** It writes a JSON file with two entries sharing the same `from` key and asserts `from_to["probe:a/1.0/f()"]` contains both targets. This directly regression-tests the C8 fix.

**Gaps:** (1) `_to_from` is ignored — bidirectional map accumulation is not verified. (2) The test stops at `load_mappings()` and does not call `merge_atom_maps`, so the file-load → merge pipeline is not covered in one test.

### Does `test_one_to_many_mapping_produces_multiple_edges` cover the edge application path?

**Yes.** It calls `merge_atom_maps` with in-memory 1-to-many maps, two Lean targets present in the merged key set, and a Rust caller depending on the mapped Rust function. It asserts `stats.mappings_applied == 2` and both Lean code-names appear in the caller's `dependencies`. This exercises the `from_to` iteration loop in `merge_atom_maps` (lines 142–147).

### Is there a test for `load_mappings()` rejecting old `"probe/translations"` schema?

**No.** No test in `types.rs`, `merge.rs`, or `tests/` passes a file with `"schema": "probe/translations"` and expects an error.

### Are there tests for bidirectional 1-to-many (both from→to and to→from)?

**No dedicated test.** Existing fixtures set both `from_to` and `to_from`, but callers always depend on the Rust side, so only the `from_to.get(dep)` branch is meaningfully exercised. A scenario where a Lean atom depends on a Lean mapped name and should gain Rust target(s) via `to_from` is missing. A scenario where one `to` maps back to multiple `from` entries in `to_from` is also untested.

### Is there a test for the guard: target doesn't exist in merged keys → edge NOT added?

**No.** No test supplies a mapping target absent from the merged atom map and asserts `mappings_applied == 0` (or that the caller's deps are unchanged). Same gap for the partial 1-to-many case (one target exists, one ghost target).

## Critical

None. P11, P12, and P13 each have tests exercising their primary behavior. Gaps are guard branches and integration paths, not wholly untested properties.

## Warnings

1. **P13 guard: missing target untested** — `merge_atom_maps` skips edges when `!key_set.contains(target)` (P13). No test adds a mapping to a non-existent code-name and verifies the edge is omitted and stats unchanged. This is the highest-risk untested branch in the changed code.

2. **P13 guard: already-present dep untested** — When the mapped target is already in `atom.dependencies`, the edge must not be re-added and `mappings_applied` must not increment. No regression test exists.

3. **`load_mappings()` schema rejection untested** — Files with `"schema": "probe/translations"` should fail with the `"probe/mappings"` error message. Without a test, a silent serde success + wrong schema string could regress undetected.

4. **No CLI integration test for `--mappings`** — All mapping coverage is unit-level in `merge.rs`. `tests/merge.rs` never passes `--mappings` to the binary, so flag wiring, file I/O, and stderr stats printing are unverified end-to-end.

5. **P13 to→from direction not explicitly exercised** — Both mapping tests assert Rust-caller → Lean-target edges only. The `to_from.get(dep)` branch (lines 149–154) lacks a fixture where the dependency key is on the "to" side of the mapping.

## Info

1. **`test_duplicate_from_keys_preserved` should assert `to_from`** — Two duplicate-from entries should produce `to_from[lean.f] = [rust.f]` and `to_from[lean.g] = [rust.f]`. Trivial addition would close bidirectional loader coverage.

2. **C8 end-to-end test opportunity** — A single test loading a duplicate-from JSON file and merging with atom fixtures would connect loader + merge in one regression anchor (currently split across two tests with different setup styles).

3. **P12 strategy-2-over-3 priority** — No fixture where file+display-name and file+line-overlap both match different Lean atoms for the same Rust atom; lower-priority fallback ordering between strategies 2 and 3 is implicit only.

4. **`build_translations_json` schema untested** — probe-aeneas emits `"schema": "probe/mappings"` but no unit test asserts the envelope shape (only generation logic is tested).

5. **Property-based testing** — Random mapping graphs with subset of targets present/absent would stress P13 guards and 1-to-many independence more thoroughly than hand-written fixtures.

6. **scip-callgraph 1-to-many** — UI tests verify filter toggle and link type enum rename; no fixture asserts a single Rust node with multiple mapping edges (consumer of hub 1-to-many output).

## Integration vs unit

- **Unit (probe hub)**: Three mapping tests in `merge.rs`; C8 loader test uses tempfile + `load_mappings`.
- **Integration (probe hub)**: `tests/merge.rs` — 12 tests, none use `--mappings`.
- **Unit (probe-aeneas)**: P11/P12 in `translate.rs` (20+ tests); enrich path in `extract.rs` uses updated Vec helper but 1-to-1 only.
- **Integration (scip-callgraph)**: Snapshot-based mapping link count in `query.integration.test.ts`; filter unit tests in `query.test.ts` / `filters.test.ts`.
