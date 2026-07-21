---
title: "ADR-004: probe-leanblueprint as a standalone blueprint enricher"
last-updated: 2026-07-21
status: Accepted
---

# ADR-004: probe-leanblueprint as a standalone blueprint enricher

## Context

probe-lean extracts dependency graphs and machine sorry-status from any Lean 4 project. Because it is generic across all Lean projects (not just software-verification ones), it cannot distinguish implementations from specifications and cannot produce meaningful progress statistics beyond a theorem count.

Lean **blueprints** solve exactly this: authors annotate their project with a human-authored roadmap and a two-axis status (is the *statement* formalized? is the *proof* complete?). Two ecosystems exist:

- **Patrick Massot `leanblueprint`** — LaTeX/plasTeX, the Mathlib-community standard. Macros `\lean`, `\leanok`, `\uses`, `\notready`, `\discussion`. Emits HTML, not machine-readable JSON.
- **Verso Blueprint** (`versoBlueprint`) — Lean-native, used by baif projects. Renders a machine-readable `blueprint-manifest.json` with per-node `statementStatus`/`proofStatus` and `canonical` Lean-decl bindings.

We want blueprint-derived progress stats without turning probe-lean into a project-type-specific monster.

## Decision

Build a **standalone `probe-leanblueprint` tool** that treats `probe-lean/extract` output as its atom base and enriches it with blueprint metadata, re-emitting a Schema 2.0 envelope. It is a direct analogue of probe-aeneas (a Rust probe that consumes other probes and enriches atoms via the hub crate).

Sub-decisions:

1. **Support both ecosystems** behind a common enrichment core fed by two adapters. Verso: parse the manifest JSON directly. Massot: a small bundled plasTeX emitter that reuses leanblueprint's own parser — we do **not** write a TeX parser.
2. **probe-leanblueprint owns build orchestration; probe-lean stays blueprint-unaware.** Because lake builds are incremental and code libs are shared with the docs target, total cost is one full compile (see [probe-leanblueprint.md § single-build guarantee](../tools/probe-leanblueprint.md#single-build-guarantee)).
3. **Machine-authoritative `verification-status` + additive blueprint fields + a mismatch flag.** The statement axis is blueprint-exclusive; on the proof axis, `verification-status` remains probe-lean's machine sorry-truth, the blueprint's claim lives in `blueprint-proof-status`, and disagreement raises `blueprint-status-mismatch`. See [P26](../engineering/properties.md#p26-blueprint-status-is-additive-machine-verification-status-stays-authoritative).
4. **Node-indexed summary is a first-class output.** A blueprint node is not 1:1 with an atom (one node → many decls; planned nodes with no decl). Enriched atoms are the spine for merge/downstream; the two-axis stats live in the `probe-leanblueprint/summary` sidecar.

## Alternatives considered

- **Extend probe-lean with a thin blueprint wrapper.** Rejected: probe-lean is written in Lean (language mismatch for a LaTeX/JSON adapter), it would couple a complementary doc layer into the generic extractor, and it contradicts the KB positioning of blueprint as doc-authoritative.
- **A blueprint-native tool disconnected from probe-lean.** Rejected: it would force brand-new ingestion logic in every downstream consumer (verilib, scip-callgraph) and fragment the shared atom model. Emitting Schema 2.0 with blueprint data as extensions means schema-driven consumers need zero changes.

## Consequences

- New repo `baif/probe-leanblueprint/` depending on the `probe` hub crate.
- New schema values `probe-leanblueprint/extract` (atoms category, already matched by the `*/extract` rule in `detect_category()`) and `probe-leanblueprint/summary` (sidecar, not merged).
- New `language: "blueprint"` and `kind: "blueprint-definition"`/`"blueprint-theorem"` for synthetic planned atoms.
- `probe merge`/`project` round-trip the blueprint extensions and synthetic atoms unchanged (P10).
- The Massot path adds a Python/plasTeX/graphviz runtime dependency, isolated to that adapter.
