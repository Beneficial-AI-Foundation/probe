---
title: Per-Tool Knowledge
last-updated: 2026-07-31
status: draft
---

# Per-Tool Knowledge

The hub owns full docs for its own subcommands (`merge`, `project`, `summary`).
Each external probe has a **catalog stub** ([ADR-005](../decisions/005-doc-ownership-boundary.md)):
its role, the hub contracts it must satisfy, and a link to the probe's own
normative docs. Mechanics live in each probe's repo, next to the code. For shared
concepts (envelope, atom fields, merge), see [engineering/](../engineering/index.md).

## Hub subcommands (full docs)

| File | Subcommand | Covers |
|------|------------|--------|
| [probe-merge.md](probe-merge.md) | `probe merge` | Merge algorithm, schema category detection, mapping application |
| [probe-project.md](probe-project.md) | `probe project` | Graph projection from mapping seeds, focus-set emission |
| [probe-summary.md](probe-summary.md) | `probe summary` | Entrypoint analysis, verified-dependency partitioning |

## External probes (catalog stubs → repo docs)

| File | Tool | Language | Repo |
|------|------|----------|------|
| [probe-rust.md](probe-rust.md) | probe-rust | Rust | [repo](https://github.com/Beneficial-AI-Foundation/probe-rust) |
| [probe-verus.md](probe-verus.md) | probe-verus | Rust + Verus | [repo](https://github.com/Beneficial-AI-Foundation/probe-verus) |
| [probe-lean.md](probe-lean.md) | probe-lean | Lean 4 | [repo](https://github.com/Beneficial-AI-Foundation/probe-lean) |
| [probe-aeneas.md](probe-aeneas.md) | probe-aeneas | Rust | [repo](https://github.com/Beneficial-AI-Foundation/probe-aeneas) |
| [probe-leanblueprint.md](probe-leanblueprint.md) | probe-leanblueprint | Rust + Python | [repo](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint) |

## When to read which file

- Modifying the merge algorithm or Schema 3.0 types → [probe-merge.md](probe-merge.md)
- Working on graph projection from mappings or focus-set emission → [probe-project.md](probe-project.md)
- Working on entrypoint analysis or verified-dependency partitioning → [probe-summary.md](probe-summary.md)
- Working on any external probe's mechanics → its stub here for the hub contracts,
  then the probe's own repo docs for the mechanics.
