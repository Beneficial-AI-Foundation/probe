---
title: "Tool: probe-leanblueprint"
last-updated: 2026-09-28
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
- **Composer obligation** ([ADR-006](../decisions/006-correspondence-records.md),
  composer shape i — enrich-and-re-emit): it runs hub enrichment over a
  foreign atom base and re-emits under its own tool identity, so it must
  call the shared authority validator on its input first — rejecting
  projections (both formats) and pre-contract envelopes — and preserve
  `status-origin` markers end to end. Without this it is a laundering
  channel: a pre-marker Lean extract goes in, an apparently authoritative
  extract with a fresh tool version comes out.
- **Synthesis rule** ([ADR-006](../decisions/006-correspondence-records.md)):
  `derive_synthetic_verification` propagates contributors' `status-origin`
  markers — a binding over marked evidence never synthesizes an unmarked
  `trusted`/`verified`. Synthetic `language: "blueprint"` statuses are
  **presentation aggregates** outside the code-atom assurance contract
  ([P23](../engineering/properties.md#p23-transitive-verification)):
  they are never trust-boundary assertions, and `probe summary` excludes
  blueprint-language atoms from every partition.
- Emits a **composed `inputs` envelope** carrying the loaded
  `Vec<InputProvenance>` through ([P9](../engineering/properties.md#p9-provenance-is-preserved))
  rather than selecting a single `source`.
- Pre-contract outputs (below the ADR-006 gate threshold for
  `probe-leanblueprint`) are rejected by hub merge/enrich/summary/project
  and are regenerated, not repaired.

## Its own invariant

The machine `verification-status` stays authoritative on the proof axis; the
blueprint's declared status is additive and a `blueprint-status-mismatch` fires
when the blueprint over-claims. This is a probe-leanblueprint rule, normative in
the probe's own [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/SCHEMA.md).
Its consumer-side obligation — preserve the additive fields through merge — is
the hub contract [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge).

## Design rationale

[ADR-004](../decisions/004-probe-leanblueprint.md) — why it is a standalone
enricher rather than an extension of probe-lean.
