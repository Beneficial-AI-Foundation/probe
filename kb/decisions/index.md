---
title: Architectural Decision Records
last-updated: 2026-07-21
status: draft
---

# Architectural Decision Records

Why we chose this approach over alternatives. Each record captures the context, decision, and consequences.

## Records

| ADR | Decision | Status |
|-----|----------|--------|
| [001-separate-repos.md](001-separate-repos.md) | Keep five separate tool directories instead of a monorepo | Accepted |
| [002-schema-2.0.md](002-schema-2.0.md) | Wrap all output in Schema 3.0 metadata envelopes | Accepted |
| [003-mappings-design.md](003-mappings-design.md) | Use bidirectional cross-language mapping files; generation semantics (application semantics superseded by ADR-006) | Accepted (application superseded) |
| [004-probe-leanblueprint.md](004-probe-leanblueprint.md) | Standalone probe-leanblueprint enriches probe-lean atoms with two-axis blueprint status (Verso + Massot); machine status stays authoritative | Accepted |
| [005-doc-ownership-boundary.md](005-doc-ownership-boundary.md) | Hub holds cross-probe contracts; each probe owns its own mechanics. `kb/tools/*.md` become catalog stubs | Accepted |
| [006-correspondence-records.md](006-correspondence-records.md) | Mappings attach correspondence records (`maps-to`/`mapped-from`), never dependency edges; `status-origin` evidence markers; enrichment recomputation with whole-atom trust; projection rejection; per-producer version gate; summary consumer contract; regeneration policy | Accepted |
