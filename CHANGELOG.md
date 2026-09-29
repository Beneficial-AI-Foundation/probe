# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- [ADR-006](kb/decisions/006-correspondence-records.md): correspondence records and the verification evidence contract (spec only — implementation lands in follow-up PRs). `probe merge --mappings` is specified to attach `maps-to`/`mapped-from` correspondence records instead of injecting cross-language dependency edges; new `status-origin` marker (`translation`, `kernel-taint`) with blocker-seed semantics; per-producer version gate with reserved contract-release thresholds (probe-lean 0.16.0, probe-aeneas 0.21.0, probe-leanblueprint 0.11.0, probe-vcvio 0.2.0, probe 0.5.0–1.0.0 interval + `merge-atoms` command rejection); `probe/projected-atoms` schema with projection rejection at recomputation boundaries; `probe summary` `imported-verified` list and blueprint-language exclusion. New properties P27 (record union/inertness); new KB page `kb/tools/probe-vcvio.md`.
- §3b producer trust audit (probe-verus, probe-lean, probe-leanblueprint) recorded in ADR-006 Decision 4; whole-atom trust adopted for code atoms with the body/closure split as documented fallback.
- `scripts/check-kb-links.sh` now validates `// @kb:` annotations in source code (file + heading anchor), not just markdown links inside `kb/` (#66).

### Changed
- **Spec**: P23 restated as a single path-based definition (enrichment recomputes rather than upgrades; merge re-enriches; `trusted` is a whole-atom boundary unless marked; missing-status atoms transparent by construction). P4/P5 restated on the carrier with the μ/F_M factoring and the mapping-compatibility law. P13 rewritten from existence-checked edge injection to unconditional record attachment. P8 extended to code-name-bearing extension arrays and mapping endpoints. P15 scoped to atoms carrying categorized subsets and required to survive transformations. P16 Lean status table replaced with the kernel-based evidence contract. P9 defines provenance as a deduplicated source inventory with structural composed detection. Hub schema-version 3.1 (minor, hub-side only).
- Aligned `docs/merge-algorithm.md`, `docs/mappings-spec.md`, `docs/SCHEMA.md`, `docs/categorical-framework.md`, glossary, and all `kb/tools/` pages with ADR-006.
- Single-sourced the interchange docs on the KB (#64): `docs/SCHEMA.md` reduced to a pointer to the normative `kb/engineering/schema.md` plus per-tool delegation and design rationale; `docs/merge-algorithm.md` and `docs/mappings-spec.md` folded into `kb/tools/probe-merge.md` / `kb/engineering/schema.md#mappings-file-format` and archived under `docs/archive/`; `docs/categorical-framework.md` moved to `kb/engineering/categorical-framework.md`; `docs/consumer-guide.md` and all remaining links repointed at the KB.
- `docs/envelope-rationale.md` trimmed to the unique rationale ("Why an Envelope Is Needed", "What the Envelope Should NOT Do"): dropped the "Schema 2.0" title, the stale "bare JSON dictionaries" premise, the drifted Field Reference / Merged Envelope / Package Versioning / Folder Structure duplicates (now KB links), and the completed Rollout section; examples updated from legacy `probe-lean/atoms`/`atomize` to `probe-lean/extract` (#66, docs-report W3).

### Fixed
- `probe merge --mappings` CLI help no longer claims mappings "add cross-language dependency edges"; it now states records are attached per ADR-006. Fixed the stale `P26` reference in `kb/engineering/schema.md` (migrated to probe-leanblueprint by ADR-005).
- `docs/ui-views.md` no longer contradicts the schema contract (#66, docs-report C1): cross-language linkage is described as derived from `maps-to`/`mapped-from` correspondence records (ADR-006), never dependency edges; the status colour mapping covers all five `verification-status` values per the VeriLib two-channel convention (dropping the non-existent `unknown`); the language/kind/status enums are KB links instead of restatements; the Rust/Lean views group `verus` and `blueprint` atoms explicitly; the stub predicate links P3's structural three-condition test; envelope dispatch prefers structural `inputs`-vs-`source` detection over schema-string matching.
- `docs/SCHEMA.md`: "full field reference" now points at `kb/engineering/schema.md#envelope-fields` (envelope-rationale keeps rationale only); "probe-verus = Rust" qualified with the kind→language rule (#66, docs-report I5).
- Dangling `@kb:` anchor in `src/commands/propagate.rs` (`#verification-status-values` → `#common-optional-fields`) (#66, docs-report I4).

## [0.4.0] - 2026-08-04

### Changed
- Applied [ADR-005](kb/decisions/005-doc-ownership-boundary.md) doc-ownership boundary to the KB: the five external-probe docs (`kb/tools/probe-{rust,verus,lean,aeneas,leanblueprint}.md`) are now catalog stubs (role + hub contracts + link to each probe's own normative docs); mechanics live in the probe repos. Removed single-probe invariants P11/P12 (probe-aeneas), P18 (probe-lean), P20 (probe-verus), P26 (probe-leanblueprint) and the resolved probe-aeneas bug records C6/C7/C8 from `kb/engineering/properties.md`, leaving a pointer section; repointed all cross-references and the `@kb:` annotations in `src/`.
- **Breaking**: bumped the interchange `schema-version` to `3.0`. The hub now accepts only `3.x` inputs (`starts_with("3.")` in `types.rs`/`propagate.rs`) and emits `3.0` for merged/summary/project output, unifying every producer. Consumers must update their major-version check from `2.` to `3.`.
- Renamed the atom scope field `is-disabled` to `untracked` across the schema spec, KB (P16/P24/P25), docs, and extract-check golden fixtures (#42). Semantics are unchanged and polarity is preserved: `untracked: true` means out of verification scope, `untracked: false` means in scope (verified atoms plus the spec-less backlog). Producers (`probe-rust`, `probe-verus`, `probe-aeneas`) emit `untracked` accordingly.

### Added
- Progress-tracking scheme in `docs/atoms_roles_statuses.md`: a snapshot summary partition (`tracked = unspecified + failed + in-progress + verified + trusted`) and a burn-up chart of cumulative frontiers (`tracked ≥ translated ≥ verified`, with `verified + trusted` as the completion frontier)
- `scripts/count-colors.sh` now reports the progress summary/chart numbers, a `translated` count (non-disabled `exec` atoms with a `translation-name`, Aeneas-only), and a self-check warning when the `tracked ≥ translated ≥ verified` invariant is violated
- KB tool spec `kb/tools/probe-leanblueprint.md`, ADR-004, and property P26 (blueprint status is additive; machine `verification-status` stays authoritative) for the new `probe-leanblueprint` enricher
- Schema documentation for `probe-leanblueprint/extract` (atoms) and `probe-leanblueprint/summary` (sidecar), the `blueprint-*` extension fields, `language: "blueprint"`, and `blueprint-definition`/`blueprint-theorem` kinds
- Explicit `detect_category()` test coverage for `*/extract` schemas (`probe-leanblueprint/extract`, `probe-aeneas/extract`)

### Fixed
- `schemas/atom-envelope.schema.json`: added `probe-leanblueprint/extract` to the `schema` pattern and `blueprint` to the per-atom `language` enum.
- `schemas/atom-envelope.schema.json`: split the single-tool envelope into an atoms branch (`*/atoms`, `*/enriched-atoms`, `*/extract` → atom dictionary) and a generic branch (`*/specs`, `*/proofs`, `*/stubs`, `*/verification-report` → unconstrained dictionary), and registered `probe-verus/stubs` and `probe-verus/verification-report`.
- Synced hub `docs/SCHEMA.md` with `kb/engineering/schema.md`: version header `2.0` → `3.0`; completed the registered-`schema` table (added `probe-lean/viewify`, `probe-leanblueprint/extract`, `probe-leanblueprint/summary`, `probe/summary`, `probe/mappings`; marked the Schema-1.x `probe-lean/{atoms,enriched-atoms,specs,proofs,stubs}` legacy); added Projection-metadata, Version-history, and Package-versioning sections.
- Per-tool extension-field lists in `docs/SCHEMA.md`, the consumer-guide's `language`/`kind`/optional-field enumerations, and `docs/envelope-rationale.md`'s schema list now link to their source instead of restating it.
- Added `probe-leanblueprint` to the README ecosystem table and the consumer-guide tool listing.

### Removed
- Removed the Lean-specific docs from the hub (per ADR-005 — Lean-specific, not cross-probe): `kb/engineering/lean-verification-landscape.md` moved to the probe-lean repo; `docs/lean-stats-brainstorm.md` and `docs/slides-lean-verification-landscape.md` kept local (brainstorm/presentation material, not published); `docs/web-cli-probe/002_probe_lean.md` deleted (legacy `syncstatus` pipeline).
- Dropped the reserved `probe-latex` tool from the schema spec, KB, docs, and JSON Schema: the `latex` per-atom and `source.language` value, the `latex:` code-name scheme, and the LaTeX kind/package-versioning entries.
- Moved `docs/atoms_roles_statuses.md` and `scripts/count-colors.sh` to the VeriLib engineering docs ([Atom statuses and colours](https://docs.verilib.org/components/processor/atom-statuses-and-colours/)). The colouring scheme is VeriLib-specific (how VeriLib presents atom statuses), not a probe concern; the script is reproduced there as the reference implementation.
- Untracked the remaining VeriLib-specific colour/stats docs (`docs/probes_statuses_colours.md`/`.pdf`, `docs/archive/verification-statuses.md`, `docs/VeriLib_Atom_Proposal.pdf`) and dropped their README links. Colour/stats are orthogonal to the probes; the files are kept locally (gitignored), with the canonical home in Beneficial-AI-Foundation/engineering-docs.

## [0.3.0] - 2026-07-17

### Added
- `probe project` subcommand: extract a focused subgraph from an atom file using cross-language mapping seeds with BFS expansion (separate `--forward-depth` and `--reverse-depth` controls)
- `--emit-focus` flag on `probe project` to produce a companion focus-set JSON compatible with probegraph `?focus=` parameter
- KB tool spec: `kb/tools/probe-project.md`

### Changed
- **BREAKING**: CLI flag `--translations` renamed to `--mappings` for `probe merge`
- **BREAKING**: Schema string `probe/translations` renamed to `probe/mappings`
- **BREAKING**: Public types renamed: `TranslationMapping` → `Mapping`, `TranslationsFile` → `MappingsFile`, `load_translations()` → `load_mappings()`
- `probe merge` now supports 1-to-many mappings: a single `from` key can map to multiple `to` targets
- Terminology: generic cross-language linking concept renamed from "translation" to "mapping" across KB, docs, and code; "translation" retained for Aeneas-specific transpilation context
- `enrich_verification_status` now distinguishes benign references to constructors/fields of extracted types (`inductive`/`structure`/`class`) from genuine orphan dependencies. Type-member references are collapsed into a single summary note instead of one "not found in atom map" warning each, so real missing dependencies stay visible. The returned `missing_deps` now lists only genuine orphans.

### Fixed
- C8: Duplicate `from` keys in mapping files no longer silently overwrite (last-wins); all targets are now collected and applied
- Known bugs C6 (RQN collision in probe-aeneas) and C7 (misleading translation-text for 0,0 lines) marked as resolved in properties.md

## [0.2.0] - 2026-05-21

### Added
- `probe enrich` subcommand: walks the dependency graph and upgrades `verification-status` from `"verified"` to `"transitively-verified"` on atoms whose entire transitive closure is verified or trusted, distinguishing transitively verified (Dark Green) from locally verified (Light Green)
- `probe summary` subcommand: partitions verified atoms into entrypoints, verified functions, and verified lemmas (schema `probe/summary`)
- New `verification-status` value `"transitively-verified"` — replaces the previous `transitive-verification-status` key design (never shipped to consumers)
- KB link-checker script (`scripts/check-kb-links.sh`) that validates all cross-references between `kb/` markdown files, including heading anchors
- CI job (`kb-links`) that runs the link checker on every push/PR
- `// @kb:` code-to-spec annotations in `types.rs`, `merge.rs`, `summary.rs`, and `main.rs` linking implementations to their KB sections
- KB discoverability guide in `CLAUDE.md` (searching headings, following `@kb:` annotations, using the glossary)

### Fixed
- Broken link in `kb/engineering/architecture.md`: `properties.md#translation-matching` corrected to `properties.md#p12-translation-strategy-priority`
