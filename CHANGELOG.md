# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **BREAKING**: CLI flag `--translations` renamed to `--mappings` for `probe merge`
- **BREAKING**: Schema string `probe/translations` renamed to `probe/mappings`
- **BREAKING**: Public types renamed: `TranslationMapping` → `Mapping`, `TranslationsFile` → `MappingsFile`, `load_translations()` → `load_mappings()`
- `probe merge` now supports 1-to-many mappings: a single `from` key can map to multiple `to` targets
- Terminology: generic cross-language linking concept renamed from "translation" to "mapping" across KB, docs, and code; "translation" retained for Aeneas-specific transpilation context

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
