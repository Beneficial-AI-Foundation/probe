---
title: Engineering
last-updated: 2026-03-19
status: draft
---

# Engineering

How the probe ecosystem is built. Start with [architecture.md](architecture.md) for the big picture, then consult specific files as needed.

## Files

| File | What it covers |
|------|---------------|
| [architecture.md](architecture.md) | Five-tool separation, data flow, repo boundaries, per-tool roles |
| [schema.md](schema.md) | Schema 3.x interchange format: envelope, atom fields, code-name URIs, mappings files, merged output |
| [properties.md](properties.md) | Invariants and correctness constraints that must hold across all tools |
| [glossary.md](glossary.md) | Precise definitions of domain terms |
| [categorical-framework.md](categorical-framework.md) | Non-normative: categorical/algebraic structure of the merge operator |

## Key relationships

- [architecture.md](architecture.md) defines component boundaries; [properties.md](properties.md) defines what must be true at those boundaries
- [schema.md](schema.md) is the contract between tools; [glossary.md](glossary.md) defines the terms used in that contract
- Per-tool details live in [../tools/](../tools/) — this section covers cross-cutting concerns only
