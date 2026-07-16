---
title: "Tool: probe-aeneas"
last-updated: 2026-06-03
status: draft
---

# probe-aeneas

**Directory**: `baif/probe-aeneas/`
**Role**: Cross-language bridge for Aeneas-transpiled projects. Generates Rust↔Lean [cross-language mappings](../engineering/glossary.md#cross-language-mapping) and delegates merging to `probe merge`.
**Subcommands**: `extract`, `translate`, `listfuns`

## What this tool is (and isn't)

probe-aeneas is a [functor factory](../engineering/glossary.md#functor), not a merge engine. It:
- **Generates** the cross-language mapping between Rust and Lean code-names
- **Orchestrates** running probe-rust and probe-lean
- **Enriches** merged atoms with Aeneas-specific metadata
- **Delegates** the actual merge to `probe::merge::merge_atom_maps()`

Domain knowledge about Aeneas transpilation lives here. Generic composition lives in [probe merge](probe-merge.md).

## Extract pipeline

The `extract` command is the full pipeline (`src/extract.rs`):

```
inputs → parallel extraction → load functions.json → translate → merge → enrich → envelope
```

1. **Resolve project** — if positional `PROJECT` given, parse `aeneas-config.yml` to derive Rust/Lean paths, optional functions.json, and Charon config
2. **Pre-generate Charon LLBC (legacy/no-manifest path only)** — when there is **no** `translation.json`, and `aeneas-config.yml` has a `charon` section, run `charon` with the full project-specific settings (cargo args, start-from, exclude, opaque) and cache at `<rust_project>/data/charon.llbc`, so `probe-rust --with-charon` finds a pre-built LLBC with the correct compilation flags. **When a `translation.json` is present this step is skipped entirely**: charon already ran once inside Aeneas to produce the manifest, so probe-rust is invoked with `--translation <path>` instead of `--with-charon` and reads charon `def_id`s directly from the manifest (no second charon run).
3. **Validate inputs** — exactly one Rust source + one Lean source + functions.json
4. **Resolve inputs** — if project paths given, run extractors; if JSON given, use directly
5. **Parallel extraction** — when both project paths given, run probe-rust and probe-lean in parallel via scoped threads
6. **Generate mappings** — priority-ordered matching against functions.json: the charon-`def_id` join (Strategy 0) followed by three name/location strategies (see below)
7. **Merge** — call `merge_atom_maps()` from probe crate with mappings
8. **Enrich** — add Aeneas-specific metadata to merged atoms
9. **Wrap** — Schema 2.0 envelope with `probe-aeneas/extract` schema

### Input modes

```
# Aeneas project directory (simplest — reads aeneas-config.yml to auto-detect paths)
probe-aeneas extract path/to/aeneas/project

# Pre-generated JSON (advanced)
probe-aeneas extract --rust atoms_rust.json --lean atoms_lean.json --functions functions.json

# Explicit project paths (advanced)
probe-aeneas extract --rust-project ./curve25519-dalek --lean-project ./dalek-lean

# Mixed
probe-aeneas extract --rust atoms_rust.json --lean-project ./dalek-lean --functions functions.json
```

The positional `PROJECT` argument parses `aeneas-config.yml` to derive `crate.dir` (Rust crate path) and uses the project root as the Lean project. If `functions.json` exists at the project root, it is reused. This aligns with the `probe-<tool> extract <project_path>` convention used by all other probes.

## Mapping generation

Implemented in `src/translate.rs`. Strict priority order — see [P12](../engineering/properties.md#p12-mapping-strategy-priority).

### Strategy 0: charon-`def_id` (highest priority)

Integer join on the charon `FunDeclId`: a Rust atom's `charon-def-id` extension equals Aeneas's `translation.json` `def_id`, binding to the family's primary (non-loop) Lean def with no name normalization (confidence: `exact`, method `charon-def-id`). Runs first, but is **provenance-gated** — it fires only when the atom's `charon-version` matches the manifest's `charon_version` (best-effort provenance; version equality is not proof of an identical run). Only the manifest's `functions` array feeds the join, since `globals`/`trait_impls` are numbered in charon's separate id spaces. A no-op for atoms that do not carry `charon-def-id`, which fall through to the name/location strategies below.

### Strategy 1: Rust-qualified-name

Uses Charon-derived fully qualified names from functions.json.

1. Build lookup: `normalized_rqn → Vec<rust_code_names>` from Rust atoms with `rust-qualified-name` extension
2. For each function in functions.json, normalize its `rust_name`
3. Look up candidates in the RQN map
4. If single candidate: match directly (confidence: `exact`)
5. If multiple candidates: disambiguate by file + line overlap (confidence: `exact-disambiguated`)

Name normalization (`normalize_rust_name`): strips lifetime parameters, brace wrappers, generics, spaces.

### Strategy 2: File + display-name

For unmatched Rust atoms with non-empty `code-path`:
1. Extract base name (last component after `::`)
2. Look up `(normalized_source_file, base_name)` in functions.json index
3. Match only if exactly 1 candidate (confidence: `file-and-name`)

### Strategy 3: File + line-overlap (lowest priority)

For unmatched Rust atoms with valid line ranges:
1. Get normalized source path
2. Look up source file in functions.json index
3. For each candidate, check line range overlap (tolerance: 10 lines)
4. Keep best overlap (confidence: `file-and-lines`)

### 1-to-1 constraint

See [P11](../engineering/properties.md#p11-mapping-generation-is-1-to-1-probe-aeneas). Enforced by `matched_rust` and `matched_lean` HashSets. Once an atom is claimed by any strategy, no later strategy can claim it again.

## Enrichment

After merge, probe-aeneas adds Aeneas-specific fields. Translation-specific fields are added to atoms that have translations; other fields are set on all Rust atoms.

**Translation-specific fields** (only on translated atoms):

| Field | Source | Description |
|-------|--------|-------------|
| `translation-name` | functions.json `lean_name` | Corresponding name in other language |
| `translation-path` | Lean atom's `code-path` | File path of translation |
| `translation-text` | Lean atom's `code-text` | Line range of translation |

**All Rust atoms**:

| Field | Source | Description |
|-------|--------|-------------|
| `is-disabled` | functions.json, translation | `true` if the function's RQN is not in functions.json, or its Lean translation carries the `@[out_of_scope]` attribute (see [P25](../engineering/properties.md#p25-atoms-not-in-the-verification-build-are-out-of-scope)) |
| `is-relevant` | functions.json | `true` if the function's RQN appears in functions.json |
| `is-public` | probe-rust (Charon) or default | `true` if declared `pub` per Charon; `false` if private or visibility data unavailable |

## Subcommands

### `extract` (primary)
Full pipeline: extract + translate + merge + enrich.

| Argument / Flag | Description |
|-----------------|-------------|
| `PROJECT` | Aeneas project directory (contains `aeneas-config.yml`). Auto-detects Rust and Lean paths. |
| `--rust` | Pre-generated Rust atoms JSON |
| `--rust-project` | Rust project directory (runs probe-rust) |
| `--lean` | Pre-generated Lean atoms JSON |
| `--lean-project` | Lean project directory (runs probe-lean) |
| `--functions` | Path to functions.json |
| `--aeneas-config` | Optional Aeneas config for manual overrides |
| `--output, -o` | Output path |

The positional `PROJECT` arg conflicts with `--rust`, `--rust-project`, `--lean`, and `--lean-project`.

### `translate`
Generate cross-language mappings only (no merge/enrich). Requires pre-generated atoms.

### `listfuns`
Run `lake exe listfuns` to generate functions.json from an Aeneas-transpiled Lean project.

## Key source files

| File | Purpose |
|------|---------|
| `src/extract.rs` | Pipeline orchestration, parallel extraction |
| `src/translate.rs` | Mapping generation (charon-`def_id` join + three name/location strategies), name normalization |
| `src/extract_runner.rs` | Auto-download and run probe-rust/probe-lean |
| `src/listfuns.rs` | Wrapper for `lake exe listfuns` |
| `src/gen_functions.rs` | Parse Aeneas-generated Lean files for name mappings |
| `src/aeneas_config.rs` | Manual override support (is-hidden, is-ignored) |
| `src/types.rs` | `FunctionRecord`, `LineRange`, `TranslateStats` |

## Key types

### FunctionRecord
Single entry from functions.json:
- `lean_name` (string, required)
- `rust_name` (Option<String>)
- `source` (Option<String>) — source file path
- `lines` (Option<String>) — line range as `"L292-L325"`
- `is_hidden` (bool, default false)
- `is_extraction_artifact` (bool, default false)

### LineRange
Parsed from `"L<start>-L<end>"` format.
- `parse()`: validates start ≤ end
- `overlaps(other, tolerance)`: checks overlap with tolerance
- `overlap_amount(other)`: computes overlap in lines

## External tool dependencies

| Tool | Required | Auto-install | Notes |
|------|----------|-------------|-------|
| probe-rust | yes (if `--rust-project`) | yes (cargo install) | |
| probe-lean | yes (if `--lean-project`) | yes (pre-built binary or source build) | |
| charon | yes on the legacy path (no `translation.json`, with a `charon` section) | no (managed by probe-rust `--auto-install`) | Pre-generates LLBC with full project config; not run when a `translation.json` is present (probe-rust reads `def_id`s via `--translation`) |
| lake | yes (for listfuns) | no | Lean toolchain |

## Dependency on probe crate

probe-aeneas imports `probe` as a git dependency (`git = "https://github.com/Beneficial-AI-Foundation/probe"`). Uses:
- `probe::commands::merge::merge_atom_files` — merge algorithm (file-level convenience over `merge_atom_maps`)
- `probe::types::Atom`, `Mapping`, `MergedAtomEnvelope`, `InputProvenance`, `Tool`

This is the only probe tool with a Rust crate dependency on the central hub.
