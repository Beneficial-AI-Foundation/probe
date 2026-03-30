---
title: Architecture
last-updated: 2026-03-19
status: draft
---

# Architecture

The probe ecosystem is a multi-language code analysis pipeline. Each tool targets a specific language, extracts structured data (call graphs, specs, verification status), and outputs JSON conforming to the [Schema 2.0](schema.md) envelope format. A central merge operator composes outputs across tools and languages.

## Components

Five tools, intentionally kept in separate directories under `baif/`:

```
probe/           Central hub — schema types, merge operator, extract-check validator
probe-rust/      Rust call graph extraction via rust-analyzer + SCIP
probe-verus/     Verus/Rust analysis: call graphs + specs + verification status
probe-lean/      Lean 4 dependency graphs + sorry detection (written in Lean)
probe-aeneas/    Cross-language bridge: generates Rust↔Lean translation mappings
```

### probe (central hub)

**Role**: Defines the canonical [Schema 2.0](schema.md) types and the universal `merge` operator.

- `src/types.rs` — `Atom`, `AtomEnvelope`, `MergedEnvelope<D>`, `SchemaCategory`, loading/validation
- `src/commands/merge.rs` — Merge algorithm: stub replacement for atoms, last-wins for specs/proofs, optional cross-language edges via `--translations`
- `probe-extract-check/` — Validator that checks extract JSON against actual source code

**Subcommands**: `merge`

**Dependencies**: None (leaf crate).

### probe-rust

