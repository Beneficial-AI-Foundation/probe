# Probe Merge Algorithm

Version: draft
Date: 2026-03-06
Parent document: [interchange-spec.md](interchange-spec.md)

This document specifies the algorithm for `probe merge`, which combines data files from
multiple `probe-*` tools into a single merged file. The merge tool handles three
categories of data: **atoms**, **specs**, and **proofs**. All inputs must be the same
category; the category is auto-detected from the `schema` field.

## Overview

`probe merge` takes two or more Schema 2.0 files and produces a single output file. The
output schema depends on the input category:

| Input category | Output schema |
|----------------|---------------|
| atoms / enriched-atoms | `probe/merged-atoms` |
| specs | `probe/merged-specs` |
| proofs | `probe/merged-proofs` |

```
probe merge <file1> <file2> [file3...] -o merged.json
```

## Algorithm

### Phase 1: Load and Validate

For each input file:

1. Parse the JSON and extract the envelope fields.
2. Validate that `schema-version` has a compatible major version (currently `2`).
   Reject files with an incompatible major version with a clear error.
3. Detect the **schema category** from the `schema` field:
   - **Atoms**: `*/atoms`, `*/enriched-atoms`, `probe/merged-atoms`
   - **Specs**: `*/specs`, `probe/merged-specs`
   - **Proofs**: `*/proofs`, `probe/merged-proofs`
   Reject unrecognized schemas.
4. Validate that all inputs belong to the **same category**. Error on mismatch.
5. Extract the `data` dictionary.
6. Record provenance: for single-tool files, capture the `source` and `schema`; for
   previously merged files, flatten the `inputs` array so provenance is carried forward
   across recursive merges.

### Phase 2: Normalize

For each entry in each loaded data dictionary:

1. **Normalize code-name keys.** Strip trailing `.` characters from code-names (a
   legacy artifact from verus-analyzer).

For **atom** files only:

2. Apply the same normalization to all entries in the atom's `dependencies` array and
   any `code-name` fields in `dependencies-with-locations`.
3. **Handle intra-file duplicates.** If normalization causes two keys within the same
   file to collide, apply stub-vs-real resolution (see Phase 3 rules). If both are real
   atoms, keep the first and warn.

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

### Phase 4: Write Output

1. Construct the output envelope:

```json
{
  "schema": "probe/merged-atoms",
  "schema-version": "2.0",
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
| Conflicts | Yes | Yes | Collisions: for atoms, real-vs-real (base kept); for specs/proofs, overrides (incoming kept) |

## Cross-Language Considerations

When merging atoms from different languages:

- Atoms with different `language` values coexist in the same `data` dictionary. The
  per-atom `language` field tells consumers how to interpret `kind` values and
  code-name formats.

- Same-language stub resolution works identically regardless of language: matching is
  purely by code-name string equality.

- **Cross-language stub resolution** (e.g., resolving a Lean stub that corresponds to a
  Rust atom via Aeneas transpilation) requires a translation mapping file. If a
  `--translations <file>` argument is provided, the merge tool loads the mapping and
  uses it to add cross-language dependency edges between atoms that represent the same
  logical function. The translations file format is specified in
  [translations-spec.md](translations-spec.md).

## Relationship to probe-verus merge-atoms

This algorithm generalizes probe-verus's `merge-atoms` command. The key differences:

| Aspect | probe-verus merge-atoms | probe merge |
|--------|------------------------|-------------|
| Input format | Bare JSON dictionaries (no envelope) | Schema 2.0 enveloped files |
| Output format | Bare JSON dictionary | Schema 2.0 envelope with `probe/merged-*` |
| Data categories | Atoms only | Atoms, specs, and proofs |
| Languages | Rust only | Any language |
| Provenance | None | `inputs` array in output envelope |
| Cross-language | N/A | Same-language by default; translations planned |

The atom merge rules (stub resolution, conflict handling, normalization) are identical.
