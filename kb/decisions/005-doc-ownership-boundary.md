---
title: "ADR-005: Doc ownership — hub holds cross-probe contracts, each probe owns its own mechanics"
last-updated: 2026-07-31
status: Accepted
---

# ADR-005: Doc ownership — hub holds cross-probe contracts, each probe owns its own mechanics

## Context

The hub KB carries a full per-tool mechanics doc for every probe
(`kb/tools/*.md`), and each probe repo also ships its own docs (`README`,
`USAGE`, `SCHEMA.md`). The same mechanics are written in two repos, and the hub
copy is not updated in the same PR as the code it describes.

Single-probe rules also sit in the shared invariants file: **P11/P12** (mapping
generation is 1-to-1, probe-aeneas) and **P26** (blueprint status is additive)
in `kb/engineering/properties.md`.

## Decision

Adopt one ownership rule for ecosystem docs.

**Boundary test — does the statement survive deleting the probe?**
Yes → it is a **hub contract**. No, it evaporates → it is **probe-specific** and
lives in the probe repo, next to the code, updated in the same PR.

**The hub (`probe`) owns:**

- The interchange envelope schema and `schema-version` rules
  (`schemas/atom-envelope.schema.json`, `docs/envelope-rationale.md`, the shared
  `docs/SCHEMA.md`).
- Cross-probe invariants: merge/project/summary semantics, category detection,
  code-name normalization, provenance and extension preservation, cross-tool
  RQN/trust-reason vocabulary, analysis-scope rules.
- Ecosystem ADRs — the decision and its rationale, linking out for mechanics.
- A **probe catalog**: per probe, its role, language, the hub contracts it must
  satisfy, and a link to the probe's own normative docs. No mechanics.

**Each probe repo owns (normative, beside its code):**

- Adapters, join/collision rules, extension fields, CLI, output specifics, and
  build/install orchestration.
- Its own single-probe invariants.

## Consequences

- `kb/tools/*.md` shrink from mechanics references to **catalog stubs**. This ADR
  ships `kb/tools/probe-leanblueprint.md` as the first instance; the other six
  (`probe-rust`, `probe-verus`, `probe-lean`, `probe-aeneas`, and the three
  hub-owned `probe-*` subcommands) follow in per-probe PRs.
- Single-probe invariants migrate out of `kb/engineering/properties.md`:
  **P11/P12** to probe-aeneas, **P26** to probe-leanblueprint. Where such a rule
  constrains the hub's own merge, it folds into the existing cross-probe property
  (P26's consumer obligation is already covered by
  [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).
  Anchor links referencing the moved properties (each probe's `SCHEMA.md`,
  ADR-004) update in the same move, deferred to the per-probe follow-ups.
- The catalog keeps a one-line invariant summary per probe for at-a-glance
  discovery.
