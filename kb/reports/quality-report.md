---
auditor: code-quality-auditor
date: 2026-06-03
scope: translation→mapping rename, C8 1-to-many HashMap fix (probe, probe-aeneas, scip-callgraph)
status: 0 critical, 4 warnings, 6 info
---

## Critical

*(none)*

Implementation satisfies all audited properties (P3, P6, P7, P10, P11–P13, P14, P17, P19). C8 is fixed in code.

## Warnings

### [W1] Known-bugs section in `properties.md` is stale — C6, C7, C8 fixed in code but still listed as open

- **Location**: `kb/engineering/properties.md` lines 239–241
- **Issue**: The Known bugs section still describes C8 as `load_translations()` last-wins overwrite, C6 as `rqn_to_rust.insert()` overwrite, and C7 as misleading `translation-text`. All three are fixed in the current codebase:
  - **C8**: `load_mappings()` in `src/types.rs` uses `HashMap<String, Vec<String>>` with `or_default().push()`; covered by `test_duplicate_from_keys_preserved` and `test_one_to_many_mapping_produces_multiple_edges` in `merge.rs`.
  - **C6**: `strategy_rust_qualified_name` in `probe-aeneas/src/translate.rs` uses `rqn_to_rust: HashMap<String, Vec<String>>` with disambiguation when len > 1.
  - **C7**: `enrich_with_aeneas_metadata` in `probe-aeneas/src/extract.rs` skips `translation-text` when `start == 0 || end == 0`.
- **Recommendation**: Move C6/C7/C8 to a "Fixed" subsection (with fix references) or remove them from Known bugs so auditors and implementers do not chase resolved defects.

### [W2] Broken links to removed `translations-spec.md` in probe docs

- **Location**: `README.md` line 20, `docs/merge-algorithm.md` lines 159–162, `docs/UI-VIEWS.md` lines 32/60, `docs/CONSUMER_GUIDE.md` line 153
- **Issue**: These files link to `docs/translations-spec.md`, which no longer exists. The spec was renamed to `docs/mappings-spec.md` (confirmed present). `docs/SCHEMA.md` and `src/main.rs` already reference the new name; these four files do not.
- **Recommendation**: Update links and CLI references (`--translations` → `--mappings`) in all four files.

### [W3] Breaking CLI rename not recorded in CHANGELOG

- **Location**: `CHANGELOG.md` `[Unreleased]` section (empty)
- **Issue**: `--translations` → `--mappings`, schema `probe/translations` → `probe/mappings`, and public type renames (`TranslationMapping` → `Mapping`, etc.) are breaking changes for CLI consumers and probe-aeneas git dep users. No entry under `[Unreleased]`.
- **Recommendation**: Add `Changed`/`Removed` entries before release; bump semver appropriately (major for CLI flag and schema string).

### [W4] probe-aeneas normative docs still use old type and schema names

- **Location**: `probe-aeneas/docs/architecture.md` lines 40, 130, 145; `probe-aeneas/docs/SCHEMA.md` lines 418, 457, 467, 628
- **Issue**: Docs reference `TranslationMapping` and "translation entries" where code now uses `probe::types::Mapping` and schema `probe/mappings`.
- **Recommendation**: Rename type references to `Mapping` and update section headings to match `mappings-spec.md` / `kb/engineering/schema.md#mappings-file-format`.

## Info

### [I1] Property verification — implementation passes

| Property | Result | Evidence |
|----------|--------|----------|
| P3 (stub detection) | Pass | `Atom::is_stub()` checks empty `code-path` and lines 0,0 (`types.rs:108–112`) |
| P6 (first-wins) | Pass | `merge_atom_maps` keeps base on real-vs-real; stub replacement; tests |
| P7 (last-wins) | Pass | `merge_generic_maps` overwrites on conflict; tests |
| P10 (extensions) | Pass | `#[serde(flatten)]` on `Atom.extensions`; `test_extensions_preserved` |
| P11 (1-to-1 generation) | Pass | `matched_rust` / `matched_lean` HashSets in `generate_translations` |
| P12 (strategy priority) | Pass | Strategies 1→2→3 called in order in `translate.rs:144–171` |
| P13 (cross-lang edges) | Pass | Both `from_to` and `to_from` iterated; existence + dedup checks (`merge.rs:141–155`) |
| P14 (deterministic output) | Pass | `BTreeMap`/`BTreeSet` for merged atoms and dependencies |
| P17 (category consistency) | Pass | `cmd_merge` rejects mixed categories (`merge.rs:297–306`) |
| P19 (no cross-repo path deps) | Pass | `probe/Cargo.toml` has no `../` path deps; probe-aeneas uses `git = "..."` for probe |

### [I2] C8 fix verified — duplicate `from` keys collect all targets

- **Location**: `src/types.rs:335–347`, `src/commands/merge.rs:1078–1160`
- **Issue**: *(resolved)* Previously `HashMap<String, String>` caused last-wins on duplicate `from` keys. Now `or_default().push()` preserves all targets; merge iterates the Vec and adds each existing target as a dependency. Tests `test_duplicate_from_keys_preserved` and `test_one_to_many_mapping_produces_multiple_edges` cover load and merge paths.

### [I3] `probe/src/` free of stale "translation" identifiers

- **Location**: `rg 'translation' src/` in probe
- **Issue**: No matches. Rename to `Mapping`, `MappingsFile`, `load_mappings`, `mappings_applied`, `--mappings` is complete in hub source.

### [I4] Schema string migration complete in active code paths

- **Location**: `rg 'probe/translations'` across probe, probe-aeneas, scip-callgraph
- **Issue**: No matches in source or active docs. One historical hit in `probe-aeneas/CHANGELOG.md` line 264 (release note for an older version) — acceptable as changelog history.

### [I5] Glossary-consistent naming split is intentional

- **Location**: glossary `cross-language mapping` vs atom extensions `translation-name`/`translation-path`/`translation-text`
- **Issue**: Generic linking concept renamed to "mapping" in merge/KB; Aeneas-specific atom metadata and probe-aeneas internal names (`generate_translations`, `run_extract_with_translations`) retain "translation" for the Rust→Lean transpilation domain. scip-callgraph bridges this: reads `translation-name` from atoms, exposes `mapping_id` / `LinkType 'mapping'` in the UI. Consistent with glossary.

### [I6] Minor KB index phrasing drift

- **Location**: `kb/index.md` line 37
- **Issue**: Still says "translation application" for probe-merge; should say "mapping application" to match `probe-merge.md` and glossary.

## Summary

| Check | Result |
|-------|--------|
| P3, P6, P7, P10, P11–P13, P14, P17, P19 | **Pass** |
| C8 (duplicate `from` keys) | **Fixed** — Vec-based maps + tests |
| C6 (RQN collision) | **Fixed** in probe-aeneas (KB not updated) |
| C7 (translation-text 0,0) | **Fixed** in probe-aeneas enrich (KB not updated) |
| Architecture boundaries (probe vs probe-aeneas) | **Pass** — generation in aeneas, application in merge |
| `probe/src/` rename completeness | **Pass** |
| `probe/translations` schema staleness | **Pass** in code; historical CHANGELOG only |
| Documentation staleness | **Partial** — 4 probe docs with broken links; probe-aeneas SCHEMA/architecture; properties.md known bugs; empty CHANGELOG |
