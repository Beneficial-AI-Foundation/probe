# probe-extract-check: Test Guide

## Quick start

```bash
# Run all tests (excluding those that need extract tools installed)
cargo test -p probe-extract-check

# Run all tests including live tool tests (requires probe-rust, probe-verus, etc.)
cargo test -p probe-extract-check -- --include-ignored

# Run a specific test
cargo test -p probe-extract-check golden_rust_micro_mutual_recursion

# Run only unit tests
cargo test -p probe-extract-check --lib

# Run only integration tests
cargo test -p probe-extract-check --test golden_tests
```

## Test layers

### Layer 1: Unit tests (22 tests)

Source-grounded validators tested in isolation with synthetic data.

#### structural (4 tests)

| Test | What it verifies |
|------|-----------------|
| `test_valid_envelope_no_errors` | Clean envelope with valid atoms produces no errors |
| `test_inverted_line_range` | `lines-start > lines-end` is flagged |
| `test_dangling_dependency` | Dependency target missing from `data` is flagged |
| `test_stubs_skip_line_range_check` | External stubs (0/0 ranges) are exempt from line checks |

#### source_checker (5 tests)

| Test | What it verifies |
|------|-----------------|
| `test_valid_rust_atom` | Correct Rust atom against real temp file produces no errors |
| `test_missing_file` | `code-path` pointing to nonexistent file is flagged |
| `test_line_range_exceeds_file` | `lines-end` beyond file length is flagged |
| `test_name_not_in_span` | `display-name` absent from source span is flagged |
| `test_lean_theorem_kind` | Lean `theorem` keyword correctly matched |

#### dep_checker (2 tests)

| Test | What it verifies |
|------|-----------------|
| `test_dep_found_in_span` | Callee name present in caller span produces no diagnostics |
| `test_dep_not_found_in_span` | Callee name absent from caller span is flagged |

#### golden (6 tests)

| Test | What it verifies |
|------|-----------------|
| `test_identical_values` | Identical JSON produces no diffs |
| `test_volatile_fields_ignored` | `timestamp`, `commit`, `repo`, `version` differences are ignored |
| `test_missing_key` | Missing key is reported as `MISSING` |
| `test_extra_key` | Extra key is reported as `EXTRA` |
| `test_value_mismatch` | Different values at same path are reported |
| `test_data_timestamp_not_ignored` | Volatile-named fields inside `data` are NOT ignored |

#### properties (5 tests)

| Test | What it verifies |
|------|-----------------|
| `test_no_overlap` | Non-overlapping atoms produce no warnings |
| `test_overlap_detected` | Two atoms at same location are flagged |
| `test_completeness_good_ratio` | 3 atoms for 3 source fns produces no warnings |
| `test_completeness_low_ratio` | 2 atoms for 10 source fns triggers completeness warning |
| `test_lean_completeness` | Lean declaration counting works correctly |

### Layer 2: Golden file tests (27 tests, of which 8 are ignored)

Each fixture is a micro source project with a hand-verified `expected.json`.

#### rust_micro (10 active tests)

Source: `src/lib.rs`, `src/math.rs`, `src/shapes.rs`, `src/macros.rs`, `src/types_only.rs`
Atoms: 15

| Test | What it verifies |
|------|-----------------|
| `golden_rust_micro_structural` | Envelope and line ranges are valid |
| `golden_rust_micro_source_and_deps` | All atoms validate against source files |
| `golden_rust_micro_atom_count` | Exactly 15 atoms (types_only contributes 0) |
| `golden_rust_micro_mutual_recursion` | `is_even` ↔ `is_odd` bidirectional deps |
| `golden_rust_micro_no_deps_atom` | `standalone()` has empty dependencies |
| `golden_rust_micro_trait_impls` | Two `area()` impls with different code-names and line ranges |
| `golden_rust_micro_generics` | Generic functions (`total_area<T>`, `scale_areas<T>`) present |
| `golden_rust_micro_closure_param` | `apply_transform` with `impl Fn` parameter exists |
| `golden_rust_micro_types_only_no_atoms` | Module with only types/consts contributes zero atoms |
| `golden_rust_micro_macro_generated` | Macro-generated `to_u64`/`to_i64` exist, `convert_both` depends on both |

#### verus_micro (4 active tests)

Source: `src/lib.rs` (Verus)
Atoms: 4

