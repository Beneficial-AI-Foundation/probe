# Probe Schema 2.0: Envelope Rationale

Version: draft
Date: 2026-03-05

## Context

The `probe-*` tools (probe-verus, probe-lean, probe-latex, etc.) analyze source code and
produce JSON files describing dependency graphs, specifications, and verification results.
Today these files are **bare JSON dictionaries** with no metadata:

```json
{
  "probe:MyModule.myFunction": {
    "display-name": "myFunction",
    "dependencies": ["probe:MyModule.helper"],
    ...
  }
}
```

This document explains why we need to wrap these dictionaries in a metadata envelope,
what the envelope should contain, and how this relates to industry standards.

## Why an Envelope Is Needed

### 1. Schema evolution and backward compatibility

When the atom format changes (new required fields, renamed fields, changed semantics),
consumers need to know which version of the format they are reading. Without a
`schema-version` field, every consumer must guess the format by probing for field names --
"does this file have a `kind` field or a `mode` field?"

This is a problem probe *will* hit as the schema evolves. Every major interchange format
(SARIF, CycloneDX, SCIP, SPDX) versions its schema explicitly. The version field is what
allows old consumers to reject incompatible files with a clear error instead of silently
misinterpreting them.

### 2. Type discrimination

`schema: "probe-lean/atoms"` vs. `"probe-lean/specs"` tells a consumer what kind of file
it is reading without relying on the filename. This matters because:

- Files get renamed, moved, or passed through APIs where the filename is lost.
- `verilib-cli` processes multiple file types and needs to dispatch on content, not path.
- A generic viewer or debugging tool can open any `.json` from `.verilib/` and know what
  it is.

### 3. Debugging and provenance

When something looks wrong in a JSON file, the envelope answers:

- **Which tool produced this?** `tool.name` and `tool.version`.
- **When was this produced?** `timestamp`.
- **Is this stale?** `source.commit` can be compared against the current `git rev-parse HEAD`.

Without the envelope, diagnosing problems requires grepping git history or guessing.

### 4. Staleness detection

The recommended workflow is: commit before running atomize/specify/verify; if not
committed, regenerate everything. The `source.commit` field in the envelope is what makes
this check possible -- `verilib-cli` can compare the envelope's commit against the repo's
HEAD and know whether the probe output is current.

### 5. Multi-tool coordination

When `verilib-cli` orchestrates probe-lean and probe-verus, the envelope lets it verify
that the outputs are compatible (same `schema-version`) and correspond to the expected
source (`source.package`, `source.commit`). Without this, verilib-cli has to trust that the
files are what the filenames claim they are.

### 6. Merging

When combining atoms from different languages, tools, or repositories, the envelope
identifies the origin of each file. The `source` fields tell the merge tool what it is
combining; the `schema-version` ensures format compatibility; the `schema` field confirms
the file type. Per-atom fields (`language`, code-name URIs) handle identity within the
merged result, but the envelope handles identity of the *inputs* to the merge.

### 7. Self-describing files for viewers

The web viewer (scip-callgraph) currently receives a bare JSON blob and must be told
externally what it represents. With an envelope, a viewer can accept any probe output file
and render it appropriately based on `schema` and `source.language`.

## What the Envelope Should NOT Do

- **Replace per-atom metadata.** The `language` and code-name URI on each atom remain the
  canonical per-atom identifiers. The envelope describes the *file*, not the individual
  atoms.
- **Provide machine-validatable JSON Schema (yet).** A `$schema` URI pointing to a hosted
  JSON Schema file is standard practice (SARIF, CycloneDX), but requires a stable public
  domain and a mature spec. Deferred until probe has both.
- **Include content hashes.** Staleness detection uses `source.commit` and `timestamp`,
  not content hashes. Hashing is a build-system concern.

## Proposed Envelope

```json
{
  "schema": "probe-lean/atoms",
  "schema-version": "2.0",
  "tool": {
    "name": "probe-lean",
    "version": "1.0.0",
    "command": "atomize"
  },
  "source": {
    "repo": "https://github.com/org/project",
    "commit": "abc123def456...",
    "language": "lean",
    "package": "MyProject",
    "package-version": "0.1.0"
  },
  "timestamp": "2026-03-05T14:30:00Z",
  "data": { ... }
}
```

### Field Reference

#### `schema` (string, required)

Identifies the producing tool and data type. Format: `<tool>/<type>`.

Known values:

- `probe-verus/atoms` -- Rust/Verus call graph atoms
- `probe-verus/specs` -- Rust/Verus function specifications
- `probe-verus/proofs` -- Rust/Verus verification results
- `probe-verus/stubs` -- Rust/Verus stubs (output of the `stubify` command)
- `probe-verus/verification-report` -- Rust/Verus verification report (output of `verify` without atoms enrichment)
- `probe-lean/atoms` -- Lean call graph atoms
- `probe-lean/specs` -- Lean function specifications
- `probe-lean/proofs` -- Lean verification results
- `probe-lean/stubs` -- Lean stubs (output of the `stubify` command)
- `probe-lean/enriched-atoms` -- Lean enriched atoms (atoms augmented with specs/proofs)
- `probe/merged-atoms` -- merged atoms from multiple tools
- `probe/merged-specs` -- merged specs from multiple tools
- `probe/merged-proofs` -- merged proofs from multiple tools

New tools register their schema values by adding them to this list.

#### `schema-version` (string, required)

The version of the interchange specification that the file conforms to. Format:
`<major>.<minor>`. A change to required fields or their semantics increments the major
version. Adding optional fields increments the minor version.

#### `tool` (object, required)

Structured metadata about the tool that produced the file.

