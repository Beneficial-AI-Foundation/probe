---
auditor: ambiguity-auditor
date: 2026-05-20
scope: verification status docs consistency (verification-statuses.md, SCHEMA.md, KB schema/properties)
status: 3 critical, 8 warnings, 5 info
---

## Critical

### [C1] Color Mapping scope bullets contradict P23 on transitive meaning

- **Location**: `docs/verification-statuses.md`, lines 50–53 (Verification scope bullets under Color Mapping)
- **Issue**: **Transitively verified** is defined as “sorry-free AND all its transitive dependencies are also sorry-free.” [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) defines `"transitive"` via reverse-BFS contamination seeded only from explicit `"unverified"` / `"failed"`. Atoms with **missing** `verification-status` (untracked/Grey, e.g. plain Rust or Verus spec functions) are **transparent** and do not block `"transitive"`. Dependencies absent from the atom map are treated as trusted (with a warning). A verified atom can therefore receive `transitive-verification-status: "transitive"` while depending on code that is not “sorry-free” in the Verus/Lean sense. The bullets were not updated in the recent docs pass and still conflict with the new **Transitive Verification Status** subsection (lines 28–35).
- **Recommendation**: Rewrite the scope bullets to match P23: Dark Green = verified with no contamination from explicit `"unverified"` / `"failed"` in the dependency closure; Light Green = verified but contaminated by at least one such dependency. Drop “sorry-free” as the defining criterion for transitive scope.

### [C2] Framework-specific color tables contradict P23 and the new subsection

- **Location**: `docs/verification-statuses.md`, lines 107–121 (Verus), 127–142 (Lean), 161–170 (Aeneas)
- **Issue**: Tables map Dark Green to “all transitive deps **sorry-free**” and Light Green to “transitive deps **not checked**.” “Not checked” implies any uninspected dependency forces Light Green; P23 only forces `"local"` when a dependency is **explicitly** `"unverified"` or `"failed"`. Missing status and missing map entries do **not** force Light Green. These tables contradict both P23 and the doc’s own **Transitive Verification Status** table (which correctly uses “explicitly unverified or failed” for `"local"`).
- **Recommendation**: Align framework tables with P23 wording (explicit contamination vs transparent/missing deps), or replace color conditions with direct references to `transitive-verification-status: "transitive"` / `"local"`.

### [C3] `"transitive"` shorthand contradicts P23 algorithm rules across normative docs

- **Location**: `docs/verification-statuses.md` line 34; `docs/SCHEMA.md` line 181; `kb/engineering/schema.md` line 149; `kb/engineering/properties.md` lines 211–212
- **Issue**: All four files describe `"transitive"` as “all transitive dependencies are verified or trusted” (or equivalent). P23’s **Key rules** (lines 217–219) state that missing `verification-status` is **transparent** (does not block transitive) and missing map entries are **treated as trusted** — neither requires deps to carry `"verified"` or `"trusted"`. A dependency with no status is technically neither verified nor trusted, yet per the algorithm it does not prevent `"transitive"`. The high-level bullet at P23 line 211 and the `"local"` bullet at line 212 (“not verified and not trusted”) suffer the same imprecision. Readers applying the shorthand will misread probe output when verified atoms depend on untracked Rust helpers.
- **Recommendation**: Replace the shorthand with algorithm-accurate definitions, e.g. `"transitive"` = verified and no explicit `"unverified"` / `"failed"` in the transitive dependency closure (missing status transparent; missing map entries treated as trusted). Update P23 bullets, both schema tables, and the verification-statuses subsection in one pass.

## Warnings

### [W1] P16 documents probe-lean `trusted` but not probe-verus `trusted`

