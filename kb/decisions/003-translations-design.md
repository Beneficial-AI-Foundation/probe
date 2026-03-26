---
title: "ADR-003: Cross-language translation mappings"
last-updated: 2026-03-19
status: accepted
---

# ADR-003: Cross-language translation mappings

## Context

When analyzing Aeneas-transpiled projects, a Rust function and its Lean translation are different atoms (different code-names, languages, source locations) but represent the same logical definition. We need to connect them so the merged call graph shows cross-language dependency edges.

## Decision

Use a separate translations file (`probe/translations` schema) containing bidirectional mappings between code-names. The merge operator applies these during `probe merge --translations <file>`.

See [schema.md](../engineering/schema.md#translations-file-format) for the file format and [properties.md](../engineering/properties.md#p11-translation-mapping-is-1-to-1) for invariants.

## Rationale

### Separation of concerns

Translation generation requires domain knowledge about Aeneas (name conventions, functions.json, transpilation patterns). The merge operator should be domain-agnostic. Separating them means:
- probe-aeneas owns translation generation (the [functor](../engineering/glossary.md#functor))
- probe merge owns composition (applies translations generically)
- Adding a new cross-language bridge (e.g. Rust↔Haskell) requires only a new translation generator — no changes to merge

### Three-strategy matching is pragmatic

Aeneas doesn't always produce clean name mappings. The three strategies (see [probe-aeneas.md](../tools/probe-aeneas.md#three-strategy-translation-matching)) handle the spectrum:
1. **Rust-qualified-name** — when Charon provides exact names (best case)
2. **File + display-name** — when names match but no qualified name available
3. **File + line-overlap** — when names don't match but source locations overlap (fallback)

Priority order ensures the most confident match wins.

### 1-to-1 constraint simplifies reasoning

Each Rust atom maps to at most one Lean atom and vice versa. This avoids ambiguity in the merged graph — every cross-language edge is unambiguous. The constraint is enforced by matched sets, not just by hoping strategies don't conflict.

### Bidirectional lookup

Translations are stored as `from`/`to` pairs but applied bidirectionally during merge. If atom A depends on code-name X, and X translates to Y, then Y is added as a dependency of A. Both directions checked because Rust atoms may depend on Lean code-names and vice versa.

## Consequences

- Translation files must be generated before merge can add cross-language edges
- The `extract` command in probe-aeneas handles this automatically (generate → merge in one step)
- Standalone `probe merge` requires the user to provide `--translations` explicitly
- Translations are confidence-tagged, enabling consumers to filter by confidence level
- Adding a new source language requires building a translation generator but no changes to probe merge

## Alternatives considered

### Embed translations in atoms

Add a `translation` field directly on atoms pointing to their counterpart. Rejected because:
- Requires each extractor to know about translations (violates separation)
- Merge would need special handling for translation fields
- Can't express translations without running both extractors first

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
