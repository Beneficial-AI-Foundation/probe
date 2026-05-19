---
auditor: ambiguity-auditor
date: 2026-05-12
scope: propagate-verification-status / P23 / transitive-verification-status KB consistency
status: 2 critical, 6 warnings, 4 info
---

## Critical

### C1 — `docs/verification-statuses.md` contradicts [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) on what “transitive” means

In the **Verification scope** bullets (lines 41–44), Dark Green is described in proof terms (“sorry-free”) and Light Green correctly mentions dependencies that are **explicitly unverified or failed**. [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) defines **`"transitive"`** via contamination from **only** explicit `"unverified"` / `"failed"` (missing `verification-status` is transparent; missing deps treated as trusted). A verified atom can therefore receive `transitive-verification-status: "transitive"` while depending on atoms with **no** verification status (e.g. plain Rust). Those dependencies are not “sorry-free” in the Verus/Lean sense—they are outside the status model. Readers using the doc’s “sorry-free” wording will misinterpret probe output and UI colors relative to the normative algorithm.

### C2 — Internal inconsistency in `docs/verification-statuses.md` color table vs scope bullets

The **Verification scope** subsection (lines 41–45) defines **Locally-scoped verified** using **explicit** unverified/failed dependencies. The **Color Mapping** table row for **Light Green** (line 49) instead says “transitive dependencies are **not checked**,” which does not match P23 (dependencies without status do **not** force Light Green) and does not match the doc’s own preceding bullet (lines 44–45). Same table: **Dark Green** (line 48) says “all transitive dependencies are sorry-free,” which again conflicts with P23’s transparent missing-status rule.

## Warnings

### W1 — Broken `@kb` fragment for P23 in `src/commands/propagate.rs`

The annotation uses `#p23-transitive-verification-scope`, but the GitHub-compatible slug for the heading **“P23. Transitive verification scope is computed by reverse-BFS contamination”** is `p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination` (see `./scripts/check-kb-links.sh` slug rules). Traceability from code to the precise KB section is therefore wrong if anyone resolves the fragment manually or extends the checker to Rust `@kb` lines.

### W2 — No `@kb` on the propagate subcommand in `src/main.rs`

`PropagateVerificationStatus` documents behavior that is specified by [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) and the [`transitive-verification-status` row](../engineering/schema.md#common-optional-fields) in `schema.md`, but only merge/summary-adjacent `@kb` refs appear at the top of `main.rs`. Adding `@kb` on the propagate variant would align with project traceability expectations.

### W3 — `docs/verification-statuses.md` never ties colors to `transitive-verification-status` or the CLI

The document discusses Dark Green / Light Green and transitive vs local scope but does not mention `probe propagate-verification-status`, the JSON field `transitive-verification-status`, or [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination). Consumers have no KB anchor from this doc to the normative behavior.

### W4 — README propagate comment is incomplete

The comment under “Compute transitive verification status” only illustrates `"transitive"`; it does not state that verified atoms may get `"local"` when contamination reaches them ([README.md](../../README.md) usage block).

### W5 — `kb/engineering/schema.md` version history omits the new field

The optional field and `probe` as producer are documented in the atom table, but the **Version history** subsection still stops at probe-rust 2.1 entries. A minor schema note for `transitive-verification-status` / propagate would reduce ambiguity about when the field appeared.

### W6 — JSON Schema may not describe `transitive-verification-status`

[`schemas/atom-envelope.schema.json`](../../schemas/atom-envelope.schema.json) has no mention of `transitive` (grep). README positions the schema as the machine-readable contract; omitting this probe-added field can confuse validators and downstream tools.

## Info

### I1 — Glossary gaps for propagate vocabulary

[P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) introduces **reverse-BFS contamination**, **`transitive-verification-status`**, and scope values **`transitive`** / **`local`**. [glossary.md](../engineering/glossary.md) defines [trusted (verification-status)](../engineering/glossary.md#trusted-verification-status) but has no entries for transitive scope or the new field; optional gloss entries would align terminology with UI docs (“Dark Green” / “Light Green”).

### I2 — [P23](../engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination) does not cross-link `schema.md`

The property cites only `probe/src/commands/propagate.rs`. A link to the `transitive-verification-status` row in [schema.md](../engineering/schema.md#common-optional-fields) would complete KB-internal cross-references.

### I3 — `docs/verification-statuses.md` uses `null` for verification status

The table lists `null` as a status value; the interchange spec treats absence of `verification-status` as optional. Low ambiguity if readers treat these as equivalent, but the doc does not say that explicitly.

### I4 — KB front-matter dates unchanged

`kb/index.md`, `glossary.md`, `properties.md`, and `schema.md` still show **last-updated** metadata from April 2026; no staleness issue for correctness, but dates do not reflect the propagate feature edit trail.

## Summary table

| Area | Result |
|------|--------|
| Glossary vs P23 / docs terms | Trust base / verification-status covered; **transitive scope / field / algorithm name** not in glossary (info). |
| P23 vs other properties | Aligns with P14 (determinism), P16 (status vocabulary); no merge-rule conflict identified. |
| `schema.md` field vs P23 | Semantics match; version history gap (warning). |
| `docs/verification-statuses.md` vs KB | **Contradictions** on transitive/local meaning (critical). |
| README propagate section | Partial; missing `"local"` mention (warning). |
| `@kb` annotations | Present on `propagate.rs` (fragment wrong); missing on `main.rs` propagate variant; P14 also cited on `propagate.rs`. |
