# Probe Merge Algorithm

Version: draft
Date: 2026-09-28
Parent document: [SCHEMA.md](SCHEMA.md)
Normative spec: [kb/engineering/schema.md](../kb/engineering/schema.md), [kb/engineering/properties.md](../kb/engineering/properties.md), [ADR-006](../kb/decisions/006-correspondence-records.md)

The algorithm for `probe merge`, which combines data files from multiple `probe-*` tools
into one. It handles three categories -- **atoms**, **specs**, and **proofs** -- each
auto-detected from the `schema` field; all inputs must be the same category.

## Overview

`probe merge` takes two or more Schema 3.0 files and produces one output file. The output
schema depends on the input category:

| Input category | Output schema |
|----------------|---------------|
| atoms / enriched-atoms | `probe/merged-atoms` |
| specs | `probe/merged-specs` |
| proofs | `probe/merged-proofs` |

```
probe merge <file1> <file2> [file3...] -o merged.json
```

## Algorithm

### Phase 1: Load and Validate Authority

For each input file:

1. Parse the JSON and extract the envelope fields.
2. Validate that `schema-version` has a compatible major version (currently `3`).
   Reject files with an incompatible major version with a clear error.
3. **Reject projections**: any input carrying the `probe/projected-atoms` schema,
   or the legacy form (`probe/merged-atoms` plus a `projection` envelope field),
   is an error — projections are views; re-enriching or merging them would
   launder truncated graphs into stronger labels (ADR-006).
4. **Reject pre-contract envelopes**: atoms envelopes whose `tool.name` is in
   the per-producer version-gate table with `tool.version` below that
   producer's contract threshold are errors. The `probe` entry is an interval
   (`threshold ≤ version < 1.0.0`) and additionally rejects
   `tool.command: "merge-atoms"` at any version.
5. Detect the **schema category** from the `schema` field:
   - **Atoms**: `*/atoms`, `*/enriched-atoms`, `*/extract`, `probe/merged-atoms`
   - **Specs**: `*/specs`, `probe/merged-specs`
   - **Proofs**: `*/proofs`, `probe/merged-proofs`
   Reject unrecognized schemas.
6. Validate that all inputs belong to the **same category**. Error on mismatch.
7. Extract the `data` dictionary.
8. Record provenance **structurally**: an envelope with an `inputs` array is
   composed — flatten its entries; an envelope with a `source` is single-tool —
   wrap it. Deduplicate identical entries (provenance is a source inventory).

Steps 3–4 are one shared authority validator, invoked at every envelope
boundary (`cmd_merge`, `merge_atom_files`, `cmd_enrich`, the raw staging
primitive). The bare-map library API cannot check authority; callers own the
envelope boundary.

### Phase 2: Normalize (per input, before conflict resolution)

For each entry in each loaded data dictionary:

1. **Normalize code-name keys.** Strip trailing `.` characters from code-names (a
   legacy artifact from verus-analyzer).

For **atom** files only:

2. Apply the same normalization to all entries in the atom's `dependencies` array,
   every code-name-bearing extension array (`requires-dependencies`,
   `ensures-dependencies`, `body-dependencies`, `type-dependencies`,
   `term-dependencies`), `code-name` fields in `dependencies-with-locations`,
   and `maps-to`/`mapped-from` record targets. Mapping-file endpoints are
   normalized by the same rule before lookup.
3. **Handle intra-file duplicates.** If normalization causes two keys within the same
   file to collide, apply stub-vs-real resolution (see Phase 3 rules). If both are real
   atoms, keep the first and warn (counted in `conflicts`). Correspondence records
   are unioned across the collision (P27).

Normalization runs per input, **before** conflict resolution — the ordering is
semantic: it decides which atom wins a collision before evidence from other
inputs is considered.

### Phase 3: Merge

The first file is the **base**. For each subsequent file, iterate over its entries and
apply category-specific conflict resolution rules.

#### Atoms: first-wins with stub replacement

| Base entry | Incoming entry | Action |
|------------|----------------|--------|
| stub | real | **Replace**: incoming wins |
| real | real | **Conflict**: keep base, emit warning with both `code-path` values |
| stub | stub | Keep base (no information gain) |
| real | stub | Keep base (no information loss) |
| (absent) | any | **Add**: insert incoming entry |

An atom is a **stub** when all three conditions hold:

- `code-path` is `""`
- `code-text.lines-start` is `0`
- `code-text.lines-end` is `0`

#### Specs and Proofs: last-wins

| Base entry | Incoming entry | Action |
|------------|----------------|--------|
| any | any (same key) | **Replace**: incoming wins |
| (absent) | any | **Add**: insert incoming entry |

