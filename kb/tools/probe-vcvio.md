---
title: "Tool: probe-vcvio"
last-updated: 2026-09-28
status: draft
---

# probe-vcvio

**Directory**: `baif/probe-vcvio/` · **Repo**: <https://github.com/Beneficial-AI-Foundation/probe-vcvio>
**Role**: Security-protocol classification annotator over probe-lean extracts.
Accepts an existing `probe-lean/extract` envelope (`--lean`), annotates atoms
with protocol classification, and re-emits every atom — verification statuses
included — as `probe-vcvio/extract` under its own tool identity.

> **Catalog stub, not a mechanics reference** ([ADR-005](../decisions/005-doc-ownership-boundary.md)).
> The tool's classification mechanics are documented normatively in its own
> repo. This page carries its role, the hub contracts it must satisfy, and
> where to read the rest.

## Normative docs (in the probe-vcvio repo)

| Doc | Covers |
|-----|--------|
| [README](https://github.com/Beneficial-AI-Foundation/probe-vcvio/blob/main/README.md) | What it is, prerequisites, quick start |

## Hub contracts it must satisfy

- Emits the shared interchange envelope; `probe-vcvio/extract` is an
  **Atoms**-category file (matched by the `*/extract` rule in
  `detect_category()`) — [P17](../engineering/properties.md#p17-schema-category-consistency);
  registered in [schema.md](../engineering/schema.md#registered-schema-values)
  and accepted by the executable schema's single-tool branch.
- Reuses the hub's `Atom` type, so `status-origin` markers and
  correspondence records survive its round-trip
  ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).
- **Composer obligation** ([ADR-006](../decisions/006-correspondence-records.md),
  composer shape ii — annotation re-emitter): it re-emits foreign
  verification evidence under its own tool identity, which conceals the
  evidence's origin from a gate keyed on the original producer's name. It
  must therefore run the shared validator's **version-gate component** on
  its input envelope before re-emitting (its exact-schema input check
  already rejects projections and non-`probe-lean/extract` shapes) — a
  pre-marker probe-lean extract is rejected, not laundered.
- Pre-contract outputs (below the ADR-006 gate threshold for `probe-vcvio`)
  are rejected by hub merge/enrich/summary/project; outputs still needed
  are regenerated from re-extracted inputs.
