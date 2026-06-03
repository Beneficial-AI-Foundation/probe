---
auditor: ambiguity-auditor
date: 2026-06-03
scope: translation→mapping rename consistency (KB + mappings-spec.md, SCHEMA.md, categorical-framework.md)
status: 2 critical, 7 warnings, 4 info
---

## Critical

### [C1] Known bugs C8 still names pre-rename API while P11–P13 use mapping vocabulary

- **Location**: `kb/engineering/properties.md`, line 241 (Known bugs C8); contrast with lines 80–106 (P11–P13) and `kb/tools/probe-merge.md` line 23
- **Issue**: C8 reads “Duplicate translation `from` keys silently overwrite in `load_translations()`.” The rename updated normative properties (P11–P13) and tool docs to **mapping** / `load_mappings()`, and `probe/src/types.rs` implements `load_mappings()` with schema `"probe/mappings"`. The bug inventory still uses the old generic term (“translation”) and the removed function name, contradicting the same file’s property definitions and the merge tool spec.
- **Recommendation**: Rewrite C8 to “Duplicate mapping `from` keys … in `load_mappings()`.” If C8 is fixed in code (per quality-report), remove or mark resolved instead of retaining stale text.

### [C2] `kb/index.md` still describes merge as “translation application”

- **Location**: `kb/index.md`, line 37
- **Issue**: Index entry for probe-merge says “translation application.” `kb/tools/probe-merge.md` (Phase 4), `kb/engineering/glossary.md` (`cross-language mapping`), and `kb/engineering/schema.md` (`--mappings`) all use **mapping application**. The hub index contradicts its own tool and engineering docs on the renamed concept.
- **Recommendation**: Change to “mapping application” (or “cross-language mapping application”).

## Warnings

### [W1] Product spec uses stale compound “translation mappings”

- **Location**: `kb/product/spec.md`, line 45
- **Issue**: “generates Rust↔Lean **translation mappings**” mixes the old generic term with the new one. Line 22 correctly says “cross-language mappings”; line 16 correctly uses “Lean translations” (Aeneas transpilation sense). Line 45 should align with the glossary term.
- **Recommendation**: “generates Rust↔Lean **cross-language mappings** for Aeneas-transpiled projects.”

### [W2] Tools index routes readers to “cross-language translation”

- **Location**: `kb/tools/index.md`, line 29
- **Issue**: “Working on **cross-language translation** or parallel orchestration” uses the old generic sense. Everywhere else in KB the generic concept is **cross-language mapping**.
- **Recommendation**: “Working on **cross-language mapping** or parallel orchestration.”

### [W3] Index probe-aeneas blurb uses ambiguous “three-strategy translation”

- **Location**: `kb/index.md`, line 42
- **Issue**: “three-strategy **translation**” is ambiguous after the rename — could mean the generic linking concept (stale) or the Aeneas `translate` subcommand (valid). `probe-aeneas.md` and `architecture.md` say “three-strategy **translation matching**” or “mapping generation,” which is clearer.
- **Recommendation**: “three-strategy **mapping matching**” or “three-strategy mapping generation.”

### [W4] Known bugs C6–C7 may be resolved but still listed as open

- **Location**: `kb/engineering/properties.md`, lines 239–240
- **Issue**: C6 (`rqn_to_rust.insert()` last-wins) and C7 (misleading `translation-text` for 0,0 lines) are documented as open defects. Quality-report notes both are fixed in probe-aeneas. C7’s `translation-text` field name is correct (Aeneas extension); the bug description itself may be stale. Leaving fixed bugs in “Known bugs” contradicts the implementation state and misleads auditors.
- **Recommendation**: Verify fix status; remove resolved items or move to a “Resolved” subsection with fix references.

### [W5] `last-updated` not bumped on KB files changed in the rename

- **Location**: Frontmatter of modified files still dated 2026-03-19 – 2026-04-07; only `kb/decisions/003-mappings-design.md` shows 2026-06-03
- **Issue**: Git diff shows substantive rename edits in `kb/index.md`, `kb/engineering/architecture.md`, `glossary.md`, `properties.md`, `schema.md`, `tools/probe-aeneas.md`, `tools/probe-merge.md`, and `decisions/index.md`, but their `last-updated` dates predate the rename (some >30 days before 2026-06-03). Stale dates hide which normative files were touched.
- **Recommendation**: Bump `last-updated` to 2026-06-03 (or merge date) on every file whose mapping terminology changed.

### [W6] Doc chain from `docs/SCHEMA.md` still points at stale merge-algorithm / translations-spec

- **Location**: `docs/SCHEMA.md` line 317 → `docs/merge-algorithm.md` lines 158–162, 175
- **Issue**: The three docs read for this audit (`SCHEMA.md`, `mappings-spec.md`, `categorical-framework.md`) are consistent with KB (`--mappings`, `probe/mappings`, `mappings-spec.md`). However, `SCHEMA.md` links to `merge-algorithm.md`, which still documents `--translations <file>`, “translations file,” and a broken link to `translations-spec.md` (renamed to `mappings-spec.md`). KB normative docs no longer mention `--translations`.
- **Recommendation**: Update `merge-algorithm.md` to `--mappings`, `mappings-spec.md`, and “mappings file.” Audit `docs/UI-VIEWS.md` and `docs/CONSUMER_GUIDE.md` for the same stale links (outside KB but in the consumer doc graph).

### [W7] KB schema lacks cross-link to detailed mappings spec

