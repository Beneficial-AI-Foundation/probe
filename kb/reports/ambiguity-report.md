---
auditor: ambiguity-auditor
date: 2026-06-03
status: 0 critical, 0 warnings, 3 info
---

## Critical

None — critical items from initial audit have been fixed:

- JSON schema now defines `projection` as an optional field (no more `additionalProperties: false` contradiction)
- JSON schema `inputs.minItems` relaxed to 1 for projection use case

## Warnings

None — all warnings from initial audit have been addressed:

- Glossary updated with `projection`, `seed set`, `focus-set` definitions
- `kb/tools/index.md` updated with probe-project entry
- P1/P9/P10 cross-references added to `probe-project.md`
- Input restriction (atoms-only) documented in probe-project.md Properties section

## Info

### [I1] Mappings file dual role not distinguished in glossary
- **Issue**: Same `probe/mappings` file used for seed selection (project) and edge injection (merge); different semantics
- **Recommendation**: Low priority — the tool docs explain each use case

### [I2] product/spec.md does not mention projection capability
- **Issue**: New subcommand absent from user-facing product spec
- **Recommendation**: Update when product spec is next revised

### [I3] Dependency trimming scope vs P15
- **Issue**: Only `dependencies` trimmed; categorized dep arrays in extensions untouched
- **Recommendation**: Document as intentional — extensions are opaque passthrough per P10
