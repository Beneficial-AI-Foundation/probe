---
title: "Tool: probe-verus"
last-updated: 2026-04-03
status: draft
---

# probe-verus

**Directory**: `baif/probe-verus/`
**Role**: Analyze Verus/Rust codebases for call graphs, specifications, and verification status. Most complex tool in the ecosystem (~13K LOC, 8 subcommands, version 5.0).
**Subcommands**: `extract`, `atomize`, `specify`, `run-verus`, `list-functions`, `merge-atoms`, `setup`, `stubify`

## Unified extract pipeline

The `extract` command runs three steps in sequence, producing a single JSON output:

```
verus-analyzer → SCIP → atomize → verus_syn → specify → cargo verus → run-verus → unified atoms
```

1. **Atomize** — SCIP-based call graph generation (same approach as probe-rust, but using verus-analyzer)
2. **Specify** — Extract function specs via [verus_syn](../engineering/glossary.md#verus_syn) AST; classify with TOML taxonomy rules
3. **Run-verus** — Run `cargo verus`, parse output, map errors to functions via interval trees

Each step can be skipped: `--skip-specify`, `--skip-verify`. Each can also be run independently as its own subcommand.

Output atoms are enriched with optional fields:
- `primary-spec` — specification text (from specify step)
- `verification-status` — `"verified"`, `"failed"`, `"unverified"` (from run-verus step)
- `is-disabled` — whether in analysis scope (from specify step)
- `requires-dependencies`, `ensures-dependencies`, `body-dependencies` — categorized dependency subsets

See [P15](../engineering/properties.md#p15-dependency-completeness) and [P16](../engineering/properties.md#p16-verification-status-mapping).

## Key challenges

### Language assignment

The `language` field on atoms is derived from the declaration `kind`, not from lexical scope (inside/outside `verus!{}` blocks). Exec functions get `language: "rust"` because they are compiled Rust code, even when they carry Verus specifications. Proof and spec functions get `language: "verus"` because they are Verus-only constructs erased at compilation. See [P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope).

### Dual AST parsing

Standard `syn` cannot parse Verus-specific syntax: `verus!{}` blocks, `spec fn`, `proof fn`, `requires`, `ensures`, `decreases`. probe-verus uses `verus_syn`, a modified fork of syn that understands these constructs.

- `src/verus_parser.rs` — verus_syn AST visitor for Verus-specific syntax. Known limitation: some complex nested `verus!{}` blocks and macro-generated items may not be parsed; these show as missing atoms in output.
- `src/rust_parser.rs` (if present, similar to probe-rust) — syn for standard Rust portions

### Interval tree for error mapping

When `cargo verus` reports verification errors, they must be mapped to specific functions. probe-verus uses rust-lapper interval trees for O(log n) lookups:

1. Build interval tree from function line ranges (`FunctionInterval`)
2. For each error location, query the tree to find the containing function
3. Categorize: verified (specs + no errors + no assume/admit), failed (specs + errors), unverified (specs + assume/admit)

### Spec taxonomy

A two-layer classification system for what each verified function proves:

**Layer 1: AST extraction** (`src/verus_parser.rs`)
- `CallNameCollector` visitor walks `verus_syn::Expr` nodes
- Extracts function names called in `requires` and `ensures` clauses
- Distinguishes function calls vs method calls
- Stop-word filtering removes noise

**Layer 2: TOML rules** (`src/taxonomy.rs`)
- `TaxonomyConfig` loaded from TOML file
- Each rule has `label`, `description`, and `match_criteria`
- All criteria within a rule must match (AND logic)
- Within list criteria, any match suffices (OR logic)
- Multi-label output: a function can match multiple rules

Available match criteria:
`mode`, `context`, `ensures_calls_contain`, `requires_calls_contain`, `name_contains`, `path_contains`, `has_ensures`, `has_requires`, `has_decreases`, `has_trusted_assumption`, `ensures_calls_empty`, `requires_calls_empty`, `ensures_fn_calls_contain`, `ensures_method_calls_contain`, `requires_fn_calls_contain`, `requires_method_calls_contain`

Debug: `--taxonomy-explain` prints which rules matched and why.

### Trait implementation disambiguation

Same challenge as probe-rust (4 fallback strategies) but compounded by Verus-specific syntax. verus-analyzer SCIP symbols may be additionally ambiguous for spec/proof/exec variants of the same function.

## Subcommands

### `extract` (primary)
Unified 3-step pipeline. Most commonly used command.

| Flag | Default | Description |
|------|---------|-------------|
| `--output, -o` | `.verilib/probes/verus_<pkg>_<ver>.json` | Output path |
| `--skip-specify` | false | Skip specification extraction |
| `--skip-verify` | false | Skip verification analysis |
| `--with-locations` | false | Include `dependencies-with-locations` |
| `--taxonomy-config` | none | TOML file for spec classification |

### `atomize`
SCIP-based call graph only (step 1 of extract).

### `specify`
Specification extraction only (step 2 of extract). Requires atoms file as input.

### `run-verus`
Verification analysis only (step 3 of extract). Can use existing atoms or run standalone.

### `list-functions`
Function enumeration via verus_syn AST parsing.

### `merge-atoms`
Legacy same-language atom merge. Superseded by `probe merge` but retained for backward compatibility.

### `setup`
Install/check external tool dependencies.

### `stubify`
Convert `.md` files with YAML frontmatter to JSON stubs.

## Key source files

| File | Purpose |
|------|---------|
| `src/commands/extract.rs` | Unified pipeline orchestration |
| `src/commands/atomize.rs` | SCIP → call graph atoms |
| `src/commands/specify.rs` | Spec extraction + taxonomy |
| `src/commands/run_verus.rs` | Verification analysis |
| `src/verus_parser.rs` | verus_syn AST visitor, CallNameCollector |
| `src/taxonomy.rs` | TOML rule engine for spec classification |
| `src/verification.rs` | Verus output parsing, error classification |
| `src/scip_cache.rs` | SCIP index caching |
| `src/tool_manager.rs` | Auto-download of external tools |

## External tool dependencies

| Tool | Required | Auto-install | Notes |
|------|----------|-------------|-------|
| verus-analyzer | yes (for atomize) | yes (`setup`) | Modified rust-analyzer for Verus |
| scip CLI | yes (for atomize) | yes (`setup`) | Downloads from GitHub |
| cargo verus | yes (for run-verus) | no | User must install Verus |

## Config structs

Internal APIs use config structs to share metadata across pipeline steps without threading individual parameters:

- `AtomizeInternalConfig` — config for atomize step
- `SpecifyInternalConfig` — config for specify step
- `ExtractInternalConfig` — config for unified pipeline

## Known inefficiencies

Documented in `docs/VERIFICATION_ARCHITECTURE.md`:
- Redundant source parsing (3 times in full pipeline)
- Multiple data structure builds
- Regex compilation on every run
- Full project parsing when only changed files needed

Interval tree optimization is already implemented. Remaining optimizations are accepted trade-offs — the bottleneck is SCIP generation (external tool), not internal parsing.
