---
title: "Tool: probe-lean"
last-updated: 2026-09-28
status: draft
---

# probe-lean

**Directory**: `baif/probe-lean/` · **Repo**: <https://github.com/Beneficial-AI-Foundation/probe-lean>
**Role**: Extract dependency graphs and verification status from Lean 4 projects.
Emits `probe-lean/extract` plus a `probe-lean/viewify` view.
**Language**: Written entirely in Lean 4 (not Rust — the primary reason for repo
separation, [ADR-001](../decisions/001-separate-repos.md)).
**Subcommands**: `extract`, `check-axioms`, `viewify`

> **Catalog stub, not a mechanics reference** ([ADR-005](../decisions/005-doc-ownership-boundary.md)).
> The tool's mechanics — the extract pipeline, lake build / Mathlib cache, sorry
> detection, declaration filtering, the spec-precedence chain, co-importability,
> every field, the CLI, toolchain matching, and security-protocol classification
> — are documented normatively in the probe's own repo. This page carries its
> role, the hub contracts it must satisfy, and where to read the rest.

## Normative docs (in the probe-lean repo)

| Doc | Covers |
|-----|--------|
| [README](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/README.md) | What it is, prerequisites, toolchain matching, quick start |
| [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md) | **Normative** output semantics: every field, sorry detection, trust base, spec-precedence chain |
| [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/USAGE.md) | Full CLI, lake build + Mathlib cache, declaration filtering config |
| [docs/classification-security-protocol.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/classification-security-protocol.md) | Security-protocol `classification` field |

## Hub contracts it must satisfy

- Emits the shared interchange envelope
  ([schema.md](../engineering/schema.md),
  [atom-envelope.schema.json](../../schemas/atom-envelope.schema.json)).
- `probe-lean/extract` is an **Atoms**-category file (`*/extract`); `probe-lean/viewify`
  is a view and is never merged —
  [P17](../engineering/properties.md#p17-schema-category-consistency).
- `dependencies` = deduplicated union of `type-dependencies` + `term-dependencies`
  ([P15](../engineering/properties.md#p15-dependency-completeness)).
- `verification-status` / `trusted-reason` use the shared vocabulary
  ([P16](../engineering/properties.md#p16-verification-status-mapping),
  [P22](../engineering/properties.md#p22-cross-tool-trust-reason-vocabulary)).
- **`status-origin: "kernel-taint"` marker** ([ADR-006](../decisions/006-correspondence-records.md)):
  probe-lean's authoritative statuses come from a kernel-level taint walk,
  and its `verified` verdict can rest on taint the emitted graph cannot
  express (e.g. a sorry in a compiler-generated proof auxiliary that is
  not an atom). Whenever the taint pass runs — including under
  `--skip-enrich` — such atoms are stamped `status-origin: "kernel-taint"`.
  Marked atoms are blocker seeds in hub enrichment
  ([P23](../engineering/properties.md#p23-transitive-verification)):
  never promoted to `transitively-verified`, and blocking promotion of
  their callers. An **unmarked** `verified` atom is safely promotable by
  hub recomputation. Pre-marker extracts (below the ADR-006 gate threshold
  for `probe-lean`) are rejected by hub merge/enrich/summary/project and
  are re-extracted, not re-used.
- Deterministic output
  ([P14](../engineering/properties.md#p14-deterministic-output)).
- Lean-specific and `classification` extension fields round-trip through
  merge/project unchanged
  ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).

## Its own invariant

Lean atoms have no `specified` field — whether an atom has specs is inferred from
`specs` being non-empty. Normative in the probe's own
[docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md).
