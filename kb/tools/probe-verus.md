---
title: "Tool: probe-verus"
last-updated: 2026-07-31
status: draft
---

# probe-verus

**Directory**: `baif/probe-verus/` · **Repo**: <https://github.com/Beneficial-AI-Foundation/probe-verus>
**Role**: Analyze Verus/Rust codebases for call graphs, function specifications,
and verification status. The most complex extractor in the ecosystem. Emits
`probe-verus/extract` (unified atoms + specs + proofs) plus per-step
atoms/specs/proofs and a verification-report.
**Language**: Rust (exec) + Verus (proof/spec). **Primary subcommand**: `extract`.

> **Catalog stub, not a mechanics reference** ([ADR-005](../decisions/005-doc-ownership-boundary.md)).
> The tool's mechanics — the pipeline, dual-AST (`verus_syn`) parsing,
> interval-tree error mapping, the spec taxonomy engine and its match criteria,
> trait-impl disambiguation, every CLI flag, the trust-base/out-of-scope rules,
> and tool auto-install — are documented normatively in the probe's own repo.
> This page carries its role, the hub contracts it must satisfy, and where to
> read the rest.

## Normative docs (in the probe-verus repo)

| Doc | Covers |
|-----|--------|
| [README](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/README.md) | What it is, prerequisites, supported projects, Verus version detection, quick start |
| [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md) | **Normative** output semantics: every schema, all fields, trust-base & out-of-scope |
| [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/USAGE.md) | Full command + flag reference, taxonomy config format |
| [docs/HOW_IT_WORKS.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/HOW_IT_WORKS.md) | SCIP call graph, `verus_syn` spans, spec extraction, taxonomy classification, trait-impl disambiguation |
| [docs/VERIFICATION_ARCHITECTURE.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/VERIFICATION_ARCHITECTURE.md) | Verification-analysis internals, trust-base overrides, interval tree |

## Hub contracts it must satisfy

- Emits the shared interchange envelope; `*/extract` and `*/atoms` are
  **Atoms**-category files ([schema.md](../engineering/schema.md),
  [P17](../engineering/properties.md#p17-schema-category-consistency)).
- `dependencies` = union of `requires`/`ensures`/`body`-dependencies
  ([P15](../engineering/properties.md#p15-dependency-completeness)).
- `verification-status` follows the shared vocabulary
  ([P16](../engineering/properties.md#p16-verification-status-mapping)); enrichment
  upgrades to `transitively-verified`
  ([P23](../engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)).
- The `kind → language` value convention (exec→rust, proof|spec→verus) matches
  the shared schema
  ([schema.md#language-assignment-for-verus-atoms](../engineering/schema.md#language-assignment-for-verus-atoms));
  `probe summary` depends on it.
- `trusted-reason` values normalize per the cross-tool vocabulary
  ([P22](../engineering/properties.md#p22-cross-tool-trust-reason-vocabulary)); RQNs
  align with probe-rust
  ([P21](../engineering/properties.md#p21-cross-tool-rqn-alignment)).
- `untracked` / out-of-scope follows the analysis-scope rules
  ([P24](../engineering/properties.md#p24-a-status-bearing-atom-is-in-analysis-scope)/[P25](../engineering/properties.md#p25-atoms-not-in-the-verification-build-are-out-of-scope));
  extensions round-trip through merge
  ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).

## Its own invariant

`language` is derived from `kind`, not lexical `verus!{}` scope (exec = compiled
Rust, proof/spec = erased Verus constructs). This is a probe-verus rule,
normative in the probe's own [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md).
The resulting `kind → language` value convention is also recorded in the shared
[schema.md#language-assignment-for-verus-atoms](../engineering/schema.md#language-assignment-for-verus-atoms),
because `probe summary` depends on it.
