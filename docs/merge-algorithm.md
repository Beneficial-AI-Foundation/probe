# Probe Merge Algorithm

Version: draft
Date: 2026-03-05
Parent document: [interchange-spec.md](interchange-spec.md)

This document specifies the algorithm for `probe merge-atoms`, which combines atom files
from multiple `probe-*` tools into a single merged atom file. The merge rules are
summarized in the [Merged Atoms](interchange-spec.md#merged-atoms) section of the
interchange spec; this document provides the full algorithm with normalization, conflict
resolution, and envelope handling.

## Overview

`probe merge-atoms` takes two or more atom files (each with a Schema 2.0 envelope) and
produces a single output file with `"schema": "probe/merged-atoms"`. The input files may
come from different tools (probe-verus, probe-lean, etc.) and contain atoms in different
languages.

```
probe merge-atoms <file1> <file2> [file3...] -o merged_atoms.json
```

## Algorithm

### Phase 1: Load and Validate

For each input file:

1. Parse the JSON and extract the envelope fields.
2. Validate that `schema-version` has a compatible major version (currently `2`).
   Reject files with an incompatible major version with a clear error.
3. Validate that `schema` ends in `/atoms` or `/enriched-atoms`. Reject other schemas
   (e.g., `/specs`, `/proofs`) -- merging is only defined for atom files.
4. Extract the `data` dictionary.
5. Record the envelope's `source` and `schema` for provenance in the output.

### Phase 2: Normalize

For each atom in each loaded data dictionary:

1. **Normalize code-name keys.** Strip trailing `.` characters from code-names (a
   legacy artifact from verus-analyzer). Apply the same normalization to all entries
   in the atom's `dependencies` array and any `code-name` fields in
   `dependencies-with-locations`.

2. **Handle intra-file duplicates.** If normalization causes two keys within the same
   file to collide, apply stub-vs-real resolution (see Phase 3 rules). If both are real
   atoms, keep the first and warn.

### Phase 3: Merge

The first file is the **base**. For each subsequent file, iterate over its atoms and
apply these rules:

| Base atom | Incoming atom | Action |
|-----------|---------------|--------|
| stub | real | **Replace**: incoming wins |
| real | real | **Conflict**: keep base, emit warning with both `code-path` values |
| stub | stub | Keep base (no information gain) |
| real | stub | Keep base (no information loss) |
| (absent) | any | **Add**: insert incoming atom |

An atom is a **stub** when all three conditions hold:

- `code-path` is `""`
- `code-text.lines-start` is `0`
- `code-text.lines-end` is `0`

### Phase 4: Write Output

1. Construct the output envelope:

```json
{
  "schema": "probe/merged-atoms",
  "schema-version": "2.0",
  "tool": {
    "name": "probe",
    "version": "<probe version>",
    "command": "merge-atoms"
  },
  "inputs": [
    {
      "schema": "<input1 schema>",
      "source": { <input1 source object> }
    },
    {
      "schema": "<input2 schema>",
      "source": { <input2 source object> }
    }
  ],
  "timestamp": "<ISO 8601 timestamp>",
  "data": { <merged atoms> }
}
```

2. The `inputs` array replaces `source` in the merged envelope. Each entry records the
   `schema` and `source` from one input file, preserving full provenance. The `source`
   field is omitted from the top-level envelope since a merged file spans multiple
   projects.

3. Serialize the merged atoms dictionary as the `data` field. Keys must be sorted for
   deterministic output.

4. Write the JSON to the output file.

## Statistics

After merging, the tool reports:

| Metric | Description |
|--------|-------------|
| Total atoms | Number of atoms in the merged output |
| Stubs replaced | Stubs in base that were replaced by real atoms |
| Stubs remaining | Stubs still present after all merges |
| Atoms added | New atoms (not in base) added from subsequent files |
| Keys normalized | Code-names that had trailing `.` stripped |
| Conflicts | Real-vs-real collisions (base version kept) |

## Cross-Language Considerations

When merging atoms from different languages:

- Atoms with different `language` values coexist in the same `data` dictionary. The
  per-atom `language` field tells consumers how to interpret `kind` values and
  code-name formats.

- Same-language stub resolution works identically regardless of language: matching is
  purely by code-name string equality.

- **Cross-language stub resolution** (e.g., resolving a Lean stub that corresponds to a
  Rust atom via Aeneas transpilation) requires a translation mapping file. This is an
  optional feature: if a `--translations <file>` argument is provided, the merge tool
  loads the mapping and uses it to match code-names across languages before applying the
  standard merge rules. Translation mappings are defined in the `translations/` folder
  described in [envelope-rationale.md](envelope-rationale.md). This feature is planned
  for a future iteration.

## Relationship to probe-verus merge-atoms

This algorithm generalizes probe-verus's `merge-atoms` command. The key differences:

| Aspect | probe-verus merge-atoms | probe merge-atoms |
|--------|------------------------|-------------------|
| Input format | Bare JSON dictionaries (no envelope) | Schema 2.0 enveloped files |
| Output format | Bare JSON dictionary | Schema 2.0 envelope with `probe/merged-atoms` |
| Languages | Rust only | Any language |
| Provenance | None | `inputs` array in output envelope |
| Cross-language | N/A | Same-language by default; translations planned |

The merge rules (stub resolution, conflict handling, normalization) are identical.
