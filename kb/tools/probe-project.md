---
title: "Tool: probe project (graph projection)"
last-updated: 2026-06-03
status: draft
---

# probe project (graph projection)

**Directory**: `baif/probe/`
**Role**: Extract a focused subgraph from an atom file using cross-language mapping seeds.

## What this tool does

`probe project` takes a Schema 2.0 atom file and a [mappings file](../engineering/schema.md#mappings-file-format), uses all mapping endpoints (`from` + `to` code-names) as seeds, then expands via BFS in both directions with separate depth controls. The output is a trimmed atom file containing only the projected subgraph.

This is the server-side complement to scip-callgraph's client-side source/sink filtering. It produces reusable, shareable JSON artifacts suitable for CI pipelines, demos, or focused analysis.

See [architecture.md](../engineering/architecture.md) for how this fits into the data flow.

## Key source files

| File | Purpose |
|------|---------|
| `src/commands/project.rs` | `project_atoms()` (pure function), `cmd_project()` (CLI handler), `ProjectStats` |
| `src/main.rs` | CLI: `probe project <input> --mappings <file> [--forward-depth N] [--reverse-depth N] [-o output] [--emit-focus]` |

## Algorithm

### Step 1: Load and validate

1. Load atom file via `load_atom_file()` — accepts both single-tool and `probe/merged-atoms` envelopes
2. Load mappings file via `load_mappings()` — validates `probe/mappings` schema
3. Build seed set: all `from` and `to` keys that exist in the atom data (missing keys logged, not errored)

### Step 2: Build reverse adjacency index

Iterate all atoms to build a "who depends on me?" map: `BTreeMap<String, BTreeSet<String>>`. Only built when `--reverse-depth > 0`.

### Step 3: BFS expansion

- **Forward** (callee direction): from seeds, follow `atom.dependencies` up to `--forward-depth`
- **Backward** (caller direction): from seeds, follow the reverse index up to `--reverse-depth`
- Union forward + backward + seeds into the included set

### Step 4: Filter and trim

- Keep only atoms whose code-name is in the included set
- **Trim dependencies**: remove references to atoms outside the projection (no dangling refs)
- Count trimmed deps for metadata

### Step 5: Write output

- Reuses `probe/merged-atoms` schema (compatible with scip-callgraph without viewer changes)
- Carries provenance from input (`inputs` for merged, wrapped `source` for single-tool)
- Adds `projection` metadata block with seeds, depths, atom counts, trimmed dep count

## CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--mappings` | required | Mappings file defining the seed set |
| `--forward-depth` | 2 | BFS depth following callees from seeds |
| `--reverse-depth` | 0 | BFS depth following callers of seeds |
| `--output` | `projected.json` | Output file path |
| `--emit-focus` | false | Also emit a focus-set JSON for `?focus=` |

## Typical usage

```bash
# Merge, then project to mapping seeds
probe merge lean.json rust.json --mappings map.json -o merged.json
probe project merged.json --mappings map.json --forward-depth 3 -o focused.json --emit-focus

# Load directly in scip-callgraph
# or use ?focus=focused_focus.json with the full merged graph
```

## Properties

- **Envelope completeness** ([P1](../engineering/properties.md#p1-envelope-completeness)): output is a valid `probe/merged-atoms` Schema 2.0 envelope with all required fields
- **Provenance preserved** ([P9](../engineering/properties.md#p9-provenance-is-preserved)): input `inputs` (merged) or `source` (single-tool, wrapped) carried through to output
- **Extensions preserved** ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)): atoms are cloned, so language-specific extension fields survive projection
- **Deterministic** ([P14](../engineering/properties.md#p14-deterministic-output)): BFS over BTreeMap/BTreeSet keys produces identical output for identical input
- Seeds that don't exist in atom data are silently skipped (logged to stderr)
- Stubs in the seed set are included (they may represent API boundaries)
- The `projection` metadata block is defined in `schemas/atom-envelope.schema.json` as an optional field on the merged envelope
- **Input restriction**: only atoms-category files are accepted (specs/proofs are rejected by `load_atom_file()`)

## Focus-set emission

When `--emit-focus` is set, writes a companion `<stem>_focus.json` compatible with scip-callgraph's `?focus=<url>` parameter:

```json
{
  "focus_nodes": ["probe:AEADScheme.decrypt", ...],
  "metadata": {
    "description": "Projection: 10 seeds, forward-depth 3, reverse-depth 0, 67 atoms"
  }
}
```
