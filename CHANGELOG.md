# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Color scheme reworked around one meaning per color, with role and language as separate axes; `scripts/count-colors.sh` implements it per-atom across all pipelines (incl. Lean and merged files) with four role groups broken down by language and a `--per-atom` mode for VeriLib. See `docs/verification-statuses.md` for the full scheme.
- Lean `def`s standing for Rust functions are implementations in every pipeline, each **graded by its own documented primary-spec theorem** (spec attribute or `<def>_spec` naming; probe-lean's sole-spec inference is ignored as too loose) — the same Lean spec probe-aeneas propagates to the Rust exec, read at the source rather than borrowed back from the exec's color. An Aeneas-generated `def` (`rust-source`) without a spec is Yellow, so probe-lean-only extracts of Aeneas code color like the merged view. An `exec` and its translation can now show different green shades when probe-aeneas's per-node enrichment marks the exec `transitively-verified` but the spec theorem only `verified`
- `count-colors.sh` tables count each function once: a Lean stand-in whose exec is itself shown is excluded from the tables and reported in a footnote (`--per-atom` still emits both atoms for VeriLib); empty groups print no table, so a generic Lean project shows only Proofs and Definitions
- `count-colors.sh` emits backstop warnings (tables mode) for dangling `translation-name`/`primary-spec` references, unrecognized statuses, status-less proofs (probe-verus#33), and P24/P25 violations; the dead partition warning was removed (the group partition is total by construction) and the translation-target set is built from all execs (incl. hidden) so their translations stay implementations
- `scripts/summarize_extract.py` recognizes the probe-lean `trusted-reason` `"externally_verified"` (previously such atoms vanished from the trust-base sections) and splits unverified atoms by kind (functions / lemmas / definitions), so an unproved Lean theorem is reported as an unverified lemma
- KB: new property P25 (`has-verification-status ⟹ ¬is-disabled`; violated by probe-verus for spec-less trusted atoms, tracked in probe-verus#32), `externally_verified` added to the trust-reason vocabulary (P22, glossary, schema, probe-lean tool doc), P23/glossary note that probe-lean runs enrichment too

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