**Role**: Extract call graph [atoms](glossary.md#atom) from standard Rust projects.

**Pipeline**: rust-analyzer → SCIP index → call graph parsing → syn AST for accurate spans → Schema 2.0 envelope

**Key challenges**:
- Trait implementation disambiguation (4 fallback strategies)
- Accurate function body spans (SCIP gives name location only; syn AST gives body range)
- SCIP caching in `<project>/data/` to avoid re-running slow tools

**Subcommands**: `extract`, `callee-crates`, `list-functions`

**External tools**: rust-analyzer (required), scip CLI (auto-downloadable)

### probe-verus

**Role**: Analyze Verus/Rust codebases for verification status, call graphs, and specifications. Most complex tool (~13K LOC, 8 subcommands).

**Pipeline** (unified `extract` command):
1. **Atomize** — SCIP-based call graph (same approach as probe-rust, using verus-analyzer)
2. **Specify** — Extract function specs via verus_syn AST; classify with TOML taxonomy rules
3. **Run-verus** — Run `cargo verus`, parse output, map errors to functions via interval trees

**Key challenges**:
- Dual AST parsing: `syn` for standard Rust + `verus_syn` for `verus!{}` blocks, `spec fn`, `proof fn`
- Interval tree (rust-lapper) for O(log n) error-to-function mapping
- AST-based spec taxonomy: `CallNameCollector` visitor extracts function names from requires/ensures clauses; TOML rules classify them

**Subcommands**: `extract`, `atomize`, `specify`, `run-verus`, `merge-atoms`, `callee-crates`, `list-functions`, `setup`, `stubify`

**External tools**: verus-analyzer (auto-downloadable), scip CLI (auto-downloadable), cargo verus

### probe-lean

**Role**: Extract dependency graphs from Lean 4 projects. Written entirely in Lean 4.

**Pipeline** (unified `extract` command):
1. Build target project via `lake build`
2. Walk Lean environment, extract declarations and dependencies (type vs term)
3. Detect sorry warnings from build output
4. Compute specs (reverse dependency edges from theorems)
5. Wrap in Schema 2.0 envelope

**Key challenges**:
- Written in Lean (cannot be a Cargo workspace member — primary reason for repo separation)
- Two-phase build: discover libraries from `lakefile.toml` → build → walk environment
- Type vs term dependency distinction
- Version-specific `.olean` files (must match target project toolchain)

**Subcommands**: `extract`, `viewify`

**External tools**: Lean 4 toolchain (elan, lake)

### probe-aeneas

**Role**: Bridge between Rust and Lean for Aeneas-transpiled projects. Generates cross-language translation mappings, then delegates merging to `probe merge`.

**Pipeline** (`extract` command, typically `probe-aeneas extract <project_path>`):
1. If positional project path given, parse `aeneas-config.yml` to resolve Rust crate (`crate.dir`) and Lean project (project root); otherwise use explicit `--rust-project` / `--lean-project` flags
2. Auto-run probe-rust and probe-lean in parallel (scoped threads)
3. Load `functions.json` (Aeneas-generated Rust↔Lean name mappings, reused from project root if present)
4. Generate translations via three-strategy matching (see [properties.md](properties.md#p12-translation-strategy-priority))
5. Call `probe::merge::merge_atom_maps` with translations
6. Enrich merged atoms with Aeneas metadata (`translation-name`, `translation-path`, `translation-text`, `is-disabled`, `is-relevant`, `is-public`)

**Key insight**: probe-aeneas is a *[functor](glossary.md#functor) factory*. It produces the [translation mapping](glossary.md#translation-mapping); `probe merge` applies it. Domain knowledge about [Aeneas](glossary.md#aeneas) lives here; generic composition lives in probe. (The algebraic structure is detailed in `probe/docs/categorical-framework.md`, a non-normative design document.)

**Subcommands**: `extract`, `translate`, `listfuns`

**External tools**: probe-rust (auto-installable), probe-lean (auto-cloned + built), lake

## Data flow

```
Target Projects (Rust, Lean, Verus)
    │
    ├── probe-rust extract ──────→ rust_atoms.json     (Schema 2.0)
    ├── probe-lean extract ──────→ lean_atoms.json     (Schema 2.0)
    ├── probe-verus extract ─────→ verus_atoms.json    (Schema 2.0)
    │
    ├── probe-aeneas extract ────→ aeneas_atoms.json   (merge + translate Rust↔Lean)
    │       │
    │       ├─ runs probe-rust and probe-lean in parallel
    │       ├─ generates translations from functions.json
    │       └─ calls probe merge internally
    │
    └── probe merge ─────────────→ merged_atoms.json   (generic cross-tool merge)

All JSON → scip-callgraph (web UI consumer)
```

## Why separate directories

See [decisions/001-separate-repos.md](../decisions/001-separate-repos.md) *(planned)*.

Summary:
1. **probe-lean requires Lean toolchain** — cannot be a Cargo workspace member
2. **Different external dependencies** — each tool has its own analyzer (rust-analyzer, verus-analyzer, lake)
3. **Different versioning cadence** — probe-verus at v5.0, others at earlier versions
4. **probe-aeneas is an orchestrator**, not a peer extractor — different architectural role
5. **Consolidation opportunity is in shared types** (probe crate as library), not repo merging

## Output locations

All tools write to a `.verilib/` directory inside the target project:

```
target-project/.verilib/
  probes/
    rust_<package>_<version>.json      # probe-rust output
    verus_<package>_<version>.json     # probe-verus output
    lean_<package>_<version>.json      # probe-lean output
  views/
    molecule_all.json                  # Filtered projections
  translations/
    verus_<pkg>__lean_<pkg>.json       # Cross-language mappings
  config.json                          # User/project configuration
```

Each file in `probes/` is self-describing via its envelope. Filename is for human convenience; consumers read the envelope.

## Shared patterns

### Auto-install
probe-rust and probe-verus auto-download external tools (scip CLI, verus-analyzer) to `~/.probe-{tool}/tools/`. Version resolution: env var override → GitHub API latest → compiled-in fallback.

### SCIP caching
Generated SCIP indexes cached in `<project>/data/` to avoid re-running slow analysis. Both probe-rust and probe-verus use this pattern.

### Schema 2.0 envelope
Every output file is wrapped in a metadata envelope containing tool info, source provenance, timestamp, and the data payload. See [schema.md](schema.md).
