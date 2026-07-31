---
title: "Tool: probe-rust"
last-updated: 2026-07-31
status: draft
---

# probe-rust

**Directory**: `baif/probe-rust/` · **Repo**: <https://github.com/Beneficial-AI-Foundation/probe-rust>
**Role**: Extractor. Produces call-graph [atoms](../engineering/glossary.md#atom)
from any standard Rust project (Cargo.toml → rust-analyzer → SCIP → `syn` spans →
Schema envelope) and emits a `probe-rust/extract` envelope. Feeds probe-aeneas for
Rust↔Lean mapping.
**Subcommands**: `extract`, `callee-crates`, `list-functions` (plus `setup`)

> **Catalog stub, not a mechanics reference** ([ADR-005](../decisions/005-doc-ownership-boundary.md)).
> The tool's mechanics — the extract pipeline, trait-impl disambiguation, SCIP
> caching, `syn` span resolution, Charon / cargo-public-api enrichment, every atom
> field, and the CLI — are documented normatively in the probe's own repo. This
> page carries its role, the hub contracts it must satisfy, and where to read the
> rest.

## Normative docs (in the probe-rust repo)

| Doc | Covers |
|-----|--------|
| [README](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/README.md) | What it is, prerequisites, install, quick start, CI integration |
| [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/SCHEMA.md) | **Normative** output semantics: every atom field, code-name format, envelope, all three command outputs, probe-verus compatibility |
| [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/USAGE.md) | Full CLI: every flag, external-tool resolution, SCIP caching, env vars, CI |
| [docs/architecture.md](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/architecture.md) | Internal mechanics: extract pipeline, trait-impl disambiguation strategies, source-file map |

## Hub contracts it must satisfy

- Emits the shared interchange envelope, so `probe merge`/`project` accept the
  extract ([schema.md](../engineering/schema.md),
  [atom-envelope.schema.json](../../schemas/atom-envelope.schema.json)).
- `probe-rust/extract` is an **Atoms**-category file (matched by the `*/extract`
  rule in `detect_category()`) —
  [P17](../engineering/properties.md#p17-schema-category-consistency).
- Its optional Rust fields (`rust-qualified-name`, `is-public`, `is-public-api`,
  `cfg`, `charon-def-id`/`charon-version`, `untracked`) round-trip through
  `merge`/`project` unchanged as extensions —
  [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge).
- `kind` is always `"exec"`; `dependencies-with-locations` `location` is always
  `"inner"` (the exec-only shape of the shared atom).
- Cross-tool join vocabulary: `charon-def-id` equals Aeneas's `translation.json`
  `def_id` (provenance-gated by `charon-version`), the integer join consumed by
  probe-aeneas; RQNs align with probe-verus
  ([P21](../engineering/properties.md#p21-cross-tool-rqn-alignment)).
- Validated by the hub's `probe-extract-check` (its sole hub dependency —
  probe-rust does **not** depend on the `probe` crate).

## Design rationale

[ADR-001](../decisions/001-separate-repos.md) — why each extractor is a
standalone repo.