| Test | What it verifies |
|------|-----------------|
| `golden_verus_micro_structural` | Envelope valid |
| `golden_verus_micro_source_and_deps` | All atoms validate against source |
| `golden_verus_micro_kinds` | `spec fn` → spec, `proof fn` → proof, `exec fn` → exec |
| `golden_verus_micro_categorized_deps` | `body-dependencies` / `requires-dependencies` populated correctly |

#### lean_micro (6 active tests)

Source: `LeanMicro/Basic.lean`
Atoms: 10

| Test | What it verifies |
|------|-----------------|
| `golden_lean_micro_structural` | Envelope valid |
| `golden_lean_micro_source_and_deps` | All atoms validate against source |
| `golden_lean_micro_kinds` | `def`, `theorem`, `structure`, `class`, `instance` kinds correct |
| `golden_lean_micro_theorem_deps` | `double_eq_add_self` depends on both `double` and `add` |
| `golden_lean_micro_instance` | Auto-named instance `instHasSizePoint` has correct kind and deps |
| `golden_lean_micro_sorry` | `sorry_example` has `verification-status: "failed"` |

#### aeneas_micro (3 active tests)

Source: `rust_src/src/lib.rs` + `lean_src/AeneasMicro.lean`
Atoms: 3

| Test | What it verifies |
|------|-----------------|
| `golden_aeneas_micro_structural` | Envelope valid |
| `golden_aeneas_micro_source_and_deps` | Rust atoms validate against Rust source |
| `golden_aeneas_micro_translations` | All atoms have `translation-name` pointing to Lean |

### Layer 3: Property checks (4 active tests)

Run the property checkers (completeness, overlap) against each golden fixture.

| Test | What it verifies |
|------|-----------------|
| `properties_rust_micro` | No overlaps, declaration count ratio within bounds |
| `properties_verus_micro` | Same |
| `properties_lean_micro` | Same |
| `properties_aeneas_micro` | Same |

### Live tool comparison (4 ignored tests)

Run actual extract tools on micro-projects and diff output against golden files.
These require the respective tools to be installed on `PATH`.

| Test | Tool required |
|------|--------------|
| `live_probe_rust_extract` | `probe-rust` |
| `live_probe_verus_extract` | `probe-verus` |
| `live_probe_lean_extract` | `probe-lean` |
| `live_probe_aeneas_extract` | `probe-aeneas` |

### Idempotency (4 ignored tests)

Run each extract tool twice on the same project and verify outputs are
structurally identical (ignoring volatile fields like timestamp).

| Test | Tool required |
|------|--------------|
| `idempotency_probe_rust` | `probe-rust` |
| `idempotency_probe_verus` | `probe-verus` |
| `idempotency_probe_lean` | `probe-lean` |
| `idempotency_probe_aeneas` | `probe-aeneas` |

## Test counts

| Category | Active | Ignored | Total |
|----------|--------|---------|-------|
| Unit tests | 22 | 0 | 22 |
| Golden file tests | 23 | 0 | 23 |
| Property checks | 4 | 0 | 4 |
| Live tool comparison | 0 | 4 | 4 |
| Idempotency | 0 | 4 | 4 |
| **Total** | **49** | **8** | **57** |

## Fixture summary

| Fixture | Files | Atoms | Edge cases |
|---------|-------|-------|------------|
| `rust_micro` | 5 `.rs` files | 15 | Mutual recursion, trait impls, generics, closures, macros, type-only module |
| `verus_micro` | 1 `.rs` file | 4 | exec/proof/spec kinds, categorized deps |
| `lean_micro` | 1 `.lean` file | 10 | def/theorem/structure/class/instance, sorry |
| `aeneas_micro` | 1 `.rs` + 1 `.lean` | 3 | Cross-language translation mappings |

## Adding a new fixture

1. Create a directory under `tests/fixtures/<name>/` with source files
2. Hand-craft `expected.json` following the Schema 2.0 envelope format
3. Run `cargo test -p probe-extract-check golden_<name>_source_and_deps` — fix any errors
4. Add tests in `tests/golden_tests.rs` for the new fixture
5. Add a `properties_<name>` test if the fixture has source files

## CLI usage

The crate also builds a CLI binary for ad-hoc validation:

```bash
# Structural checks only (no source needed)
probe-extract-check output.json

# Full validation against source project
probe-extract-check output.json --project /path/to/project

# Ignore warnings, fail only on errors
probe-extract-check output.json --project /path/to/project --allow-warnings
```