- `name` (string, required): tool identifier (e.g., `"probe-lean"`, `"probe-verus"`)
- `version` (string, required): semver version of the tool
- `command` (string, required): the command that produced this file (e.g., `"atomize"`,
  `"specify"`, `"verify"`, `"pipeline"`, `"stubify"`)

The `command` field records which invocation produced the file. This is distinct from
`schema` (which describes the data type): `schema` says *what* the data is,
`tool.command` says *how* it was generated. These are usually 1:1 (`atomize` produces
atoms), but not always (`pipeline` produces enriched-atoms).

Using a structured object (rather than a bare `tool-version` string) allows future
extension (e.g., `tool.commit` for development builds) without a breaking change.

#### `source` (object, required)

Structured metadata about what was analyzed.

- `repo` (string, required): git remote URL of the analyzed project
- `commit` (string, required): full git commit hash at analysis time
- `language` (string, required): source language (`"rust"`, `"lean"`, `"latex"`)
- `package` (string, required): package/crate/project name
- `package-version` (string, required): version identifier (see
  [Package Versioning](#package-versioning))

Using a structured object (rather than a bare `source` string) makes each piece
independently queryable and avoids parsing conventions.

#### `timestamp` (string, required)

ISO 8601 timestamp of when the analysis was run. Combined with `source.commit`, this
enables staleness detection: the commit identifies *what* was analyzed, the timestamp
identifies *when*.

#### `data` (object, required)

The payload. Structure depends on `schema` and is defined by the interchange specification
and per-language schema documents.

### Package Versioning

Package versioning differs by ecosystem:

- **Rust (Cargo):** Every crate has a mandatory semver version. Use it directly
  (e.g., `"4.1.3"`).
- **Lean (Lake):** The `version` field in lakefile is optional. Use it if present;
  otherwise fall back to the 7-character short git commit hash (e.g., `"a1b2c3d"`).
- **LaTeX:** No package versioning convention. Use the short git commit hash.

This means `package-version` is always present and non-empty, but its format varies.
Consumers should treat it as an opaque identifier, not assume semver.

## Folder Structure

The `.verilib/` folder lives **inside the analyzed target project**. For multi-repo
scenarios, one project is chosen as the "home base" and its `.verilib/` aggregates probe
results from all relevant repos.

### Current layout (Schema 1.x)

```
target-project/.verilib/
  config.json
  atoms.json
  specs.json
  proofs.json
  stubs.json
  graph.json
  structure/
```

### Proposed layout (Schema 2.0)

```
target-project/.verilib/
  config.json                            # User/project configuration (not part of schema)
  probes/
    lean_MyProject_0.1.0.json            # probe-lean output (self-describing via envelope)
    verus_dalek_4.1.3.json               # probe-verus output from related repo
  views/
    molecule_all.json                    # Filtered projection (replaces stubs.json)
    package_all.json
  translations/
    verus_dalek__lean_dalek.json         # Cross-language atom mappings
```

Each file in `probes/` is self-describing via its envelope. The filename convention
`<language>_<package>_<version>.json` is for human convenience; consumers should read
the envelope, not parse the filename.

### What lives where

- **`probes/`**: Raw output from probe tools. One file per analysis run. Written by
  probe-lean, probe-verus, etc.
- **`views/`**: Derived projections computed from probes. Written by verilib-cli or
  downstream tools. Replaces the current `stubs.json`.
- **`translations/`**: Cross-language atom mappings. Written by verilib-cli based on
  transpiler metadata (e.g., Aeneas Rust-to-Lean mappings).
- **`config.json`**: User/project configuration. Not part of the schema spec; it is a
  tool configuration concern.

## Merged Envelope Variant

When `probe merge` produces a merged file, the envelope differs from single-tool output:

- `schema` is set based on the input category: `"probe/merged-atoms"`,
  `"probe/merged-specs"`, or `"probe/merged-proofs"`.
- `source` is **omitted** (a merged file spans multiple projects).
- `inputs` (array, required) replaces `source`. Each entry records the `schema` and
  `source` object from one input file, preserving full provenance. When a previously
  merged file is used as input, its `inputs` are flattened into the new output so
  provenance is carried forward across recursive merges.

```json
{
  "schema": "probe/merged-atoms",
  "schema-version": "2.0",
  "tool": {
    "name": "probe",
    "version": "0.1.0",
    "command": "merge"
  },
  "inputs": [
    {
      "schema": "probe-verus/atoms",
      "source": {
        "repo": "https://github.com/ArtificialBreeze/curve25519-dalek",
        "commit": "a1b2c3d4...",
        "language": "rust",
        "package": "curve25519-dalek",
        "package-version": "4.1.3"
      }
    },
    {
      "schema": "probe-lean/atoms",
      "source": {
        "repo": "https://github.com/ArtificialBreeze/curve25519-dalek-lean-verify",
        "commit": "f6e5d4c3...",
        "language": "lean",
        "package": "Curve25519DalekLeanVerify",
        "package-version": "0.1.0"
      }
    }
  ],
  "timestamp": "2026-03-05T15:00:00Z",
  "data": { }
}
```

The full merge algorithm is specified in [merge-algorithm.md](merge-algorithm.md).

## Rollout

The only consumer of probe output is currently `verilib-cli`, which we control. There is
no need for a migration period or backward compatibility flags. The rollout is:

1. Implement envelope in probe-lean.
2. Implement envelope in probe-verus.
3. Update verilib-cli to read the new format.

All three changes can be coordinated in lockstep.

## Future Work

- **JSON Schema validation files**: Publish `.schema.json` files at stable URLs; add
  `$schema` URI to the envelope.
- **Provenance tracking for views**: When views combine data from multiple probes, record
  which probe files were inputs (`sources` field in view envelope).
