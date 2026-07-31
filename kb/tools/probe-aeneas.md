---
title: "Tool: probe-aeneas"
last-updated: 2026-07-31
status: draft
---

# probe-aeneas

**Directory**: `baif/probe-aeneas/` · **Repo**: <https://github.com/Beneficial-AI-Foundation/probe-aeneas>
**Role**: Cross-language bridge for Aeneas-transpiled projects. Generates Rust↔Lean
[cross-language mappings](../engineering/glossary.md#cross-language-mapping) and
delegates merging to `probe merge` — an instantiation of the hub merge operator.
Emits `probe-aeneas/extract`.
**Subcommands**: `extract`, `translate`, `listfuns`

> **Catalog stub, not a mechanics reference** ([ADR-005](../decisions/005-doc-ownership-boundary.md)).
> The tool's mechanics — the matching strategies and normalization, charon/LLBC
> orchestration, the enrichment fields, every CLI flag, and tool auto-install —
> are documented normatively in the probe's own repo. This page carries its role,
> the hub contracts it must satisfy, and where to read the rest.

## Normative docs (in the probe-aeneas repo)

| Doc | Covers |
|-----|--------|
| [README](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/README.md) | What it is, prerequisites, supported projects, quick start |
| [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md) | **Normative** output semantics: Rust-specific and translation-metadata fields, `untracked`, `is-public` |
| [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/USAGE.md) | Full CLI, the four mapping strategies + normalization, charon pre-generation |
| [docs/architecture.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/architecture.md) | Internal mechanics: the merge instantiation, phases, shared-type usage |

## Hub contracts it must satisfy

- Emits the shared interchange envelope; `probe-aeneas/extract` is an
  **Atoms**-category file ([P17](../engineering/properties.md#p17-schema-category-consistency)),
  accepted by `probe merge`/`project`.
- `translate` emits a valid `probe/mappings` file; merge consumes it as
  **1-to-many** and adds an edge only if the target exists
  ([P13](../engineering/properties.md#p13-cross-language-edges-require-existence),
  [schema.md#mappings-file-format](../engineering/schema.md#mappings-file-format),
  [mappings-spec.md](../../docs/mappings-spec.md)).
- `translation-*` / `is-public` extensions round-trip through merge unchanged
  ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).
- **The only probe with a Rust crate dependency on the hub**: imports
  `merge_atom_files`, `Atom`, `Mapping`, `MergedAtomEnvelope`, `InputProvenance`,
  `Tool` from `probe::`, and calls `enrich_verification_status`
  ([P23](../engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination))
  as the final extract step.

## Its own invariant

Mapping generation is 1-to-1 and strategy-priority-ordered — a probe-aeneas rule,
normative in the probe's own [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/USAGE.md)
and [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md).
The hub's merge does not depend on it — it accepts 1-to-many mappings
([P13](../engineering/properties.md#p13-cross-language-edges-require-existence)).

## Design rationale

[ADR-003](../decisions/003-mappings-design.md) — bidirectional cross-language
mappings for cross-language edges.