Specs and proofs have no stub concept. When the same code-name appears in multiple
inputs, the **last** one wins. This is appropriate because re-running `specify` or
`verify` should override stale results.

On every equal-key atom resolution (stub replacement, real-vs-real, stub-vs-stub),
the surviving atom's `maps-to`/`mapped-from` correspondence arrays are the set
union of both sides' (P27).

### Phase 3b: Attach Correspondence Records (optional, `--mappings`)

For each mapping entry `from → to` (endpoints normalized): attach a `maps-to`
record `{target, confidence, method?}` to the `from` atom if present in the
merged map, and a mirror `mapped-from` record to the `to` atom if present.
`dependencies` is never modified; a dangling target is warned about, never
skipped; records are sorted and deduplicated by `(target, confidence, method)`.
See [mappings-spec.md](mappings-spec.md) and ADR-006.

### Phase 3c: Re-enrich (atoms only)

After all inputs are combined, run enrichment recomputation
([P23](../kb/engineering/properties.md#p23-transitive-verification)) over the
merged atom map — one reverse BFS from the unified seed set (explicit
`failed`/`unverified` atoms plus every `status-origin`-bearing atom), labels
set fresh. Stub resolution can invalidate labels computed at extract time, so
merged labels are recomputed, never inherited. A raw staging primitive for
multi-step pipelines defers this pass (but not authority validation); its
output carries potentially stale derived statuses.

### Phase 4: Write Output

1. Construct the output envelope:

```json
{
  "schema": "probe/merged-atoms",
  "schema-version": "3.0",
  "tool": {
    "name": "probe",
    "version": "<probe version>",
    "command": "merge"
  },
  "inputs": [
    {
      "schema": "<input1 schema>",
      "source": { "<input1 source object>" : "..." }
    },
    {
      "schema": "<input2 schema>",
      "source": { "<input2 source object>" : "..." }
    }
  ],
  "timestamp": "<ISO 8601 timestamp>",
  "data": { "<merged entries>" : "..." }
}
```

2. The `inputs` array replaces `source` in the merged envelope. Each entry records the
   `schema` and `source` from one input file, preserving full provenance. When a
   previously merged file is used as input, its `inputs` entries are flattened into the
   new output. The `source` field is omitted from the top-level envelope since a merged
   file spans multiple projects.

3. Serialize the merged data dictionary as the `data` field. Keys must be sorted for
   deterministic output.

4. Write the JSON to the output file.

## Statistics

After merging, the tool reports:

| Metric | Atoms | Specs/Proofs | Description |
|--------|-------|--------------|-------------|
| Total entries | Yes | Yes | Number of entries in the merged output |
| Stubs replaced | Yes | -- | Stubs in base that were replaced by real atoms |
| Stubs remaining | Yes | -- | Stubs still present after all merges |
| New entries added | Yes | Yes | New entries (not in base) added from subsequent files |
| Keys normalized | Yes | Yes | Code-names that had trailing `.` stripped |
| Conflicts | Yes | Yes | Collisions: for atoms, real-vs-real (base kept) and post-normalization intra-file collisions; for specs/proofs, overrides (incoming kept) |
| Records attached | Yes | -- | `maps-to`/`mapped-from` records attached (if `--mappings`), plus dangling-target warnings |

## Cross-Language Considerations

When merging atoms from different languages:

- Atoms with different `language` values coexist in the same `data` dictionary. The
  per-atom `language` field tells consumers how to interpret `kind` values and
  code-name formats.

- Same-language stub resolution works identically regardless of language: matching is
  purely by code-name string equality.

- **Cross-language correspondence** requires a cross-language mapping file. If a
  `--mappings <file>` argument is provided, the merge tool attaches
  `maps-to`/`mapped-from` correspondence records to the mapped atoms (Phase 3b) —
  it never adds cross-language dependency edges, and it never resolves a stub in
  one language against an atom in another (the two are distinct atoms linked by a
  record). Cross-language resolution is a derived consumer view over the records.
  The mappings file format is specified in [mappings-spec.md](mappings-spec.md).

## Relationship to probe-verus merge-atoms

probe-verus's `merge-atoms` is a legacy, independent merge implementation slated
for retirement or reimplementation as a caller of this algorithm (ADR-006). Its
outputs masquerade as the hub (`tool.name: "probe"` at probe-verus's own
version) and silently drop verification statuses, `status-origin` markers, and
correspondence records. The hub rejects them at any version via the `probe` gate
interval and the `tool.command: "merge-atoms"` check (Phase 1, step 4).
