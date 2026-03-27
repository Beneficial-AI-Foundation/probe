---
title: "Tool: probe-rust"
last-updated: 2026-03-19
status: draft
---

# probe-rust

**Directory**: `baif/probe-rust/`
**Role**: Extract call graph [atoms](../engineering/glossary.md#atom) from standard Rust projects.
**Subcommands**: `extract`, `callee-crates`, `list-functions`

## Extract pipeline

The `extract` command is the primary pipeline:

```
Cargo.toml → rust-analyzer → SCIP index → call graph → syn AST spans → Schema 2.0 envelope
```

Steps (in `src/commands/extract.rs`):
1. **Validate project** — find `Cargo.toml`
2. **Generate SCIP JSON** — run rust-analyzer, produce SCIP index, convert to JSON. Cached in `<project>/data/`.
3. **Parse SCIP** — build call graph from symbol references
4. **Gather metadata** — git info, package name/version from Cargo.toml
5. **Convert to atoms** — accurate line numbers via syn AST visitor
6. **Detect duplicates** — error unless `--allow-duplicates` (keeps first)
7. **Enrich with Charon** (optional, `--with-charon`) — add `rust-qualified-name` and `is-public` (from Charon LLBC `attr_info.public`) for Aeneas compatibility
8. **Add external stubs** — referenced but unanalyzed dependencies
9. **Wrap and write** — Schema 2.0 envelope to `.verilib/probes/`

## Key challenges

### Accurate function body spans

SCIP gives only the name location of a function (the identifier). To get the full body range (required for `code-text`), probe-rust runs a syn AST visitor (`src/rust_parser.rs`) that walks the source and records start/end lines for each function item.

### Trait implementation disambiguation

When multiple trait impls exist for the same type (e.g. `Add<Scalar>` and `Add<&Scalar>`), SCIP symbol names can be ambiguous. probe-rust uses 4 fallback strategies:
1. Signature text matching
2. Self type matching
3. Definition type context
4. Line number fallback

Implemented across `src/commands/extract.rs` and `src/rust_parser.rs`.

### SCIP caching

SCIP index generation is slow (runs rust-analyzer over the full project). Generated indexes are cached in `<project>/data/` and reused unless `--regenerate-scip` is passed.

Tool downloads cached in `~/.probe-rust/tools/`.

## Subcommands

### `extract`
Primary command. Produces Schema 2.0 envelope with atoms.

| Flag | Default | Description |
|------|---------|-------------|
| `--output, -o` | `.verilib/probes/rust_<pkg>_<ver>.json` | Output path |
| `--regenerate-scip` | false | Force SCIP regeneration |
| `--with-locations` | false | Include `dependencies-with-locations` |
| `--allow-duplicates` | false | Don't error on duplicate code-names |
| `--auto-install` | false | Auto-download scip CLI |
| `--with-charon` | false | Add `rust-qualified-name` via Charon |

### `callee-crates`
BFS traversal from a function through its call graph, grouping callees by crate. No envelope (raw JSON).

| Flag | Description |
|------|-------------|
| `--atoms` | Path to extract output |
| `--function` | Starting function code-name |
| `--depth` | BFS depth |

### `list-functions`
Enumerate all functions via syn AST parsing. No envelope (raw JSON).

| Flag | Description |
|------|-------------|
| `--format` | `text`, `json`, or `detailed` |

## Key source files

| File | Purpose |
|------|---------|
| `src/commands/extract.rs` | Main extraction pipeline |
| `src/commands/callee_crates.rs` | BFS crate dependency traversal |
| `src/commands/list_functions.rs` | Function enumeration |
| `src/rust_parser.rs` | syn AST visitor for function body spans |
| `src/scip_cache.rs` | SCIP index caching and generation |
| `src/tool_manager.rs` | Auto-download of external tools |
| `src/metadata.rs` | Git + Cargo metadata gathering, envelope construction |

## External tool dependencies

| Tool | Required | Auto-install | Notes |
|------|----------|-------------|-------|
| rust-analyzer | yes | no | `rustup component add rust-analyzer` |
| scip CLI | yes | yes (`--auto-install`) | Downloads from GitHub |
| charon | no | no | Only with `--with-charon` |

## Schema differences from probe-verus

probe-rust outputs `schema-version: "2.1"`, a minor version bump from the base 2.0 spec. The 2.1 additions are new optional fields (`rust-qualified-name`, `is-disabled`). Consumers validate that `schema-version` starts with `"2."`, so 2.1 is fully compatible with 2.0 consumers.

Other differences from probe-verus:
- `kind` is always `"exec"` (no proof/spec distinction in standard Rust)
- `dependencies-with-locations` `location` is always `"inner"` (no precondition/postcondition)
- `rust-qualified-name` is optional (only with `--with-charon`)
- `is-public` is optional (only with `--with-charon`; `true` if item is declared `pub`, `false` if private, absent when Charon not used or match failed)
- `is-disabled` is always `false` (no disable concept in standard Rust extraction)
