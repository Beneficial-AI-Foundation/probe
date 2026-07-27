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
7. **Enrich with Charon** (optional) — add `rust-qualified-name`, `is-public` (from Charon LLBC `attr_info.public`), and the `charon-def-id`/`charon-version` provenance pair for Aeneas compatibility. Two sources: `--with-charon` reads a Charon LLBC (running charon if needed); `--translation <path>` reads an Aeneas `translation.json` instead (charon `def_id`s come from the manifest's `functions[]` entries, no charon run — implies enrichment and takes precedence over `--with-charon`). The manifest path fails **closed**: a `charon-def-id` is stamped only when a single span-validated candidate matches, so a bad id never feeds the downstream integer join.
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
| `--with-charon` | false | Add `rust-qualified-name` (+ `is-public`, `charon-def-id`/`charon-version`) via a Charon LLBC |
| `--translation <PATH>` | — | Add the same Charon-derived fields from an Aeneas `translation.json` instead of running charon; implies enrichment and takes precedence over `--with-charon` |

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
| charon | no | no | Only with `--with-charon`; not needed with `--translation` (reads an existing Aeneas manifest) |

## Schema differences from probe-verus

probe-rust outputs `schema-version: "3.0"`. Consumers validate that `schema-version` starts with `"3."`. The optional Rust fields (`rust-qualified-name`, `untracked`, and the Charon provenance pair `charon-def-id`/`charon-version`) are carried in the 3.0 envelope; the 3.0 major reflects the breaking `is-disabled`→`untracked` atom-field rename adopted across the ecosystem.

Other differences from probe-verus:
- `kind` is always `"exec"` (no proof/spec distinction in standard Rust)
- `dependencies-with-locations` `location` is always `"inner"` (no precondition/postcondition)
- `rust-qualified-name` is optional (only with Charon enrichment: `--with-charon` or `--translation`)
- `is-public` is optional (only with `--with-charon`; `true` if item is declared `pub`, `false` if private, absent when Charon not used or match failed)
- `charon-def-id`/`charon-version` are optional (only with Charon enrichment; emitted **together or not at all** — a `FunDeclId` and the charon version that produced it, enabling a precise integer join to Aeneas's `translation.json` `def_id`)
- `untracked` is always `false` (no disable concept in standard Rust extraction)
