---
title: "ADR-005: Doc ownership — hub holds cross-probe contracts, each probe owns its own mechanics"
last-updated: 2026-07-31
status: Accepted
---

# ADR-005: Doc ownership — hub holds cross-probe contracts, each probe owns its own mechanics

## Context

The hub KB carries a full **per-tool mechanics doc** for every probe
(`kb/tools/*.md`), alongside the cross-cutting invariants
(`kb/engineering/properties.md`) and the interchange schema. Each probe repo
*also* ships its own docs (`README`, `USAGE`, `SCHEMA.md`). The same mechanics
are therefore written down in two repos, and the hub copy drifts because it is
not updated in the same PR as the code it describes.

This is not hypothetical:

- `kb/tools/probe-leanblueprint.md` still says the tool emits a **"Schema 2.0"**
  envelope (lines 15, 103). The tool emits **3.0** (`src/emit.rs`:
  `SCHEMA_VERSION = "3.0"`), the probe's own normative `docs/SCHEMA.md` says 3.0,
  and the sibling [ADR-004](004-probe-leanblueprint.md) says 3.0. The hub doc
  contradicts a decision record in the same repo.
- The same doc omits the probe-lean **auto-install** step and the newer
  `blueprint-*` fields (upstream-proved split, `*-probe-lean-confirmed`,
  `blueprint-provenance`) that `docs/SCHEMA.md` already specifies.
- `docs-probe-consistency-review.md` (finding 2) independently found the same
  root cause for the `language`/`kind` enums: "three copies of the same facts,
  two already wrong."

Meanwhile the probe already declares the intended direction:
`probe-leanblueprint/docs/SCHEMA.md` calls itself normative and the hub tool doc
"a **non-normative summary**"; `kb/tools/index.md` says each file "covers what is
**unique** to that tool." The boundary is written down but was never held.

Single-probe rules have also leaked into the shared invariants file: **P11/P12**
("Mapping generation is 1-to-1 **(probe-aeneas)**") and **P26** ("Blueprint
status is additive") sit in `kb/engineering/properties.md` next to genuinely
cross-probe properties.

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
- The drift class is eliminated for the migrated docs: a probe's mechanics change
  in one PR, in one repo, next to the code.
- Single-probe invariants migrate out of `kb/engineering/properties.md`:
  **P11/P12** to probe-aeneas, **P26** to probe-leanblueprint. Where such a rule
  really constrains the hub's own merge (P26's consumer obligation is already
  covered by [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)),
  it folds into the existing cross-probe property rather than being duplicated.
  Anchor links that reference the moved properties (each probe's `SCHEMA.md`,
  ADR-004) update in the same move. Deferred to the per-probe follow-ups so this
  ADR stays reviewable.
- Cost: one extra click from the hub to a probe's docs. The catalog keeps a
  one-line invariant summary per probe so at-a-glance discovery survives.

## Alternatives considered

- **Keep full mechanics in the hub and sync manually** (status quo; the recently
  merged "sync SCHEMA.md with KB" work, PR #47). Rejected as the durable fix:
  cross-repo manual sync is exactly what drifted — two of three enum copies were
  already wrong, and the `Schema 2.0` error survived that very sync PR.
- **Pure-pointer catalog** (no per-probe invariant line in the hub). Rejected: it
  trades all at-a-glance discovery for one saved click. A single, controlled
  one-line summary per probe is worth keeping.
