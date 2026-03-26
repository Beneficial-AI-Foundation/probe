---
title: "ADR-002: Schema 2.0 metadata envelope"
last-updated: 2026-03-19
status: accepted
---

# ADR-002: Schema 2.0 metadata envelope

## Context

Probe tools produce JSON files containing atoms, specs, proofs, and verification reports. In Schema 1.x, these were bare JSON dictionaries — the file format had to be inferred from filename or calling context. This caused problems as the ecosystem grew.

## Decision

Wrap every probe output file in a metadata envelope containing `schema`, `schema-version`, `tool`, `source`, `timestamp`, and `data`.

See [schema.md](../engineering/schema.md#envelope) for the full specification.

## Rationale

### Schema evolution and backward compatibility

Consumers need `schema-version` to handle format changes. Without it, they must guess the format by probing field names. Every major interchange format (SARIF, CycloneDX, SCIP, SPDX) versions explicitly.

### Type discrimination

`schema: "probe-lean/atoms"` vs `"probe-lean/specs"` tells the consumer what the file contains without relying on filename. Files get renamed, moved, or passed through APIs where filenames are lost.

### Debugging and provenance

The envelope answers: Which tool produced this? (`tool.name`, `tool.version`). When? (`timestamp`). Is it stale? (`source.commit` vs `git rev-parse HEAD`).

### Multi-tool coordination

When orchestrating multiple probe tools, the envelope verifies outputs are compatible (same `schema-version`) and correspond to the expected source (`source.package`, `source.commit`).

### Merging requires provenance

When combining atoms from different tools/languages/repos, the envelope identifies the origin of each input file. The merged output carries an `inputs` array preserving full provenance chain.

### Self-describing files for viewers

A web viewer can accept any probe output file and render appropriately based on `schema` and `source.language`.

## What the envelope does NOT do

- **Replace per-atom metadata** — `language` and code-name URI on each atom remain canonical
- **Provide JSON Schema validation** — no stable public domain yet for `$schema` URIs
- **Include content hashes** — staleness uses `source.commit` + `timestamp`, not hashes

## Consequences

- All tools must produce envelopes (no bare dictionaries from primary commands)
- Consumers must read the envelope before accessing `data`
- Merged files use `inputs` array instead of `source` — consumers must handle both shapes
- File size increases slightly (~200 bytes per file for envelope overhead)
- probe-verus `merge-atoms` (legacy, bare dictionary) is retained for backward compatibility but superseded by `probe merge`

## Migration

Schema 1.x → 2.0 was a coordinated rollout:
1. Implement envelope in probe-lean
2. Implement envelope in probe-verus
3. Update verilib-cli (sole consumer at the time)

No migration period needed — sole consumer was under our control.
