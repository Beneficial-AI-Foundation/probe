---
title: "Tool: probe project (graph projection)"
last-updated: 2026-09-30
status: draft
---

# probe project (graph projection)

**Directory**: `baif/probe/`
**Role**: Extract a focused subgraph from an atom file using cross-language mapping seeds.

## What this tool does

`probe project` takes a Schema 3.0 atom file and a [mappings file](../engineering/schema.md#mappings-file-format), uses all mapping endpoints (`from` + `to` code-names) as seeds, then expands via BFS in both directions with separate depth controls. The output is a trimmed atom file containing only the projected subgraph.

This is the server-side complement to probegraph's client-side source/sink filtering. It produces reusable, shareable JSON artifacts suitable for CI pipelines, demos, or focused analysis.

See [architecture.md](../engineering/architecture.md) for how this fits into the data flow.

## Key source files

| File | Purpose |
|------|---------|
| `src/commands/project.rs` | `project_atoms()` (pure function), `cmd_project()` (CLI handler), `ProjectStats` |
| `src/main.rs` | CLI: `probe project <input> --mappings <file> [--forward-depth N] [--reverse-depth N] [-o output] [--emit-focus]` |

## Algorithm

### Step 1: Load and validate

1. Load atom file via `load_validated_atom_file()` (ReadOnly scope) — accepts single-tool, `probe/merged-atoms`, and already-projected `probe/projected-atoms` envelopes, and reports whether the input is a projection (in either format) so Step 2 knows to skip enrichment
2. Run the **version-gate component** of the shared authority validator on the input ([ADR-006](../decisions/006-correspondence-records.md)): project re-stamps its output `tool.name: "probe"` at the current version, which would otherwise conceal a pre-contract origin from any later check
3. Load mappings file via `load_mappings()` — validates `probe/mappings` schema
4. Build seed set: all `from` and `to` endpoints, normalized ([P8](../engineering/properties.md#p8-code-name-normalization)), that exist in the (normalized) atom data (missing keys logged, not errored)

### Step 2: Prepare the carrier (enrichment on authoritative inputs only)

On an authoritative (non-projected) input, apply `prepare = enrich ∘ normalize`: normalize the map (P8), then **recompute enrichment on the full input graph** ([P23](../engineering/properties.md#p23-transitive-verification)) *before* any trimming — otherwise a stale label from a producer's embedded old enrichment would be frozen into a depth-limited view that the projection rejection rule then makes unrepairable. Labels are never recomputed from the trimmed view. A post-normalization collision between distinct real atoms rejects the input fail-closed, exactly as `probe enrich` does ([P8](../engineering/properties.md#p8-code-name-normalization)) — silently selected evidence would be frozen into the view. An already-projected input skips the enrichment half: it stays readable and keeps its labels untouched, but is still normalized (with the same collision rejection) so seed matching runs over normalized keys (P8). Every input, projected or not, is first checked for out-of-enum `status-origin` values and rejected fail-closed, as `probe enrich` does ([ADR-006](../decisions/006-correspondence-records.md)).

### Step 3: Build reverse adjacency index

Iterate all atoms to build a "who depends on me?" map: `BTreeMap<String, BTreeSet<String>>`. Only built when `--reverse-depth > 0`.

### Step 4: BFS expansion

- **Forward** (callee direction): from seeds, follow `atom.dependencies` up to `--forward-depth`
- **Backward** (caller direction): from seeds, follow the reverse index up to `--reverse-depth`
- Union forward + backward + seeds into the included set
- Selection is **dependency-only**: correspondence records (`maps-to`/`mapped-from`) are never traversed. (The pre-ADR-006 edge injection made mapped counterparts reachable; that reachability came from fabricated edges and is not reproduced.)

### Step 5: Filter and trim

- Keep only atoms whose code-name is in the included set
- **Trim dependencies**: remove references to atoms outside the projection (no dangling refs)
- **Trim categorized extension arrays** (`requires-dependencies`, `type-dependencies`, …) with the same filter — [P15](../engineering/properties.md#p15-dependency-completeness) decomposition is preserved
- Count trimmed deps for metadata

### Step 6: Write output

- Writes the **`probe/projected-atoms`** schema ([ADR-006](../decisions/006-correspondence-records.md)): a projection is a view; `probe merge` and `probe enrich` reject it (in this and the legacy `probe/merged-atoms`+`projection` form), while read-only consumers (summary, probegraph) accept it
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

# Load directly in probegraph
# or use ?focus=focused_focus.json with the full merged graph
```

## Properties

- **Envelope completeness** ([P1](../engineering/properties.md#p1-envelope-completeness)): output is a valid `probe/projected-atoms` Schema 3.x envelope with all required fields
- **Provenance preserved** ([P9](../engineering/properties.md#p9-provenance-is-preserved)): input `inputs` (merged) or `source` (single-tool, wrapped) carried through to output
- **Extensions preserved** ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)): atoms are cloned, so language-specific extension fields survive projection; categorized dependency arrays are trimmed consistently with `dependencies` ([P15](../engineering/properties.md#p15-dependency-completeness))
- **Labels describe the original graph** ([P23](../engineering/properties.md#p23-transitive-verification)): enrichment is recomputed on the full authoritative input before trimming, never from the trimmed view; already-projected inputs keep their labels untouched
- **Deterministic** ([P14](../engineering/properties.md#p14-deterministic-output)): BFS over BTreeMap/BTreeSet keys produces identical output for identical input
- Seeds that don't exist in atom data are silently skipped (logged to stderr); seed matching runs over normalized names ([P8](../engineering/properties.md#p8-code-name-normalization))
- Stubs in the seed set are included (they may represent API boundaries)
- The `projection` metadata block is defined in `schemas/atom-envelope.schema.json` as an optional field on the projected envelope
- **Input restriction**: only atoms-category files are accepted (specs/proofs are rejected by `load_validated_atom_file()`); pre-contract envelopes are rejected by the version gate ([ADR-006](../decisions/006-correspondence-records.md))
- **Output restriction**: projections are views — `probe merge` and `probe enrich` reject them; regenerate from the authoritative graph instead of composing views

## Focus-set emission

When `--emit-focus` is set, writes a companion `<stem>_focus.json` compatible with probegraph's `?focus=<url>` parameter:

```json
{
  "focus_nodes": ["probe:AEADScheme.decrypt", ...],
  "metadata": {
    "description": "Projection: 10 seeds, forward-depth 3, reverse-depth 0, 67 atoms"
  }
}
```
