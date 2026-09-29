# Probe Envelope Rationale

Version: final (rationale record)
Date: 2026-09-29

## Context

Every `probe-*` tool wraps its JSON output (dependency graphs,
specifications, verification results) in a metadata envelope. The
envelope shipped with Schema 2.0 and is normatively specified in
[kb/engineering/schema.md § Envelope](../kb/engineering/schema.md#envelope)
(fields, merged variant, registered schema values, package versioning).
This document records only the *rationale*: why the envelope exists and
what it deliberately does not do.

## Why an Envelope Is Needed

### 1. Schema evolution and backward compatibility

When the format changes, consumers need to know which version they are reading. Without
`schema-version`, every consumer must guess by probing for field names ("does this file
have `kind` or `mode`?"). Every major interchange format (SARIF, CycloneDX, SCIP, SPDX)
versions its schema so old consumers can reject incompatible files with a clear error
instead of misinterpreting them.

### 2. Type discrimination

`schema: "probe-lean/extract"` vs. `"probe-lean/viewify"` tells a consumer what kind of
file it is reading without relying on the filename. This matters because:

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

Recommended workflow: commit before running an extraction, else regenerate.
`source.commit` makes this possible -- `verilib-cli` compares it against the repo's HEAD
to tell whether the output is current.

### 5. Multi-tool coordination

When `verilib-cli` orchestrates probe-lean and probe-verus, the envelope lets it check that
outputs are compatible (same `schema-version`) and correspond to the expected source
(`source.package`, `source.commit`) rather than trusting filenames.

### 6. Merging

When combining atoms from different languages, tools, or repositories, the envelope
identifies the origin of each file. The `source` fields tell the merge tool what it is
combining; the `schema-version` ensures format compatibility; the `schema` field confirms
the file type. Per-atom fields (`language`, code-name URIs) handle identity within the
merged result, but the envelope handles identity of the *inputs* to the merge -- carried
forward as the
[merged envelope's `inputs` provenance](../kb/engineering/schema.md#merged-envelope-variant).

### 7. Self-describing files for viewers

A web viewer (probegraph) that receives a bare JSON blob must be told externally what it
represents. With an envelope, a viewer can accept any probe output file and render it
appropriately based on `schema` and `source.language`.

## What the Envelope Should NOT Do

- **Replace per-atom metadata.** The `language` and code-name URI on each atom remain the
  canonical per-atom identifiers. The envelope describes the *file*, not the individual
  atoms.
- **Embed a `$schema` URI (yet).** A `$schema` URI pointing to a hosted JSON Schema file
  is standard practice (SARIF, CycloneDX), but requires a stable public hosting domain.
  A machine-readable JSON Schema does exist in-repo
  ([schemas/atom-envelope.schema.json](../schemas/atom-envelope.schema.json), see
  [schema-validation.md](schema-validation.md)); embedding its URI in the envelope is
  deferred until it has a stable public home.
- **Include content hashes.** Staleness detection uses `source.commit` and `timestamp`,
  not content hashes. Hashing is a build-system concern.

## Field reference

The envelope's field-by-field contract, the merged variant, the registered
`schema` values, and per-language package versioning are all specified in
[kb/engineering/schema.md](../kb/engineering/schema.md) -- see
[§ Envelope fields](../kb/engineering/schema.md#envelope-fields),
[§ Merged envelope variant](../kb/engineering/schema.md#merged-envelope-variant),
[§ Registered schema values](../kb/engineering/schema.md#registered-schema-values), and
[§ Package versioning by language](../kb/engineering/schema.md#package-versioning-by-language).
This document intentionally restates none of it.
