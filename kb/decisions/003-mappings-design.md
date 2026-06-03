---
title: "ADR-003: Cross-language mappings"
last-updated: 2026-06-03
status: accepted
---

# ADR-003: Cross-language mappings

## Context

When analyzing cross-language projects, a function in one language and its counterpart in another are different atoms (different code-names, languages, source locations) but represent the same logical definition. We need to connect them so the merged call graph shows cross-language dependency edges.

This applies to Aeneas-transpiled projects (Rust↔Lean translations) and to manual cross-language linking (e.g. Rust functions mapped to Lean security primitives).

## Decision

Use a separate mappings file (`probe/mappings` schema) containing bidirectional mappings between code-names. The merge operator applies these during `probe merge --mappings <file>`.

A single `from` key may map to multiple `to` targets (1-to-many), enabling scenarios like mapping one Rust function to several Lean constructs.

See [schema.md](../engineering/schema.md#mappings-file-format) for the file format and [properties.md](../engineering/properties.md#p11-mapping-generation-is-1-to-1-probe-aeneas) for invariants.

## Rationale

### Separation of concerns

Mapping generation requires domain knowledge (Aeneas name conventions, functions.json, transpilation patterns — or manual curation). The merge operator should be domain-agnostic. Separating them means:
- probe-aeneas owns mapping generation for Aeneas projects (the [functor](../engineering/glossary.md#functor))
- Manual mapping files can be authored for any cross-language scenario
- probe merge owns composition (applies mappings generically)
- Adding a new cross-language bridge (e.g. Rust↔Haskell) requires only a new mapping generator — no changes to merge

### Three-strategy matching is pragmatic (probe-aeneas)

Aeneas doesn't always produce clean name mappings. The three strategies (see [probe-aeneas.md](../tools/probe-aeneas.md#three-strategy-mapping-generation)) handle the spectrum:
1. **Rust-qualified-name** — when Charon provides exact names (best case)
2. **File + display-name** — when names match but no qualified name available
3. **File + line-overlap** — when names don't match but source locations overlap (fallback)

Priority order ensures the most confident match wins.

### 1-to-1 generation, 1-to-many application

probe-aeneas generates 1-to-1 mappings (each Rust atom maps to at most one Lean atom). However, `probe merge` accepts 1-to-many mappings: a single `from` key can map to multiple `to` targets. This supports manual mappings where one implementation corresponds to multiple formal constructs.

### Bidirectional lookup

Mappings are stored as `from`/`to` pairs but applied bidirectionally during merge. If atom A depends on code-name X, and X maps to Y, then Y is added as a dependency of A. Both directions checked because atoms in either language may depend on code-names from the other.

## Consequences

- Mapping files must be generated (or manually authored) before merge can add cross-language edges
- The `extract` command in probe-aeneas handles this automatically (generate → merge in one step)
- Standalone `probe merge` requires the user to provide `--mappings` explicitly
- Mappings are confidence-tagged, enabling consumers to filter by confidence level
- Adding a new source language requires building a mapping generator but no changes to probe merge
- 1-to-many support means a single source function can link to multiple formal targets

## Alternatives considered

### Embed mappings in atoms

Add a `mapping` field directly on atoms pointing to their counterpart. Rejected because:
- Requires each extractor to know about mappings (violates separation)
- Merge would need special handling for mapping fields
- Can't express mappings without running both extractors first

### Match by convention (same name = same definition)

Rely on naming conventions (e.g. Lean `Module.func` corresponds to Rust `module::func`). Rejected because:
- Aeneas name mangling makes this unreliable
- No way to express confidence
- Fails for generics, trait impls, overloaded names

### Single merged code-name

Give the Rust and Lean versions the same code-name. Rejected because:
- Code-names encode language-specific information (crate version, Lean namespace)
- Would break the invariant that code-names are deterministic per-tool
- Hides the fact that two distinct source definitions exist
