---
title: "Tool: probe (merge operator)"
last-updated: 2026-03-19
status: draft
---

# probe (merge operator)

**Directory**: `baif/probe/`
**Role**: Central hub — defines [Schema 2.0](../engineering/schema.md) types and the universal merge operator.
**Subcommands**: `merge`

## What this tool does

`probe merge` takes two or more Schema 2.0 JSON files and produces a single merged output. It is the only composition operator in the ecosystem — all tools that need to combine data go through it.

See [architecture.md](../engineering/architecture.md) for how this fits into the data flow.

## Key source files

| File | Purpose |
|------|---------|
| `src/types.rs` | `Atom`, `AtomEnvelope`, `MergedEnvelope<D>`, `SchemaCategory`, `load_envelope()`, `load_translations()` |
| `src/commands/merge.rs` | `merge_atom_maps()`, `merge_generic_maps()`, `normalize_atoms()`, `cmd_merge()` |
| `src/main.rs` | CLI: `probe merge <file1> <file2> [--output] [--translations]` |

## Merge algorithm detail

### Phase 1: Load and validate

1. Parse each input file's envelope
2. Validate `schema-version` starts with `"2."`
3. Detect [schema category](../engineering/glossary.md#schema-category) from `schema` field
4. Validate all inputs belong to the same category
5. Flatten provenance from all inputs

### Phase 2: Normalize

Strip trailing `.` from all code-name keys and dependency references. This handles a legacy verus-analyzer artifact. See [P8](../engineering/properties.md#p8-code-name-normalization).

### Phase 3: Merge

- **Atoms**: `merge_atom_maps()` — first-wins with [stub](../engineering/glossary.md#stub) replacement. See [P6](../engineering/properties.md#p6-atom-merge-is-first-wins-with-stub-replacement).
- **Specs/Proofs**: `merge_generic_maps()` — last-wins. See [P7](../engineering/properties.md#p7-specsproofs-merge-is-last-wins).

### Phase 4: Apply translations (optional)

When `--translations <file>` is provided, for each atom's dependencies:
- Look up each dependency in both directions of the translation map
- If a translated code-name exists in the merged key set and isn't already a dependency, add it
- See [P13](../engineering/properties.md#p13-cross-language-edges-require-existence)

### Phase 5: Write output

Construct merged envelope with `inputs` array (not `source`), serialize with sorted keys for [determinism](../engineering/properties.md#p14-deterministic-output).

## Statistics reported

After merging, the tool prints:

| Metric | Atoms | Specs/Proofs |
|--------|-------|-------------|
| Total entries | yes | yes |
| Stubs replaced | yes | — |
| Stubs remaining | yes | — |
| New entries added | yes | yes |
| Keys normalized | yes | yes |
| Conflicts | yes (real-vs-real, base kept) | yes (overrides, incoming kept) |
| Translations applied | yes (if `--translations`) | — |

## Categorical framework

`probe merge` is described algebraically in `probe/docs/categorical-framework.md`. Key insight: it satisfies [associativity](../engineering/properties.md#p4-merge-associativity), [identity](../engineering/properties.md#p5-merge-identity), and commutativity for disjoint keys. Each probe tool is a [doctrine](../engineering/glossary.md#doctrine); probe-aeneas is a [functor](../engineering/glossary.md#functor) factory.

## probe-extract-check

Subdirectory `probe/probe-extract-check/` (~2.4K LOC). Validates extract JSON against actual source code:
- Checks that `code-path` files exist
- Checks that line ranges are valid
- Verifies atom metadata consistency with source

Used in probe-rust and probe-verus test suites.

## Relationship to probe-verus merge-atoms

`probe merge` generalizes probe-verus's `merge-atoms` command:

| Aspect | probe-verus merge-atoms | probe merge |
|--------|------------------------|-------------|
| Input | Bare JSON (no envelope) | Schema 2.0 enveloped |
| Output | Bare JSON | Schema 2.0 envelope |
| Categories | Atoms only | Atoms, specs, proofs |
| Languages | Rust only | Any |
| Provenance | None | `inputs` array |
| Cross-language | N/A | Via `--translations` |

Merge rules (stub resolution, conflict handling, normalization) are identical.
