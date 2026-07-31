# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- `--emit-focus` flag on `probe project` to produce a companion focus-set JSON compatible with scip-callgraph `?focus=` parameter
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
