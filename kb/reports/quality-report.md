---
auditor: code-quality-auditor
date: 2026-05-20
scope: docs/verification-statuses.md, docs/SCHEMA.md (doc updates for P23 and trusted status)
status: 1 critical, 1 warning, 4 info
---

## Critical

### C1 — `docs/verification-statuses.md` Color Mapping still contradicts P23 and the new Transitive Verification Status subsection

The newly added **Transitive Verification Status** table (lines 28–35) correctly documents P23 semantics: `"transitive"` when all reachable deps are verified or trusted, `"local"` when at least one dep is **explicitly** `"unverified"` or `"failed"`, computed by `probe propagate-verification-status` via reverse-BFS contamination, only on verified atoms. This matches `kb/engineering/schema.md` (lines 147–149), **P23**, and `src/commands/propagate.rs`.

However, the **Color Mapping → Verification scope** bullets immediately below (lines 50–53) still define **Transitively verified** as “sorry-free AND all its transitive dependencies are also sorry-free.” Per P23, atoms with **missing** `verification-status` (untracked/Grey) are transparent and do **not** block `"transitive"`; missing deps in the atom map are treated as trusted. A verified atom can therefore be `"transitive"` while depending on plain Rust functions with no status — those deps are not “sorry-free” in the Verus/Lean sense. This contradicts both P23 and the doc’s own new subsection.

The Color Mapping **table rows** for Dark Green / Light Green (lines 57–58) were updated to the correct “explicitly unverified or failed” wording, but the scope bullets were not, leaving an internal contradiction in the same section.

## Warnings

### W1 — Framework-specific color tables still use pre-P23 phrasing

The **Framework-Specific Behavior** tables (Verus lines 107–121, Lean 127–142, Aeneas 161–170) still map Light Green to “transitive deps **not checked**” and Dark Green to “all transitive deps **sorry-free**.” That phrasing implies dependency-status inspection rather than P23’s contamination model (only explicit `"unverified"` / `"failed"` force `"local"`; missing status and missing deps do not). These tables were not updated in this pass and remain inconsistent with the new Transitive Verification Status subsection and with `kb/engineering/properties.md#p23`.

## Info

### I1 — Changed sections align with KB and implementation (pass)

**`docs/SCHEMA.md` Common Optional Fields** (lines 178–181): The three updated/added rows match `kb/engineering/schema.md` (lines 147–149) verbatim in field names, types, producers, allowed values, and semantics. `verification-status` now lists `"trusted"`; `trusted-reason` values match **P22** (`"admit"`, `"external-body"`, `"assume-specification"` for probe-verus; `"axiom"`, `"external"` for probe-lean); `transitive-verification-status` correctly names `"transitive"` / `"local"`, producer `probe`, and command `probe propagate-verification-status`.

**Staleness check (changed sections only)**: Command name matches Clap subcommand `PropagateVerificationStatus` → `propagate-verification-status` in `src/main.rs`. Field names match `extensions` keys in `src/commands/propagate.rs`. No version or schema-name references appear in the changed table rows; no staleness found there.

**Implementation match**: Documented behavior in both changed files matches `propagate_verification_status()` — reverse-BFS over verified atoms, contamination seeded only from `"unverified"` / `"failed"`, `"trusted"` and missing status transparent, missing map entries treated as trusted with warnings, `"transitive-verification-status"` set only on `"verified"` atoms.

### I2 — Glossary gaps for transitive scope vocabulary

`kb/engineering/glossary.md` defines [trusted (verification-status)](../engineering/glossary.md#trusted-verification-status) and trust-base concepts but has no entries for `transitive-verification-status`, `"transitive"` / `"local"` scope values, or “reverse-BFS contamination.” The new docs use `"transitive"` / `"local"` consistently with P23 and the schema table; glossary coverage would help cross-doc terminology alignment.

### I3 — New subsection lacks KB cross-links

The Transitive Verification Status subsection names the CLI command and field but does not link to `kb/engineering/properties.md#p23` or the `transitive-verification-status` row in `kb/engineering/schema.md`. Low severity because content is correct; links would anchor human-facing docs to the normative spec.

### I4 — JSON Schema file still omits new optional fields

`docs/SCHEMA.md` (line 465) references `schemas/atom-envelope.schema.json` as the machine-readable contract. That schema has no definitions for `transitive-verification-status` or `trusted-reason` (grep). The updated Common Optional Fields table is correct; the JSON Schema lag is pre-existing documentation drift unrelated to this edit’s table content.

## Summary

| Check | Result |
|-------|--------|
| `docs/SCHEMA.md` changed rows vs `kb/engineering/schema.md` | **Pass** — no contradictions |
| `docs/verification-statuses.md` new subsection vs P23 / `propagate.rs` | **Pass** — correct values, field names, command, algorithm name |
| `trusted` / `trusted-reason` vs P22 and glossary | **Pass** |
| Command / field staleness in changed sections | **Pass** |
| Internal doc consistency (`verification-statuses.md`) | **Fail** — Color Mapping scope bullets and framework tables still contradict P23 (C1, W1) |
| Glossary terminology | **Info** — transitive scope terms not yet in glossary |
