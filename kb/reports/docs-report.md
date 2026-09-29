---
auditor: docs-auditor
date: 2026-09-28
repo: probe (hub)
kb: kb/ (own KB)
scope: branch la/docs-single-source @ 980614e (post docs-single-sourcing, PR #65); audited docs/, README, CHANGELOG, probe-extract-check/TESTING.md, and untracked artifacts. kb/ spec content itself is covered by the three KB auditors and was used as ground truth here, except where code was the ground truth.
status: 3 critical, 11 warnings, 6 info
---

Docs verified claim-by-claim against `src/`, `probe-extract-check/`,
`.github/workflows/`, `schemas/`, sibling repos under `../`, and git history.
This audit ran immediately after the single-sourcing restructure (PR #65), which
already resolved the `docs/SCHEMA.md` / merge-algorithm / mappings-spec drift
class; findings below are what remains.

## Critical

### [C1] `docs/ui-views.md` contradicts the schema contract in ways that break a UI built from it
- **Location**: docs/ui-views.md:16, 42-48, 56-59, 89-90, 207-214, 288
- **Issue**: Five independent contradictions with `kb/engineering/schema.md`:
  (1) lines 56-59 and 288 still say cross-language "mapping edges" connect atoms
  in merged output — ADR-006's explicitly rejected design; line 31 of the same
  file was updated to correspondence records, so the file self-contradicts.
  (2) line 89-90 colors by `green/red/grey/blue = verified/failed/unverified/unknown`
  — `"unknown"` is not a status, and `trusted` / `transitively-verified` (the
  enrichment output a UI most needs to distinguish) have no color; same gap in
  the filter table at 207-214. (3) line 16 gives the `language` set as
  rust/lean/verus — missing `blueprint`, so a partition built on it drops every
  probe-leanblueprint synthetic atom. (4) lines 42-44 put `proof`/`spec` kinds
  under the Rust column while line 29 defines the Rust view as
  `language == "rust"` — under the kind→language rule those atoms carry
  `language: "verus"`, so the doc's own Rust view hides them. (5) lines 140-142
  and 224-226 define stubs by `code-path == ""` alone; P3 requires all three
  conditions.
- **Evidence**: kb/engineering/schema.md:128 (language set), :134-137 (kinds),
  :139-147 (kind→language), :157 (status enum), :262-265 + src/types.rs:114-118
  (three-condition stub test); kb/decisions/006-correspondence-records.md
  (records, never edges).
- **Recommendation**: Rewrite 56-59/288 to correspondence records; replace the
  enum restatements (16, 42-48, 89-90, 207-214) with KB links, keeping only the
  UI-owned color/toggle mapping extended to all five statuses; fix the stub
  predicate. ui-views.md is now the largest remaining enum-restatement surface
  in `docs/`.

### [C2] `docs/probes-overview-slides.md` presents superseded status semantics as current guidance
- **Location**: docs/probes-overview-slides.md:195, 217-222, 315, 345
- **Issue**: Tracked, orphaned (zero inbound links) July-2026 slide deck. It
  uses the dead field name `disabled` (renamed `untracked` in the breaking 3.0
  rename), proposes a six-value status vocabulary that is not the shipped
  five-value contract, and states the rule "any function that doesn't have a
  spec is disabled" — the exact opposite of P24/P25 (spec-less in-scope
  functions are `untracked: false`, the tracked backlog). Its colour/status
  half (161-365) is the material CHANGELOG 0.4.0 records as deliberately moved
  to the VeriLib engineering docs; this deck is the last tracked survivor of
  that sweep. Line 345 links the proposal doc only via a pinned-SHA URL; the
  live successor is docs.verilib.org.
- **Evidence**: kb/engineering/properties.md:240,258 (P24/P25);
  kb/engineering/schema.md:382 (is-disabled→untracked); CHANGELOG.md:46-47.
- **Recommendation**: Archive with an outcome header (colour scheme moved to
  VeriLib docs; status vocabulary superseded by properties.md). Do not keep as
  a live doc — a reader cannot tell it is a snapshot.

### [C3] Tracked KB files reference untracked working files — dangling on the forge today
- **Location**: kb/decisions/006-correspondence-records.md:12-13;
  kb/reports/ambiguity-report.md:16,65
- **Issue**: ADR-006 cites `merge-soundness-review.md` and
  `merge-soundness-fix-plan.md` by name "in the hub repo"; the tracked
  ambiguity report cites the plan's §9 and has an open finding (W2) against
  `docs/lightning-talk-probes.md` / `docs/slides23-31.md`. All of these are
  untracked. The live fix plan's own front matter points at
  `docs/archive/merge-soundness-fix-plan-full.md` — also untracked, inside an
  otherwise-tracked directory. Anyone reading ADR-006 on GitHub cannot follow
  its provenance citations.
- **Evidence**: `git status --porcelain` vs `git grep -n merge-soundness`
  (kb/decisions/006:12-13, kb/reports/ambiguity-report.md:16).
- **Recommendation**: Track `merge-soundness-fix-plan.md`,
  `merge-soundness-review.md`, and `docs/archive/merge-soundness-fix-plan-full.md`
  (the plan is live and drives the open PR train). For W2's slide files, either
  track them or move them to engineering-docs and amend the finding.

## Warnings

### [W1] ADR-006 spec/code inversion — merge-order constraint, not new drift
- **Location**: kb/engineering/schema.md:93,206; src/main.rs:37-42 vs
  src/commands/merge.rs:131-166; src/types.rs:163-177
- **Issue**: The KB (and, since PR #65, the `--mappings` CLI help) describe
  correspondence-record attachment and `probe/projected-atoms` category
  detection; the code still injects dependency edges
  (`atom.dependencies.insert`, stat label "Cross-lang edges") and
  `detect_category` has no `probe/projected-atoms` arm. This is the known
  ADR-006 "spec only — implementation lands in follow-up PRs" state
  (CHANGELOG Unreleased; merge-soundness plan PRs 3a/3b), not undocumented
  drift — but nothing in schema.md marks those two statements as pending, and
  the CLI help now describes unimplemented behavior to end users.
- **Recommendation**: Land PRs 3a/3b per the plan. If they are more than a
  release away, add a "pending implementation (ADR-006 rollout)" marker at
  schema.md:93 and in the CLI help.

### [W2] `probe project` is invisible in every user-facing doc
- **Location**: README.md:9,48-64; docs/consumer-guide.md; docs/testing-guide.md:132-148
- **Issue**: The clap parser ships four subcommands (`merge`, `project`,
  `enrich`, `summary`; src/main.rs:20-122) and `probe project` has a KB page,
  but README says "(`merge`, `enrich`, `summary`)", the Usage block has no
  project example, and the testing guide lists none of its 14 unit tests.
- **Recommendation**: Add `probe project` to README (one usage line:
  `--mappings` required, `--forward-depth` default 2, `--reverse-depth`
  default 0, `-o` default `projected.json`, `--emit-focus`) and to the
  consumer guide's progress section.

### [W3] `docs/envelope-rationale.md` is written as an unshipped proposal and uses legacy schemas in its flagship examples
- **Location**: docs/envelope-rationale.md:1,10-20,91,95,141,202-230,266,284-293
- **Issue**: Title still says "Schema 2.0"; "Today these files are bare JSON
  dictionaries" (shipped years ago); canonical examples use the Schema-1.x
  legacy `probe-lean/atoms` + `atomize`; the "Proposed layout" names a
  `translations/` folder where the KB specifies `.verilib/mappings/`; the
  Rollout section ("only consumer is verilib-cli, no migration needed") is
  contradicted by the KB's own multi-consumer lockstep-bump runbook. Its Field
  Reference and Merged Envelope sections duplicate kb/engineering/schema.md
  field-for-field and have already drifted (no structural composed-provenance
  detection).
- **Recommendation**: Trim to the genuinely unique rationale ("Why an Envelope
  Is Needed", "What the Envelope Should NOT Do"), retitle without "2.0",
  replace the field reference / merged-variant / package-versioning /
  folder-structure sections with KB links, delete the Rollout section. Repoint
  docs/SCHEMA.md's "full field reference" sentence to the KB.

### [W4] `docs/extract-check-design.md` is a stale lower-resolution copy of TESTING.md
- **Location**: docs/extract-check-design.md:58,76-77,99-107,124,127,132-141
- **Issue**: Every count is wrong (rust_micro 12→15 atoms, 3→5 files;
  integration 25→27 active; total 51→65); it omits the path-safety checks, the
  P14 `dependencies-with-locations` ordering check, and the 4 idempotency
  tests. ~80% duplicates probe-extract-check/TESTING.md, whose versions of the
  same tables are correct.
- **Recommendation**: Move the unique "What 'correct' means" properties table
  (lines 6-18) and the three-layer framing into TESTING.md as a preamble;
  delete the rest; update the two inbound links (README.md, consumer-guide).

### [W5] `probe-extract-check/TESTING.md` undercounts its own suite
- **Location**: probe-extract-check/TESTING.md:24,40,47,68,75,169-178
- **Issue**: Unit counts stale (22 stated, 30 actual — the missing 8 are the
  security-hardening and P14-ordering tests, so the doc under-sells real
  coverage); Layer-2 header "(27 tests, of which 8 are ignored)" is wrong
  twice (23 golden, all active). Every named test resolves correctly — only
  the arithmetic is stale. Exit codes (2 = parse failure, 1 = errors or
  warnings without `--allow-warnings`, 0 = clean) and the `-p` short flag are
  documented nowhere.
- **Recommendation**: Replace hand-maintained totals with "run
  `cargo test -p probe-extract-check`"; keep the per-test semantic tables
  (100% accurate today); add exit codes.

### [W6] `docs/schema-validation.md` claims validation in repos that don't validate
- **Location**: docs/schema-validation.md:6,8,26,51
- **Issue**: Headings claim probe-verus and probe-lean validate against the
  JSON Schema; neither has any jsonschema dependency or reference. "The
  machine-readable contract all probe-* codebases should validate against"
  overstates coverage: `probe/summary`, `probe/mappings`,
  `probe/projected-atoms`, `probe-lean/viewify`, `probe-leanblueprint/summary`
  are not registered in the schema's `oneOf`.
- **Recommendation**: Scope the headings to this repo (the only validator
  today), state the schema's actual coverage, and keep it as the single home
  of the validation recipe (see W7).

### [W7] Schema-validation recipe is triplicated
- **Location**: README.md:66-72 ↔ docs/schema-validation.md ↔ docs/testing-guide.md:17-28
- **Issue**: Three copies of the same recipe; testing-guide additionally points
  readers to a README section that contains no examples.
- **Recommendation**: Canonical home `docs/schema-validation.md`; README keeps
  a one-line pointer; testing-guide links there directly.

### [W8] `docs/testing-guide.md` test inventory is stale and incomplete
- **Location**: docs/testing-guide.md:21-23,132-148,147
- **Issue**: "Schema validation (7 tests)" → 9; "Merge unit (15 tests)" → 22;
  `tests/propagate.rs` (7), `propagate.rs` unit (20), `project.rs` (14),
  `summary.rs` (7), `types.rs` (1) all unlisted; "translation-based edge
  creation" uses pre-0.3.0 terminology and describes the ADR-006-superseded
  behavior; probe-leanblueprint silently missing from the per-repo table.
- **Recommendation**: Same cure as W5 — drop hand-maintained counts, keep the
  semantic descriptions, fix the terminology, note leanblueprint has no
  TESTING.md yet.

### [W9] `docs/consumer-guide.md` example table 404s for probe-lean
- **Location**: docs/consumer-guide.md:123-124,129
- **Issue**: `examples/lean_Curve25519Dalek_0.1.0.json` does not exist in
  probe-lean (its example is `lean_ExampleProject_0.1.0.json`; the
  curve25519 file lives in probe-aeneas), so the GitHub link is dead and "All
  examples use the curve25519-dalek ecosystem" is no longer true. probe-vcvio
  is absent from the tool table (also from README's ecosystem table).
- **Recommendation**: Fix the row, soften the "all examples" claim, decide
  whether draft-status probe-vcvio should be listed.

### [W10] `docs/structure/` trilogy and `docs/web-cli-probe/001_motivation.md` are outcome-less historical proposals
- **Location**: docs/structure/001-003, docs/web-cli-probe/001_motivation.md (all orphaned, zero inbound links, untouched since 2026-03-20)
- **Issue**: 001/002 specify the verilib-cli product (implemented there; hub
  scope violation per ADR-005; `veri-name` never built; `scip:` naming
  reversed to `probe:`; spec-status vocabulary contradicts properties.md).
  003's "Schema of probe outputs" (L50-110) is the highest-risk stale content
  in the repo: pre-envelope bare dicts, `scip:` dependencies, `specified` and
  boolean `verified` fields — a grep for `atoms.json`/`specs.json` lands here
  with no version marker. web-cli-probe/001's 10-item plan is essentially all
  shipped (in verilib-cli, probe-lean viewify, probe-aeneas) or explicitly
  dropped (latex); its sibling 002 was already deleted in the ADR-005 sweep.
- **Recommendation**: Archive all four with outcome headers (003's header must
  name kb/engineering/schema.md as the superseding spec; web-cli-probe/001 may
  simply be deleted to match its sibling).

### [W11] kb/reports contains three reports for a different repo
- **Location**: kb/reports/{ambiguity,quality,test}-report-probe-aeneas.md (untracked)
- **Issue**: Their headers name `repo: probe-aeneas`; they landed here because
  probe-aeneas has no local kb/ and the auditor skills resolved the hub KB as
  the output root. The unsuffixed tracked siblings audit this repo — the
  suffix convention worked, but the files sit in the wrong repo and are
  untracked. One nugget inside: a merge-order constraint ("probe#58 before
  probe-aeneas 0.20.0 conformance").
- **Recommendation**: Move them to probe-aeneas (create kb/reports/ there or
  give the skill an output-dir override); put the merge-order note on the
  issue if still open.

## Info

### [I1] Repo-root scratch artifacts (~19 MB) neither tracked nor ignored
- **Location**: test.json (5.7 MB merged output), summary_curve25519-dalek_4.1.3.json (schema 2.0), dots.pdf, "Signal Shot Launch".pptx/pdf ×3 (one 0-byte), scripts/__pycache__/, .vscode/
- **Recommendation**: Delete or move out; add `*.json` scratch patterns /
  `.vscode/` / `__pycache__/` to .gitignore as appropriate.

### [I2] Orphaned figures in docs/assets
- **Location**: tracked atom-bar.png, atom-dot.png, shapes-roles.png; untracked division.{mmd,png}, pipeline.{mmd,png}
- **Issue**: Zero inbound references from any markdown, tracked or untracked.
  The untracked pair likely belongs with the untracked slide decks.
- **Recommendation**: Move with the decks or delete; `division.mmd`'s
  facts-vs-palette split is drawn nowhere else if that idea still matters.

### [I3] `docs/probe-dispatch-plan.md` (untracked) holds unique design content
- **Issue**: Marker-file dispatch design, the live Schema 2.0 (verilib
  atomizer) vs 3.0 (hub) skew, and deployment-gap observations exist nowhere
  else. The doc prescribes its own home (`kb/tools/probe-dispatch.md` + ADR).
- **Recommendation**: Track it (or extract the skew note and open questions
  into an issue) — it is a plan, and plans rot fastest untracked.

### [I4] Dangling `@kb:` anchor
- **Location**: src/commands/propagate.rs:64
- **Issue**: `// @kb: kb/engineering/schema.md#verification-status-values` —
  no such heading (the values live in § Common optional fields).
  `check-kb-links.sh` does not validate `@kb:` anchors.
- **Recommendation**: Fix the anchor; optionally teach the script to check
  `@kb:` references.

### [I5] README copy-paste gap and imprecision in docs/SCHEMA.md
- **Location**: README.md:49-53; docs/SCHEMA.md:84-86
- **Issue**: README shows `cargo build` then bare `probe merge …` with no
  install step. SCHEMA.md's "probe-verus = Rust" elides the kind→language
  rule (the next sentence saves it).
- **Recommendation**: One-line fixes.

### [I6] Session review artifacts at root are fully applied
- **Location**: docs-probe-consistency-review.md, schema-vs-kb-audit.md, pr-63-review-comments.md (untracked)
- **Issue**: Their recommendations landed in PR #65 / PR #63 (verify the
  remaining pr-63 comment items were posted before discarding).
- **Recommendation**: Delete after that check.

## Overlap map (topic × canonical home)

| Topic | Canonical | Duplicates to fold/link |
|-------|-----------|-------------------------|
| Envelope fields, merged variant | kb/engineering/schema.md | envelope-rationale.md §Field Reference, §Merged Envelope (W3) |
| Status/kind/language enums | kb/engineering/schema.md | ui-views.md (C1); probes-overview-slides (C2, archive) |
| Schema-validation recipe | docs/schema-validation.md | README §JSON Schema, testing-guide §Schema validation (W7) |
| extract-check tests | probe-extract-check/TESTING.md | extract-check-design.md (W4, fold+delete) |
| Merge algorithm / mappings format | kb (done in PR #65) | — |

## Proposed target structure

`docs/` live set after executing: `SCHEMA.md`, `consumer-guide.md`,
`envelope-rationale.md` (trimmed ~60%), `ui-views.md` (corrected),
`schema-validation.md`, `testing-guide.md` — 6 reference docs, none normative.
`docs/archive/`: +5 (structure ×3, web-cli-probe/001 or delete,
probes-overview-slides), each with an outcome header.
`extract-check-design.md` deleted after folding into TESTING.md.
Tracked additions: merge-soundness plan + review + archived full plan,
probe-dispatch-plan (or issue). Net: 15 tracked docs/ files → 6 live + archive.

## Structural gaps

1. No drift guard: nothing in CI fails when an enum is restated outside the KB.
   A grep-based check (status/kind/language enum strings appear only in
   kb/engineering/schema.md) would have caught C1/C2 at commit time.
2. `probe project` has no user-facing documentation (W2).
3. Exit codes for probe-extract-check documented nowhere (W5).

## Verified clean

- README relative links (13/13 resolve), per-tool doc table, merge/enrich/summary usage lines.
- CHANGELOG: all referenced files/commits verified against history; dangling paths are accurate history, internally cross-referenced.
- docs/SCHEMA.md post-rewrite: all KB section claims resolve; no restated tables; no stale version.
- consumer-guide: extract commands and flags across all five sibling repos, `.verilib/probes/` default, core-field list vs JSON Schema, stub filter, summary sidecar semantics, probe-extract-check CLI.
- probe-extract-check/TESTING.md: all named tests/fixtures resolve one-for-one (only the totals are stale).
- ui-views.md: field names, code-name formats, `untracked` usage (no `is-disabled` residue), merged schema string.
- kb/tools/ catalog stubs conform to ADR-005 (role + contracts + delegation, no mechanics); P11/P12/P18/P20/P26 migration confirmed (commit 232eb85).
- No case-only filename collisions; no byte-identical duplicate docs.
