---
auditor: code-quality-auditor
date: 2026-06-03
status: 0 critical, 0 warnings, 3 info
---

## Critical

None

## Warnings

None — all warnings from initial audit have been fixed:

- JSON schema `additionalProperties: false` updated to allow optional `projection` field
- JSON schema `inputs.minItems` relaxed from 2 to 1 for single-tool projection
- `@kb:` annotation slug fixed in `project.rs`
- Missing P1/P9/P10 cross-references added to `probe-project.md`
- Glossary updated with projection/seed-set/focus-set terms
- `kb/tools/index.md` updated with probe-project entry

## Info

### [I1] ProjectedEnvelope duplicates MergedEnvelope fields
- **Location**: `src/commands/project.rs`
- **Issue**: Hand-rolled struct rather than extending `MergedEnvelope<Atom>` with a `projection` field
- **Recommendation**: Low priority — consider refactoring if more envelope variants emerge

### [I2] projection.mappings-file stores basename only
- **Location**: `src/commands/project.rs`, `focus_path_from()`
- **Issue**: `file_name()` loses directory context
- **Recommendation**: Acceptable for metadata; consumers should use the CLI invocation for full path

### [I3] Output schema semantics for single-tool input
- **Location**: `cmd_project()` always emits `probe/merged-atoms`
- **Issue**: Semantically imprecise for single-tool input, but intentional for scip-callgraph compatibility
- **Recommendation**: Disambiguated by `tool.command: "project"` — document this in KB
