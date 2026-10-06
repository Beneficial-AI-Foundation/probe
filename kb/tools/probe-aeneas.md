---
title: "Tool: probe-aeneas"
last-updated: 2026-10-06
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
  accepted by `probe merge`/`project`. It is a **composed** envelope
  (`inputs: [Rust, Lean]`, no `source`) — detected structurally, inventory
  preserved ([P9](../engineering/properties.md#p9-provenance-is-preserved)).
- `translate` emits a valid `probe/mappings` file; merge consumes it as
  **1-to-many** and attaches correspondence records
  ([P13](../engineering/properties.md#p13-correspondence-records-attach-unconditionally),
  [P27](../engineering/properties.md#p27-correspondence-records-are-unioned-and-inert),
  [schema.md#mappings-file-format](../engineering/schema.md#mappings-file-format)). The generated
  `Vec<Mapping>` stays authoritative through its pipeline — endpoint-only
  maps are derived indexes over the P8-normalized records; `confidence`/
  `method` are never discarded or reconstructed from endpoints.
- **`status-origin: "translation"` marker** ([ADR-006](../decisions/006-correspondence-records.md)):
  every verification status copied from a Lean atom onto a Rust atom is
  marked, and a copied `transitively-verified` is normalized to `verified`
  at copy time. Marked atoms are blocker seeds in hub enrichment
  ([P23](../engineering/properties.md#p23-transitive-verification)) —
  imported evidence is never turned into a local transitive claim.
- **Single final enrichment**: its extract pipeline stages on the hub's
  raw (no-enrichment, authority-validating) merge primitive and runs
  enrichment exactly once, after its metadata phase; `--skip-enrich`
  guards the only enrichment pass. Precomputed input files pass the same
  authority validation (projection rejection + version gate), before any
  other work. An input provenance guard also rejects a Rust input unless
  every provenance entry has schema `probe-rust/extract`, and a Lean input
  unless every entry has schema `probe-lean/extract`. So a probe-aeneas
  output cannot be fed back in, and every status on a Rust atom is a copy
  from the same run.
- **Scope** ([P24](../engineering/properties.md#p24-a-status-bearing-atom-is-in-analysis-scope),
  [P25](../engineering/properties.md#p25-atoms-not-in-the-verification-build-are-out-of-scope)):
  `untracked` follows the in-scope rule. A status or a matched translation
  keeps a Rust function tracked, unless the translation carries
  `@[out_of_scope]`.
- `translation-*` / `is-public` extensions round-trip through merge unchanged
  ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).
- **Rust crate dependency on the hub** (probe-verus, probe-leanblueprint and
  probe-vcvio have one too): 0.21.0 imports `merge_atom_files_raw`, `endpoint_lookup_maps`,
  `load_atom_file`, `Atom`, `Mapping`, `MergedAtomEnvelope`, `InputProvenance` and
  `Tool` from `probe::`, and calls `enrich_verification_status`
  ([P23](../engineering/properties.md#p23-transitive-verification)).
- Pre-contract extracts (below the [ADR-006](../decisions/006-correspondence-records.md)
  gate threshold for `probe-aeneas`) are rejected by hub merge/enrich/summary/project
  and are regenerated, not repaired.

## Its own invariant

Mapping generation is 1-to-1 and strategy-priority-ordered — a probe-aeneas rule,
normative in the probe's own [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/USAGE.md)
and [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md).
The hub's merge does not depend on it — it accepts 1-to-many mappings
([P13](../engineering/properties.md#p13-correspondence-records-attach-unconditionally)).

## Design rationale

[ADR-003](../decisions/003-mappings-design.md) — bidirectional cross-language
mapping generation. [ADR-006](../decisions/006-correspondence-records.md) —
application semantics (correspondence records, `status-origin: "translation"`,
single final enrichment).
