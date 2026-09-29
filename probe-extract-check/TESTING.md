# probe-extract-check: Test Guide

How the validator checks that JSON from `probe-rust extract`, `probe-verus extract`,
`probe-lean extract`, and `probe-aeneas extract` is correct against the source code.

## What "correct" means — properties to verify

| Property | Description |
|----------|-------------|
| **Completeness** | Every declaration in source has a corresponding atom |
| **No phantoms** | No atoms for declarations that don't exist in source |
| **Correct locations** | `code-path` exists, `lines-start`/`lines-end` bracket the actual declaration |
| **Correct names** | `display-name` matches the declaration at that location |
| **Correct kind** | `kind` matches the actual declaration kind (value sets per language: [kb/engineering/schema.md § Kind values](../kb/engineering/schema.md#kind-values)) |
| **Correct dependencies** | If A calls B, B is in A's dependencies; no spurious deps |
| **Referential integrity** | Every dependency target either exists as an atom key or is a known external |
| **Correct module** | `code-module` matches the actual module path |

Three complementary layers verify these properties:

1. **Source-grounded validators** — check extract JSON against the actual
   source files (`structural.rs`, `source_checker.rs`, `dep_checker.rs`),
   tested in isolation by the unit tests below.
2. **Golden file tests** — curated micro-projects with hand-verified
   `expected.json`, compared via a structural JSON diff that ignores
   volatile fields (`golden.rs`).
3. **Property-based checks** — checks that run against any extract output
   without golden files: completeness counting and location-overlap
   detection (`properties.rs`).

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

Test totals are not maintained by hand in this file — run
`cargo test -p probe-extract-check` for the current counts.

## Test layers

### Layer 1: Unit tests

Source-grounded validators tested in isolation with synthetic data.

#### structural

| Test | What it verifies |
|------|-----------------|
| `test_valid_envelope_no_errors` | Clean envelope with valid atoms produces no errors |
| `test_inverted_line_range` | `lines-start > lines-end` is flagged |
| `test_dangling_dependency` | Dependency target missing from `data` is flagged |
| `test_stubs_skip_line_range_check` | External stubs (0/0 ranges) are exempt from line checks |

#### source_checker

| Test | What it verifies |
|------|-----------------|
| `test_valid_rust_atom` | Correct Rust atom against real temp file produces no errors |
| `test_missing_file` | `code-path` pointing to nonexistent file is flagged |
| `test_line_range_exceeds_file` | `lines-end` beyond file length is flagged |
| `test_name_not_in_span` | `display-name` absent from source span is flagged |
| `test_lean_theorem_kind` | Lean `theorem` keyword correctly matched |
| `test_path_traversal_with_dotdot` | `code-path` with `..` escaping the project root is an error |
| `test_absolute_path_treated_as_error` | Absolute `code-path` is an error, never read |
| `test_symlink_escape_rejected` | Symlink inside the project resolving outside the root is rejected |

#### dep_checker

| Test | What it verifies |
|------|-----------------|
| `test_dep_found_in_span` | Callee name present in caller span produces no diagnostics |
| `test_dep_not_found_in_span` | Callee name absent from caller span is flagged |
| `test_symlink_escape_skipped_in_dep_checker` | Symlink resolving outside the root is skipped, not read |
| `test_cache_uses_canonical_key` | Path aliases (`src/../src/lib.rs`) resolve to the same file and pass the root check |

#### golden

| Test | What it verifies |
|------|-----------------|
| `test_identical_values` | Identical JSON produces no diffs |
| `test_volatile_fields_ignored` | `timestamp`, `commit`, `repo`, `version` differences are ignored |
| `test_missing_key` | Missing key is reported as `MISSING` |
| `test_extra_key` | Extra key is reported as `EXTRA` |
| `test_value_mismatch` | Different values at same path are reported |
| `test_data_timestamp_not_ignored` | Volatile-named fields inside `data` are NOT ignored |

#### properties

| Test | What it verifies |
|------|-----------------|
| `test_no_overlap` | Non-overlapping atoms produce no warnings |
| `test_overlap_detected` | Two atoms at same location are flagged |
| `test_completeness_good_ratio` | 3 atoms for 3 source fns produces no warnings |
| `test_completeness_low_ratio` | 2 atoms for 10 source fns triggers completeness warning |
| `test_lean_completeness` | Lean declaration counting works correctly |
| `test_sorted_dwl_passes` | Sorted `dependencies-with-locations` produces no warnings (P14) |
| `test_unsorted_dwl_warns` | Unsorted `dependencies-with-locations` is flagged (P14) |
| `test_no_dwl_passes` | Atoms without `dependencies-with-locations` are exempt |

### Layer 2: Golden file tests

Each fixture is a micro source project with a hand-verified `expected.json`.
All golden tests run by default (none are ignored).

#### rust_micro

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

#### verus_micro

Source: `src/lib.rs` (Verus)
Atoms: 4

| Test | What it verifies |
|------|-----------------|
| `golden_verus_micro_structural` | Envelope valid |
| `golden_verus_micro_source_and_deps` | All atoms validate against source |
| `golden_verus_micro_kinds` | `spec fn` → spec, `proof fn` → proof, `exec fn` → exec | <!-- enum-ok -->
| `golden_verus_micro_categorized_deps` | `body-dependencies` / `requires-dependencies` populated correctly |

#### lean_micro

Source: `LeanMicro/Basic.lean`
Atoms: 10

| Test | What it verifies |
|------|-----------------|
| `golden_lean_micro_structural` | Envelope valid |
| `golden_lean_micro_source_and_deps` | All atoms validate against source |
| `golden_lean_micro_kinds` | `def`, `theorem`, `structure`, `class`, `instance` kinds correct | <!-- enum-ok -->
| `golden_lean_micro_theorem_deps` | `double_eq_add_self` depends on both `double` and `add` |
| `golden_lean_micro_instance` | Auto-named instance `instHasSizePoint` has correct kind and deps |
| `golden_lean_micro_sorry` | `sorry_example` has `verification-status: "failed"` |

#### aeneas_micro

Source: `rust_src/src/lib.rs` + `lean_src/AeneasMicro.lean`
Atoms: 3

| Test | What it verifies |
|------|-----------------|
| `golden_aeneas_micro_structural` | Envelope valid |
| `golden_aeneas_micro_source_and_deps` | Rust atoms validate against Rust source |
| `golden_aeneas_micro_translations` | All atoms have `translation-name` pointing to Lean |

### Layer 3: Property checks

Run the property checkers (completeness, overlap) against each golden fixture.

| Test | What it verifies |
|------|-----------------|
| `properties_rust_micro` | No property errors, no location overlaps (completeness runs but only errors/overlaps are asserted) |
| `properties_verus_micro` | Same |
| `properties_lean_micro` | Same |
| `properties_aeneas_micro` | Same |

### Live tool comparison (ignored by default)

Run actual extract tools on micro-projects and diff output against golden files.
These require the respective tools to be installed on `PATH`.

| Test | Tool required |
|------|--------------|
| `live_probe_rust_extract` | `probe-rust` |
| `live_probe_verus_extract` | `probe-verus` |
| `live_probe_lean_extract` | `probe-lean` |
| `live_probe_aeneas_extract` | `probe-aeneas` |

### Idempotency (ignored by default)

Run each extract tool twice on the same project and verify outputs are
structurally identical (ignoring volatile fields like timestamp).

| Test | Tool required |
|------|--------------|
| `idempotency_probe_rust` | `probe-rust` |
| `idempotency_probe_verus` | `probe-verus` |
| `idempotency_probe_lean` | `probe-lean` |
| `idempotency_probe_aeneas` | `probe-aeneas` |

## Per-probe integration tests

Each individual probe repository also carries its own integration tests
that validate extract output. How they use probe-extract-check differs
per repo: probe-rust and probe-verus depend on it as a dev-dependency
and call the library API; probe-lean's CI installs the CLI and runs it
as a subprocess over its example extract; probe-aeneas validates
structurally with `serde_json` alone and has no probe-extract-check
dependency. (probe-verus's `tests/extract_check.rs` exists but is not
currently invoked by its CI, which runs `--lib` plus three other named
`--test` targets.)

| Probe | Test file | What it checks |
|-------|-----------|----------------|
| [probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust) | `tests/extract_check.rs` | Loads `examples/rust_curve25519-dalek_4.1.3.json` as `AtomEnvelope`, runs structural checks, validates `probe:` key prefixes and required fields |
| [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus) | `tests/extract_check.rs` | Loads `tests/fixtures/unified_test/atoms.json` as `AtomEnvelope`, validates `probe:` prefixes and Verus-specific kinds (exec/proof/spec) | <!-- enum-ok -->
| [probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas) | `tests/extract_check.rs` | Validates `MergedEnvelope` structure via `serde_json::Value`, runs the full merge pipeline via library API with pre-generated JSON (no external tools needed) |
| [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean) | `Tests/Main.lean` | Loads its example extract, validates envelope fields, a non-empty atom set, `DeclKind` values, and `verification-status` |

probe-rust additionally has an `#[ignore]` live test calling
`cmd_extract` via the library API (requires scip + rust-analyzer).
probe-verus's equivalent was merged into its `extract_backward_compat`
test, which is not `#[ignore]` — it skips at runtime when
verus-analyzer/scip are unavailable.

## Fixture summary

| Fixture | Files | Atoms | Edge cases |
|---------|-------|-------|------------|
| `rust_micro` | 5 `.rs` files | 15 | Mutual recursion, trait impls, generics, closures, macros, type-only module |
| `verus_micro` | 1 `.rs` file | 4 | exec/proof/spec kinds, categorized deps | <!-- enum-ok -->
| `lean_micro` | 1 `.lean` file | 10 | def/theorem/structure/class/instance, sorry | <!-- enum-ok -->
| `aeneas_micro` | 1 `.rs` + 1 `.lean` | 3 | Cross-language translation mappings |

## Adding a new fixture

1. Create a directory under `tests/fixtures/<name>/` with source files
2. Hand-craft `expected.json` following the Schema 3.0 envelope format
3. Sanity-check it with the CLI:
   `probe-extract-check tests/fixtures/<name>/expected.json -p tests/fixtures/<name>` — fix any errors
4. Add tests in `tests/golden_tests.rs` for the new fixture (a filter
   like `cargo test -p probe-extract-check golden_<name>` matches
   nothing — and exits 0 — until this step)
5. Add a `properties_<name>` test if the fixture has source files

## CLI usage

The crate also builds a CLI binary for ad-hoc validation:

```bash
# Structural checks only (no source needed)
probe-extract-check output.json

# Full validation against source project (-p is the short form)
probe-extract-check output.json --project /path/to/project

# Ignore warnings, fail only on errors
probe-extract-check output.json -p /path/to/project --allow-warnings
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Clean run (warnings allowed only with `--allow-warnings`) |
| 1 | Errors found, or warnings found without `--allow-warnings` |
| 2 | Input JSON could not be read or parsed |
