---
auditor: ambiguity-auditor
date: 2026-03-27
status: 0 critical, 5 warnings, 4 info
scope: is-public visibility feature (probe-rust + probe-aeneas)
---

## Critical

None.

## Warnings

### [W1] Stale normative schema spec (`engineering/schema.md`)
- **Location**: Tool-specific extension lists, version history table
- **Issue**: `probe-rust` extensions documented as only `rust-qualified-name`. The ecosystem schema changelog for 2.1 lists `rust-qualified-name` and `is-disabled` but not `is-public`. Upstream `probe-rust/docs/SCHEMA.md` already documents `is-public`.
- **Recommendation**: Add `is-public` to the probe-rust extension bullet. Extend the 2.1 version-history row so KB and per-tool SCHEMA stay aligned.

### [W2] Stale tool doc (`tools/probe-rust.md`)
- **Location**: Extract step 7, `--with-charon` flag, schema differences section
- **Issue**: Charon enrichment described only as adding `rust-qualified-name`. `is-public` is omitted everywhere.
- **Recommendation**: Update step 7 and the flag row to include `is-public`. Add a schema-differences bullet.

### [W3] Missing enrichment and pipeline semantics (`tools/probe-aeneas.md`)
- **Location**: "Enrichment" section
- **Issue**: Enrichment table does not list `is-public`. Intro says fields are added to atoms "that have translations," but `is-disabled`, `is-relevant`, and `is-public` are set for all Rust atoms.
- **Recommendation**: Split narrative into translation-only fields vs all-Rust-atoms fields. Add `is-public` to enrichment table.

### [W4] Stale architecture summary (`engineering/architecture.md`)
- **Location**: probe-aeneas pipeline step 6
- **Issue**: Enrichment listed as `translation-name`, `translation-path`, `translation-text`, `is-disabled` only — omits `is-public` and `is-relevant`.
- **Recommendation**: Extend the bullet to include `is-public` and `is-relevant`.

### [W5] Incomplete Charon glossary entry (`engineering/glossary.md`)
- **Location**: Charon definition
- **Issue**: Charon described as producing FQNs only. Now also supplies visibility via LLBC `attr_info.public`.
- **Recommendation**: Add clause mentioning visibility enrichment.

## Info

### [I1] Undefined term: `is-public` in glossary
- **Location**: `engineering/glossary.md`
- **Issue**: `is-public` not defined as a glossary term. Not strictly required but improves discoverability.
- **Recommendation**: Add concise entry or fold into Charon/extensions entries.

### [I2] Property coverage — no dedicated invariant
- **Location**: `engineering/properties.md`
- **Issue**: `is-public` not named. P10 and P14 cover it implicitly. Default-to-false behavior not stated as a property.
- **Recommendation**: No new property mandatory unless normative consumer contract desired.

### [I3] Envelope minor version vs atom fields
- **Location**: architecture.md / probe-aeneas.md
- **Issue**: Pre-existing: merged envelope says 2.0 while Rust atoms carry 2.1-era optional fields. `is-public` is another such field.
- **Recommendation**: Document if KB ever covers envelope vs atom field versioning.

### [I4] KB lags per-repo docs
- **Location**: probe-aeneas/probe-rust `docs/SCHEMA.md` already updated for `is-public`
- **Issue**: KB falls behind repository-level documentation for the same feature.
- **Recommendation**: Treat KB updates as part of the same change set.
