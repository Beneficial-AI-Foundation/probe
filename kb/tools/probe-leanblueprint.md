---
title: "Tool: probe-leanblueprint"
last-updated: 2026-07-31
status: draft
---

# probe-leanblueprint

**Directory**: `baif/probe-leanblueprint/` · **Repo**: <https://github.com/Beneficial-AI-Foundation/probe-leanblueprint>
**Role**: Enricher over a `probe-lean/extract` atom base. Joins Lean **blueprint**
progress (Verso manifest or Massot LaTeX) onto probe-lean's code call graph by
declaration name and re-emits a `probe-leanblueprint/extract` envelope plus a
two-axis `probe-leanblueprint/summary` sidecar. A direct analogue of
[probe-aeneas](probe-aeneas.md).
**Subcommand**: `extract`

> **Catalog stub, not a mechanics reference** ([ADR-005](../decisions/005-doc-ownership-boundary.md)).
> The tool's mechanics — Verso/Massot adapters, join/collision rules, the two-axis
> status vocabulary, every `blueprint-*` field, the CLI, and the probe-lean
> auto-install — are documented normatively in the probe's own repo. This page
> carries its role, the hub contracts it must satisfy, and where to read the rest.

## Normative docs (in the probe-leanblueprint repo)

| Doc | Covers |
|-----|--------|
| [README](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/README.md) | What it is, supported blueprint ecosystems and projects, quick start |
| [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/SCHEMA.md) | **Normative** output semantics: status axes, node classification, every `blueprint-*` field, both output envelopes |
| [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/USAGE.md) | Install (incl. the probe-lean auto-install), flags, output formats, the `blueprint_stats.py` reporter |
| [docs/architecture.md](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/architecture.md) | Internal mechanics: extract pipeline, single-build guarantee, the atom↔blueprint join algorithm, source-file map |

## Hub contracts it must satisfy

- Emits the shared interchange envelope at the current `schema-version` (**3.0**),
  so `probe merge`/`project` accept the extract —
  [engineering/schema.md](../engineering/schema.md),
  [schemas/atom-envelope.schema.json](../../schemas/atom-envelope.schema.json).
- `probe-leanblueprint/extract` is an **Atoms**-category file (matched by the
  `*/extract` rule in `detect_category()`); the `probe-leanblueprint/summary`
  sidecar is not a category and is never merged —
  [P17](../engineering/properties.md#p17-schema-category-consistency).
- Its `blueprint-*` extensions round-trip through `merge`/`project` unchanged —
  [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge).
- Reuses `probe::commands::propagate::enrich_verification_status` and depends on
  the hub crate for shared types (`Atom`, `AtomEnvelope`, `Source`, `Tool`,
  `CodeText`, `load_atom_file`).

## Its own invariant

The machine `verification-status` stays authoritative on the proof axis; the
blueprint's declared status is additive and a `blueprint-status-mismatch` fires
when the blueprint over-claims. This is a probe-leanblueprint rule (currently
recorded as [P26](../engineering/properties.md#p26-blueprint-status-is-additive-machine-verification-status-stays-authoritative);
migrating to the probe repo per [ADR-005](../decisions/005-doc-ownership-boundary.md)).
Its consumer-side obligation — preserve the additive fields through merge — is
the hub contract [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge).

## Design rationale

[ADR-004](../decisions/004-probe-leanblueprint.md) — why it is a standalone
enricher rather than an extension of probe-lean.