- **Location**: `kb/engineering/properties.md`, lines 127–147 (P16)
- **Issue**: P16 maps Verus output only to `"verified"`, `"failed"`, and `"unverified"`. It does not state when probe-verus emits `"trusted"` or `"trusted-reason"`. [P22](../engineering/properties.md#p22-cross-tool-trust-reason-vocabulary), `kb/engineering/schema.md`, `docs/SCHEMA.md`, and the glossary all treat `"trusted"` as a probe-verus output value. P16 is incomplete for probe-verus trust-base classification.
- **Recommendation**: Extend P16 with a probe-verus row or subsection for `"trusted"` (e.g. `admit`, `#[verifier::external_body]`, `assume_specification`), cross-referencing P22 for `trusted-reason` values.

### [W2] `docs/verification-statuses.md` is not linked from the KB or README

- **Location**: `kb/index.md`, `README.md`, `kb/engineering/schema.md`, `docs/CONSUMER_GUIDE.md`
- **Issue**: `docs/SCHEMA.md` is linked from README and CONSUMER_GUIDE; `kb/engineering/schema.md` points to per-tool SCHEMA docs. `docs/verification-statuses.md` — the human-facing color/status reference — appears only in auditor reports. No KB entry or index link connects UX language (Dark Green / Light Green) to the normative field definitions.
- **Recommendation**: Add a link from `kb/index.md` (product or engineering section), from `docs/SCHEMA.md` Common Optional Fields (for `verification-status` / `transitive-verification-status`), and optionally from `docs/UI-VIEWS.md`.

### [W3] `docs/CONSUMER_GUIDE.md` common optional fields are stale

- **Location**: `docs/CONSUMER_GUIDE.md`, lines 104–109
- **Issue**: Lists `verification-status` values as `"verified"`, `"failed"`, `"unverified"` only — omits `"trusted"`. Does not mention `trusted-reason` or `transitive-verification-status`, both now documented in `docs/SCHEMA.md` and `kb/engineering/schema.md`.
- **Recommendation**: Sync the CONSUMER_GUIDE table with the Common Optional Fields section of SCHEMA.md.

### [W4] `kb/tools/probe-verus.md` omits `trusted` from verification-status

- **Location**: `kb/tools/probe-verus.md`, line 29
- **Issue**: Output field list gives `"verified"`, `"failed"`, `"unverified"` only. Contradicts schema docs and P22, which document probe-verus `trusted-reason` values.
- **Recommendation**: Add `"trusted"` and `trusted-reason` to the probe-verus output field list with a pointer to P22.

### [W5] KB `last-updated` dates stale for scope files

- **Location**: `kb/engineering/schema.md` (2026-04-07), `kb/engineering/properties.md` (2026-04-07), `kb/engineering/glossary.md` (2026-04-07)
- **Issue**: All three are more than 30 days before 2026-05-20 (cutoff 2026-04-20). They predate P23, `transitive-verification-status`, and the recent docs sync. Other KB files (index entries, tool docs, decisions) are similarly older than 30 days.
- **Recommendation**: Bump `last-updated` on schema.md, properties.md, and glossary.md when those files are next edited for the findings above.

### [W6] JSON Schema does not describe new optional fields

- **Location**: `schemas/atom-envelope.schema.json`; referenced from `docs/SCHEMA.md` line 465
- **Issue**: No definitions for `transitive-verification-status`, `trusted-reason`, or `"trusted"` as a `verification-status` value. SCHEMA.md positions the JSON Schema as the machine-readable contract.
- **Recommendation**: Add optional property definitions (or a documented `$defs` extension block) for hub-computed and trust-base fields.

### [W7] New subsection lacks cross-links to normative KB

- **Location**: `docs/verification-statuses.md`, lines 28–35
- **Issue**: The **Transitive Verification Status** subsection correctly names `probe propagate-verification-status`, reverse-BFS contamination, and field values — an improvement over the prior audit. It still has no links to [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) or the `transitive-verification-status` row in [schema.md](../engineering/schema.md).
- **Recommendation**: Add markdown links so human-facing docs anchor to the normative spec.

### [W8] Color Mapping table rows still disagree with scope bullets and P23

- **Location**: `docs/verification-statuses.md`, lines 57–58 (Color Mapping table)
- **Issue**: Dark Green row: “no transitive dependency is explicitly unverified or failed” — **aligns** with P23. Light Green row: “at least one transitive dependency is explicitly unverified or failed” — **aligns** with P23. But lines 50–53 scope bullets and framework tables (C1, C2) use different criteria. Internal inconsistency within the same document persists after the partial update.
- **Recommendation**: Harmonize scope bullets and framework tables with the Color Mapping table rows (which are already P23-accurate for explicit contamination).

## Info

### [I1] Glossary gaps for terms used in changed doc sections

- **Location**: `kb/engineering/glossary.md`
- **Issue**: Defines [trusted (verification-status)](../engineering/glossary.md#trusted-verification-status) and [trust base](../engineering/glossary.md#trust-base) (covers `trusted-reason` values). Missing entries for: `transitive-verification-status`, `"transitive"` / `"local"` scope values, reverse-BFS contamination, and the UX terms **specification status** (`specified` / `unspecified`) used in `docs/verification-statuses.md` lines 37–44.
- **Recommendation**: Add concise glossary entries for transitive scope vocabulary; optionally add specification-status if it remains a doc-level concept rather than a stored field.

### [I2] `null` verification status vs JSON absence

- **Location**: `docs/verification-statuses.md`, lines 26, 42, 64
- **Issue**: Table lists `null` as a verification status value. Interchange spec (`docs/SCHEMA.md`, `kb/engineering/schema.md`) uses **field absence** (“Absent when verification was skipped”), not the string or JSON null. Low ambiguity if readers treat them as equivalent, but the doc never states that explicitly.
- **Recommendation**: Add a note that `null` in this document means the field is omitted from atom JSON, not a literal `"null"` value.

### [I3] Verus attribute name inconsistency

- **Location**: `docs/verification-statuses.md` lines 25, 63, 112; `docs/SCHEMA.md` line 179 vs `kb/engineering/glossary.md` lines 135–136
- **Issue**: Docs use `#[verifier::external]`; glossary and P22 use `#[verifier::external_body]` and `"external-body"` as `trusted-reason`. May reflect shorthand vs exact attribute name.
- **Recommendation**: Standardize on `external_body` / `external-body` in docs that describe probe output, or note both forms if Verus accepts aliases.

### [I4] P23 does not cross-link schema field row

- **Location**: `kb/engineering/properties.md`, lines 207–223
- **Issue**: P23 cites implementation in `probe/src/commands/propagate.rs` but not the `transitive-verification-status` row in [schema.md](../engineering/schema.md#common-optional-fields).
- **Recommendation**: Add a “Specified in” link to the schema common optional fields table.

### [I5] `kb/engineering/schema.md` omits absence note for `verification-status`

- **Location**: `kb/engineering/schema.md` line 147 vs `docs/SCHEMA.md` line 179
- **Issue**: Per-tool SCHEMA.md notes `verification-status` is absent when verification was skipped; KB schema.md does not. Minor asymmetry between reference docs.
- **Recommendation**: Add the absence note to the KB schema table for parity.

## Consistency summary

| Question | `verification-statuses.md` | `SCHEMA.md` | `kb/schema.md` | `kb/properties.md` | Verdict |
|----------|---------------------------|-------------|----------------|--------------------|---------|
| `verification-status` values | verified, unverified, failed, trusted, **null** (UX) | verified, failed, unverified, trusted; absent when skipped | verified, failed, unverified, trusted | P16: Verus → verified/failed/unverified; Lean → + trusted | **Gap**: P16 missing Verus trusted; null vs absent (W1, I2) |
| `transitive-verification-status` values | transitive, local | transitive, local | transitive, local | transitive, local (P23) | **Agree** on enum |
| `trusted-reason` semantics | (not in changed sections) | Present only when trusted; verus/lean value lists | Same | P22 canonical mapping | **Agree** |
| When `transitive-verification-status` present | Only on `verification-status: "verified"` | Only on verified atoms | Only on verified atoms | Only verified atoms (P23) | **Agree** |
| Semantics of `"transitive"` / `"local"` | Subsection + color sections mixed | Shorthand “all deps verified/trusted” | Same shorthand | P23 bullets vs key rules | **Contradictions** (C1–C3, W8) |

## Cross-reference map

| Document | Linked from |
|----------|-------------|
| `docs/SCHEMA.md` | README, CONSUMER_GUIDE, `kb/engineering/schema.md` |
| `docs/verification-statuses.md` | Auditor reports only — **should be** linked from KB index, SCHEMA.md, UI-VIEWS.md (W2) |