- **Location**: `kb/engineering/schema.md` § Mappings file format (lines 249–269); contrast `docs/SCHEMA.md` lines 319–321
- **Issue**: `docs/SCHEMA.md` links to `mappings-spec.md` for the full format. KB `schema.md` inlines a summary but does not link to `docs/mappings-spec.md` or ADR-003. Readers of the KB may miss confidence levels (`manual`), folder conventions, and generation approaches documented only in the spec doc.
- **Recommendation**: Add “See also: [mappings-spec.md](../../docs/mappings-spec.md)” under § Mappings file format.

## Info

### [I1] Glossary has no entries for `Mapping` / `MappingsFile` types

- **Location**: `kb/engineering/glossary.md`
- **Issue**: Glossary defines `cross-language mapping` as a concept but not the Schema 2.0 types `Mapping`, `MappingsFile`, or the loader `load_mappings()` referenced in `probe-merge.md`. The rename introduced these as the canonical names in code.
- **Recommendation**: Add brief glossary entries pointing to `schema.md#mappings-file-format` and `probe/src/types.rs`.

### [I2] RQN glossary entry still says “translation generation”

- **Location**: `kb/engineering/glossary.md`, line 125
- **Issue**: “Used as the highest-confidence matching strategy in probe-aeneas **translation generation**.” For the generic linking pipeline this is now **mapping generation** (the `translate` subcommand name remains Aeneas-specific).
- **Recommendation**: “mapping generation (via probe-aeneas `translate`).”

### [I3] Dual vocabulary (mapping vs translation-*) is correct but undocumented in one place

- **Location**: `kb/engineering/glossary.md` § extensions (line 69); atom fields in `schema.md` lines 172–174
- **Issue**: Generic cross-language linking → **mapping** / `--mappings` / `probe/mappings`. Aeneas transpilation metadata → **`translation-name`**, **`translation-path`**, **`translation-text`** (unchanged). All remaining “translation” hits in KB grep are Aeneas-correct (see audit table below). A single glossary note explicitly stating this split would prevent future conflation.
- **Recommendation**: Add a one-sentence note under `cross-language mapping`: “Do not confuse with atom extension fields `translation-*`, which describe Aeneas Lean transpilation metadata on individual atoms.”

### [I4] Charon entry uses “translation matching” without Aeneas qualifier

- **Location**: `kb/engineering/glossary.md`, line 121
- **Issue**: “high-confidence **translation matching** (Strategy 1)” — acceptable as Aeneas-domain language, but slightly ambiguous post-rename.
- **Recommendation**: “high-confidence **mapping matching** (Strategy 1 of probe-aeneas `translate`).”

## Translation grep audit (`kb/`)

Every remaining `translation` occurrence was checked. **None** require rename to “mapping” except the items flagged above (C1, C2, W1–W3, I2, I4).

| File | Usage | Verdict |
|------|-------|---------|
| `glossary.md` | Lean translations, `translation-name` extension, Aeneas transpilation, Charon/RQN matching | **Correct** (Aeneas / extension fields) |
| `schema.md` | `translation-name/path/text` extension fields | **Correct** |
| `architecture.md` | “translation matching,” `translation-*` enrichment fields | **Correct** (Aeneas matching + extension names) |
| `probe-aeneas.md` | “translation matching,” Translation-specific fields, `translation-*` | **Correct** |
| `product/spec.md` | “Lean translations” (Q&A), “translation mappings” (line 45) | Line 16 **Correct**; line 45 **Stale** (W1) |
| `properties.md` | C6–C8 bug descriptions | C8 **Stale** (C1); C7 field name **Correct** |
| `lean-verification-landscape.md` | Aeneas translation project category, diagram edge labels, Lean translations | **Correct** |
| `index.md` | “translation application,” “three-strategy translation” | **Stale** (C2, W3) |
| `tools/index.md` | “cross-language translation” | **Stale** (W2) |
| `reports/*.md` | Auditor meta-reports about the rename | **Correct** (audit context) |

## Broken cross-reference check

| Check | Result |
|-------|--------|
| `003-translations-design.md` in KB | **Fixed** → `003-mappings-design.md` in `index.md` and `decisions/index.md` |
| `translations-spec.md` in KB | **None found** |
| `#p11-translation-mapping-is-1-to-1` anchor | **Fixed** → `#p11-mapping-generation-is-1-to-1-probe-aeneas` |
| `#p12-translation-strategy-priority` | **Fixed** → `#p12-mapping-strategy-priority` |
| `--translations` in KB | **None found** (consistent `--mappings`) |
| `probe/translations` schema in KB | **None found** (consistent `probe/mappings`) |

## Consistency summary (mapping rename)

| Concept | glossary | schema (KB) | properties | probe-merge | probe-aeneas | docs (3 read) | Verdict |
|---------|----------|-------------|------------|-------------|--------------|---------------|---------|
| Generic linking term | cross-language mapping | mappings file, `--mappings` | P11–P13 mappings | `--mappings`, `load_mappings()` | cross-language mappings | mappings-spec, categorical-framework | **Agree** |
| Schema value | (via schema.md) | `probe/mappings` | — | — | — | mappings-spec, SCHEMA cross-ref | **Agree** |
| CLI flag | `--mappings` | `--mappings` | — | `--mappings` | delegates to merge | categorical-framework | **Agree** |
| Aeneas atom fields | `translation-name` example | `translation-*` extensions | C7 field name | — | Translation-specific fields | SCHEMA.md extension table | **Agree** (intentional) |
| Hub index wording | mapping | mapping | mapping | mapping application | mapping generation | — | **Gap** (C2, W3) |
| Product wording | cross-language mapping | — | — | — | — | — | **Gap** (W1) |
| Bug inventory API name | — | — | `load_translations()` in C8 | `load_mappings()` | — | — | **Contradiction** (C1) |
| Downstream doc chain | — | no link to mappings-spec | — | — | — | SCHEMA → stale merge-algorithm | **Gap** (W6, W7) |
