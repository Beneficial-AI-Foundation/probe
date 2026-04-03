---
title: Per-Tool Knowledge
last-updated: 2026-04-03
status: draft
---

# Per-Tool Knowledge

Each file covers what is **unique** to that tool — what it does differently from the common patterns described in [engineering/](../engineering/index.md). For shared concepts (envelope, atom fields, merge), see the engineering section.

## Files

| File | Tool | LOC | Language | Complexity |
|------|------|-----|----------|------------|
| [probe-merge.md](probe-merge.md) | probe (merge) | ~1.5K | Rust | Low |
| [probe-summary.md](probe-summary.md) | probe (summary) | ~0.3K | Rust | Low |
| [probe-rust.md](probe-rust.md) | probe-rust | ~6K | Rust | Medium |
| [probe-verus.md](probe-verus.md) | probe-verus | ~13K | Rust | Highest |
| [probe-lean.md](probe-lean.md) | probe-lean | ~5.7K | Lean 4 | Medium-high |
| [probe-aeneas.md](probe-aeneas.md) | probe-aeneas | ~2.3K | Rust | Medium |

## When to read which file

- Modifying the merge algorithm or Schema 2.0 types → [probe-merge.md](probe-merge.md)
- Working on entrypoint analysis or verified-dependency partitioning → [probe-summary.md](probe-summary.md)
- Fixing Rust extraction issues (SCIP, trait disambiguation) → [probe-rust.md](probe-rust.md)
- Working on Verus verification, spec taxonomy, or dual-AST parsing → [probe-verus.md](probe-verus.md)
- Touching Lean environment walking, sorry detection, or lake builds → [probe-lean.md](probe-lean.md)
- Working on cross-language translation or parallel orchestration → [probe-aeneas.md](probe-aeneas.md)
