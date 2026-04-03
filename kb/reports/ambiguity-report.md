---
auditor: ambiguity-auditor
date: 2026-04-03
status: 0 critical, 5 warnings, 7 info
---

## Re-audit (2026-04-03)

Focused pass on fixes for **W2**, **W5**, and **W8** from the prior report. **Resolved:**

| ID | Resolution (verified) |
|----|------------------------|
| **W2** | `kb/engineering/architecture.md` “Why separate directories” now references `decisions/001-separate-repos.md` without *(planned)*, consistent with ADR-001 `status: accepted`. |
| **W5** | `probe query` is documented: `kb/index.md` links `tools/probe-query.md`; architecture lists `src/commands/query.rs` and subcommands `merge`, `query`; `kb/product/spec.md` adds capability **5. Entrypoint analysis**; `kb/tools/probe-query.md` specifies behavior, CLI, output, and properties. |
| **W8** | P20-related files called out previously now carry `last-updated: 2026-04-03` (`properties.md`, `schema.md`, `glossary.md`, `probe-verus.md`), along with other KB files touched in this round (`index.md`, `architecture.md`, `spec.md`, `probe-query.md`). |

## Critical

None.

Normative texts reviewed (including P20, Verus `language` assignment, and cross-links between `schema.md`, `properties.md`, `glossary.md`, and `tools/probe-verus.md`) are mutually consistent on **atom-level** `language` vs `kind` for probe-verus. No contradiction was found between P20 and the schema or glossary.

## Warnings

### [W1] P16 omits probe-lean `verification-status: "failed"`
- **Location**: `kb/engineering/properties.md`, §P16 (approx. lines 128–138)
- **Issue**: P16 documents only sorry-based `"unverified"` vs otherwise `"verified"` for probe-lean. `kb/tools/probe-lean.md` additionally specifies build failure → `"failed"`. That third state is not part of the cross-tool invariant list, so correctness checks against `properties.md` alone can miss it.
- **Recommendation**: Extend P16 with the same three-way table as in `probe-lean.md` (or a single sentence referencing build failure), and add a cross-link from `probe-lean.md` to the updated property.

### [W3] ADR-001 vs P19 / probe-aeneas on how probe-aeneas depends on probe
- **Location**: `kb/decisions/001-separate-repos.md`, “Consequences” (approx. lines 47–49); `kb/tools/probe-aeneas.md` “Dependency on probe crate” (approx. lines 176–182); `kb/engineering/properties.md` P19
- **Issue**: ADR-001 states probe-aeneas imports probe via a **local path** dependency and that moving directories breaks it. The tool KB documents a **git** URL dependency and P19 mandates git (not path) across repositories. Readers cannot tell which is normative for published/consumed layouts vs monorepo-style checkouts.
- **Recommendation**: Amend ADR-001 consequences to distinguish local dev (path/patch) from published dependency policy, and reference P19. Ensure `probe-aeneas.md` remains the operational source of truth for the dependency declaration.

### [W4] Contradictory Charon installation guidance (probe-aeneas vs probe-rust)
- **Location**: `kb/tools/probe-aeneas.md`, “External tool dependencies”, Charon row (approx. line 173); `kb/tools/probe-rust.md`, same table (approx. line 102)
- **Issue**: probe-aeneas says Charon is “managed by probe-rust `--auto-install`”. probe-rust lists Charon as not auto-installed (`no` / `no`). Operators get conflicting instructions.
- **Recommendation**: Reconcile both tables with actual probe-rust behavior; if Charon is never auto-installed, drop the probe-aeneas note or replace it with the real mechanism (e.g. user-installed, or probe-aeneas pre-generation only).

### [W6] P14 “probe-rust … tracked” has no anchor or tracker
- **Location**: `kb/engineering/properties.md`, P14 (approx. lines 114–115)
- **Issue**: The known non-determinism in probe-rust is acknowledged but not tied to an issue, ADR, or KB section, so “tracked” is unverifiable and may go stale.
- **Recommendation**: Add a decision link, issue link, or explicit KB “known gaps” bullet with acceptance criteria.

### [W7] Property coverage: stub `dependencies` not stated in properties
- **Location**: `kb/engineering/glossary.md` §stub; `kb/engineering/schema.md` §Stubs; `kb/engineering/properties.md` P3
- **Issue**: Glossary and schema say stubs have `dependencies: []`. P3 defines stub detection only via `code-path` and `code-text`. Nothing in P1–P20 requires empty dependencies for stubs, so an extractor could theoretically diverge while still satisfying P3.
- **Recommendation**: Either add a one-line invariant under P3 (or a short P21) that stubs MUST have `dependencies: []`, or soften glossary/schema wording if empty dependencies are illustrative only.

## Info

### [I1] No files exceed 30-day staleness (as of 2026-04-03)
- **Location**: all `last-updated` fields under `kb/`
- **Issue**: None — dates range from 2026-03-19 through 2026-04-03, i.e. newer than 2026-03-04.
- **Recommendation**: None required for the 30-day rule; keep updating dates when normative text changes.

### [I2] ADR-002 example schema value is legacy
- **Location**: `kb/decisions/002-schema-2.0.md` (approx. line 27)
- **Issue**: Rationale uses `probe-lean/atoms` as an example discriminator; `kb/engineering/schema.md` notes current probe-lean outputs use `probe-lean/extract` / `viewify` with legacy values only for older files.
- **Recommendation**: Swap the example to a current schema string to avoid teaching deprecated identifiers.

### [I3] P12 vs schema: `heuristic` confidence
- **Location**: `kb/engineering/properties.md` P12; `kb/engineering/schema.md`, translations / confidence list (approx. line 264)
- **Issue**: Schema lists `heuristic` as a confidence level; P12 enumerates only the four strategy-driven levels. Readers cannot map `heuristic` to a strategy or invariant.
- **Recommendation**: Add a sentence in P12 or schema stating when `heuristic` is produced, or remove it from the schema list if unused.

### [I4] Merge “commutativity for disjoint keys” lacks a property ID
- **Location**: `kb/tools/probe-merge.md`, “Categorical framework” (approx. line 73)
- **Issue**: Associativity and identity are tied to P4/P5; commutativity for disjoint keys is not listed in `properties.md`, yet merge order is emphasized elsewhere (P6 first-wins base).
- **Recommendation**: Add a short property or a non-normative note clarifying that commutativity applies only when there are no overlapping keys (or rephrase if the claim is informal).

### [I5] “Lexical scope” used in P20 / tool docs but not in glossary
- **Location**: `kb/engineering/properties.md` P20; `kb/tools/probe-verus.md` “Language assignment”
- **Issue**: The term is understandable in context but is not a defined glossary entry for consistency audits.
- **Recommendation**: Optional one-line gloss under **kind** or a footnote pointing to P20.

### [I6] Product spec does not distinguish envelope `source.language` from atom `language`
- **Location**: `kb/product/spec.md`; `kb/engineering/schema.md` envelope vs atom tables
- **Issue**: After P20, Verus exec atoms carry `language: "rust"` while the project may still be described at envelope level as Rust; the product layer does not warn consumers not to conflate the two fields.
- **Recommendation**: One short paragraph in the Output / Schema reference path (with links to schema + P20).

### [I7] tools/index LOC/complexity snapshot may drift
- **Location**: `kb/tools/index.md` (table approx. lines 13–19)
- **Issue**: Figures (e.g. probe ~1.5K LOC) are approximate and not tied to a refresh policy; hub growth (`query`, etc.) isn’t reflected.
- **Recommendation**: Treat as indicative only or add “approx / see repo” disclaimer; update when tooling changes significantly.
