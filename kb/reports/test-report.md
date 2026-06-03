---
auditor: test-quality-auditor
date: 2026-06-03
status: 0 critical, 2 warnings, 2 info
---

## Critical

None — critical gaps from initial audit have been addressed:

- Added `test_circular_dependencies_terminate` (cycle safety)
- Added `test_extensions_preserved_through_projection` (P10)
- Added `test_stub_seeds_included` (P3 interaction)
- Added `test_one_to_many_mapping_seeds` (1-to-many C8 feature)

## Warnings

### [W1] No end-to-end CLI output test
- **Issue**: Tests exercise `project_atoms()` but not `cmd_project()` serialization. A regression in envelope construction would not be caught by unit tests alone.
- **Recommendation**: Add an integration test in `tests/project.rs` that writes to a temp file and validates schema

### [W2] No JSON serialization determinism test
- **Issue**: `test_determinism` compares BTreeMap key order but not full JSON output
- **Recommendation**: Compare `serde_json::to_string_pretty` output (excluding timestamp) for stronger P14 coverage

## Info

### [I1] Property-based testing opportunity
- BFS invariants ("no dangling deps", "included ⊇ seeds") are good proptest candidates

### [I2] Partial-missing seeds untested
- `test_missing_seeds_skipped` covers all-ghost; no test mixes valid + invalid mapping keys
