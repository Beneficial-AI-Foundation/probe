---
auditor: docs-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB)
scope: main @ ed558b7 (post docs-cleanup — PRs #67/#69/#71/#73 all merged); re-audit after executing the 2026-09-28 report. Audited README, docs/ (live + archive), probe-extract-check/TESTING.md, CHANGELOG [Unreleased], scripts, CI, the tracked merge-soundness plans, and the one untracked artifact. kb/ spec content is the KB auditors' scope and served as ground truth, except where the code was the ground truth.
status: 0 critical, 13 warnings, 11 info
---

Re-audit of the cleaned-up docs, claim-by-claim against `src/`,
`schemas/`, `probe-extract-check/`, `.github/workflows/`, the sibling
repos' `origin/main` under `../`, and git history. The 2026-09-28
report's executed verdicts all held (see Verified clean); everything
below is either newly found, introduced since, or a known carry-over.

## Critical

None. The one critical-severity fact found this round (W0) is already
scheduled work in the merge-soundness plan, so per the W1 precedent it
is reported as a known merge-order constraint, not new drift.

## Warnings

### [W0] The JSON Schema rejects the KB-legal composed envelope probe-aeneas ships — known, scheduled in merge-soundness PR 3a
- **Location**: schemas/atom-envelope.schema.json ($defs.singleToolAtomsEnvelope);
  docs/schema-validation.md:4-5; tests/schema_validation.rs:217-250
- **Issue**: `kb/engineering/schema.md:75` (and P9) state that producers
  other than the hub may emit composed envelopes under their own schema
  strings — "e.g. `probe-aeneas/extract` with `inputs: [Rust, Lean]`
  and no `source`". probe-aeneas's shipped
  `examples/aeneas_curve25519-dalek_4.1.3.json` is exactly that shape,
  and it fails validation (re-verified with jsonschema 0.28 semantics /
  Draft 2020-12: 1 error — `singleToolAtomsEnvelope` requires `source`
  and its `additionalProperties: false` forbids `inputs`;
  `mergedAtomsEnvelope` pins `schema` to `probe/merged-atoms`). The
  repo's own test masks the gap:
  `single_tool_aeneas_extract_envelope_is_valid` constructs a synthetic
  `probe-aeneas/extract` **with** a `source` object — a shape
  probe-aeneas does not emit. `docs/schema-validation.md`'s headline
  ("validates both single-tool and merged envelopes") overstates
  accordingly: the failure class is structural (composed single-tool),
  not just the unregistered-schema-string list the doc gives at :66-68
  (that list itself is verbatim-accurate).
- **Evidence**: `git -C ../probe-aeneas show origin/main:examples/...`
  top-level keys `[data, inputs, schema, schema-version, timestamp, tool]`;
  Draft202012Validator → 1 error.
- **Status**: **Already scheduled** — merge-soundness-fix-plan.md §6
  ("Composed-provenance detection is structural" + "Executable schema":
  rework the branch split to key on provenance shape so "the Aeneas
  composed envelope validates instead of matching neither branch"),
  assigned to PR 3a in §9 together with "Aeneas-envelope validation
  tests (test m)", which also replaces the masking synthetic test. The
  same PR registers `probe/projected-atoms` and `probe-vcvio` in the
  schema, shrinking schema-validation.md's "not yet registered" list.
- **Recommendation**: No separate action; land PR 3a per the plan, then
  update `docs/schema-validation.md`'s coverage prose and unregistered
  list in that PR.

### [W1] ADR-006 spec/code inversion — known carry-over, unchanged
- **Location**: src/commands/merge.rs:131-166; src/types.rs:163-177;
  src/commands/propagate.rs:151-165
- **Issue**: Unchanged from the previous report: the KB and CLI help
  describe correspondence-record attachment and `probe/projected-atoms`
  detection; the code still injects dependency edges and
  `detect_category` has no projected-atoms arm. Additionally noted this
  round: `propagate.rs` only ever upgrades (`verified` →
  `transitively-verified`); P23's restated semantics are a
  recomputation that can also demote. All of this is the documented
  "spec only — implementation lands in follow-up PRs" state.
- **Recommendation**: Land the merge-soundness plan's PRs 3a/3b. Not a
  docs action; listed so the next audit doesn't rediscover it.

### [W2] TESTING.md's per-probe integration section misdescribes three sibling repos
- **Location**: probe-extract-check/TESTING.md:211-213, 222-223
- **Issue**: The intro claims each probe repo validates extract output
  "using `probe-extract-check` as a dev-dependency", running "in each
  probe's CI", "library API directly (no subprocess)". Verified against
  the siblings' origin/main: (1) probe-aeneas has no
  probe-extract-check dependency at all (dev-deps: `serde_json` only);
  (2) probe-lean's CI installs the CLI and runs it as a subprocess
  (`.github/workflows/ci.yml:187,193`); (3) probe-verus's CI runs
  `cargo test --lib` plus three named `--test` targets, none of which
  is `extract_check` — the file exists but never runs in CI. Separately,
  :222-223 claims probe-verus has an `#[ignore]` live test calling
  `cmd_extract`; that test was merged into `extract_backward_compat`
  (issue #23), which is not `#[ignore]` (it runtime-gates on
  `tools_available()`). The probe-rust half of both claims is correct.
- **Recommendation**: Rewrite the intro per-repo (the table rows
  themselves are accurate); fix or drop the :222-223 sentence. File the
  probe-verus CI omission in that repo (see Real bugs).

### [W3] TESTING.md describes assertions four tests don't make
- **Location**: probe-extract-check/TESTING.md:147, 180-183, 220, 88
- **Issue**: (1) `golden_verus_micro_categorized_deps` — doc says
  "`body-dependencies` / `requires-dependencies` populated correctly";
  the test (`golden_tests.rs:262-279`) asserts only `body-dependencies`
  (the fixture carries `requires-dependencies`, so this is a coverage
  gap the doc masks). (2) `properties_{verus,lean,aeneas}_micro` — the
  "Same" rows inherit "no overlaps, ratio within bounds" from
  `properties_rust_micro`, but those three assert only error-freedom;
  overlap and completeness are warnings and go unchecked, and even the
  rust variant does not assert the ratio half. (3) The probe-lean row's
  "atom count" is a non-emptiness check in `Tests/Main.lean`, not a
  count. (4) `test_cache_uses_canonical_key` asserts only
  `errors.is_empty()`; cache-entry sharing is stated intent, never
  observed.
- **Recommendation**: Weaken the four descriptions to what is asserted,
  or strengthen the tests (the categorized-deps one is worth
  strengthening — see Real bugs).

### [W4] README and `probe enrich --help` use the P23 wording the KB explicitly retired
- **Location**: README.md:60-61; src/main.rs:82-86
- **Issue**: Both say enrichment upgrades atoms "whose entire
  transitive closure is verified or trusted".
  `kb/engineering/properties.md:208` states P23's path/seed-based rule
  "*replaces* the earlier 'every transitively reachable dependency is
  verified or trusted' wording", and the code follows the KB
  (`propagate.rs:21-23` seeds only from `unverified`/`failed`;
  missing-status atoms are transparent). The docs describe semantics
  that would wrongly predict non-promotion when status-less atoms are
  in the closure.
- **Recommendation**: One-sentence fix in both places ("no failed or
  unverified atom is reachable along a non-trusted path", or link P23).

### [W5] consumer-guide's flagship atom example teaches the wrong `display-name` convention
- **Location**: docs/consumer-guide.md:93 (vs :88)
- **Issue**: The worked example gives `"display-name": "add"` for the
  code-name `...scalar/Scalar#add()`. `kb/engineering/schema.md:122`
  specifies `"MyStruct::method"`, P21 mandates `SelfType::method` for
  impl methods, `docs/SCHEMA.md:72` says `"Scalar::add"`, and real
  probe-rust data agrees (`"Scalar52::zeroize"` etc.).
- **Recommendation**: `"add"` → `"Scalar::add"`.

### [W6] consumer-guide denies an example file probe-leanblueprint ships
- **Location**: docs/consumer-guide.md:140-142
- **Issue**: "probe-leanblueprint has no standalone example JSON" — its
  origin/main has
  `examples/verso-blueprint-project-template/extract.json`
  (`probe-leanblueprint/extract`, schema-version 3.0) plus six
  `extract.summary.json` sidecars.
- **Recommendation**: Fix the sentence; optionally add the row to the
  example-files table.

### [W7] consumer-guide describes the blueprint summary as per-node; the KB says aggregated
- **Location**: docs/consumer-guide.md:193-194
- **Issue**: "counting statement and proof status per blueprint node" —
  `kb/engineering/schema.md:97` defines `probe-leanblueprint/summary`
  as "aggregated over blueprint nodes, **not keyed per node**", and the
  shipped artifact's `data` is `{totals, all, definitions, theorems,
  headline, by-chapter}`.
- **Recommendation**: "per blueprint node" → "aggregated across
  blueprint nodes (with per-chapter breakdown)".

### [W8] Dead probegraph link in ui-views' doc table
- **Location**: docs/ui-views.md:367
- **Issue**: Links `docs/guides/INTERACTIVE_VIEWER.md` in probegraph;
  that path does not exist on probegraph's origin/main (the guides dir
  has `viewer.md`, `ci-integration.md`, `metrics-reference.md`,
  `vscode-extension.md`). The other four probegraph links resolve.
- **Recommendation**: Repoint to `docs/guides/viewer.md`.

### [W9] Stale "(Rust, Lean, CI)" enumerations for the validation recipes
- **Location**: README.md:24; docs/SCHEMA.md:114
- **Issue**: `docs/schema-validation.md` now has Rust / command-line /
  CI sections (the Lean heading was renamed in #71); README's own
  §JSON Scheme line already says "(Rust, CLI, CI)" but the doc-list
  bullet at :24 and SCHEMA.md:114 still say Lean.
- **Recommendation**: Two one-word fixes.

### [W10] The live merge-soundness plan targets doc paths that no longer exist
- **Location**: merge-soundness-fix-plan.md:10, 703, 802, 812, 1089,
  1137-1150
- **Issue**: The unexecuted plan names `docs/categorical-framework.md`
  (moved to `kb/engineering/categorical-framework.md` in #65),
  `docs/merge-algorithm.md` and `docs/mappings-spec.md` (archived and
  frozen in #65) as §9 rewrite targets, and cites line numbers in
  `docs/SCHEMA.md` (~196, ~318) that predate its reduction to 114
  lines. Whoever executes PRs 3a/3b from this plan will be misdirected
  into rewriting frozen archive files.
- **Recommendation**: Patch the plan's §9 target list to the KB paths
  (the archived copies are explicitly "not updated"; their outcome
  headers already delegate to the KB).

### [W11] The archived full merge-soundness plan is the only archive file without an outcome header
- **Location**: docs/archive/merge-soundness-fix-plan-full.md:1-10
- **Issue**: Its front matter still reads "status: agreed — not yet
  started". The other 7 archive files all carry a dated
  "> **Archived …**" header (the plan's own ground rule, line 33). The
  live condensed plan points here, but the pointer is one-directional —
  a reader landing in the archive sees a live-looking plan.
- **Recommendation**: Prepend the standard header ("full version
  preserved for provenance; the live, condensed plan is
  `merge-soundness-fix-plan.md` at the repo root").

### [W12] The tracked ambiguity report cites a dead path and unverifiable files
- **Location**: kb/reports/ambiguity-report.md:91, 65
- **Issue**: :91 locates a finding in `docs/categorical-framework.md`
  (moved to kb/ in #65). :65's W2 finding cites
  `docs/lightning-talk-probes.md` / `docs/slides23-31.md`, which
  decision 4 (2026-09-29) made local-only/gitignored; the plan's own
  instruction was to "amend that finding" if they weren't tracked, and
  it wasn't — the finding is now unverifiable from a clean clone.
- **Recommendation**: Update the two locations (or annotate W2 as
  referring to local-only material, with the decks' edge-injection
  content still due an update when PR 3b lands).

## Info

### [I1] Post-move link rot inside the frozen archive copies
- **Location**: docs/archive/merge-algorithm.md:10-11,138;
  docs/archive/mappings-spec.md:9,12,138;
  docs/archive/merge-soundness-fix-plan-full.md:11
- **Issue**: Body links written pre-move now resolve one directory
  short (`../kb/…` → `docs/kb/…`; `SCHEMA.md` →
  `docs/archive/SCHEMA.md`; `merge-soundness-review.md` → archive-local).
  The prepended headers use correct `../../kb/…` paths, and the files
  are explicitly "kept for historical reference; not updated" — hence
  info, not warning. `check-kb-links.sh` cannot catch these (it only
  walks `kb/`).
- **Recommendation**: Optional mechanical fix; or accept as frozen.

### [I2] One-condition stub predicates in the Python snippets
- **Location**: docs/consumer-guide.md:178-181; docs/testing-guide.md:43-44
- **Issue**: Both filter stubs by `code-path != ""` alone. P3 is a
  three-condition iff, and consumer-guide itself states all three at
  :124. Correct on conformant data; the snippets are the copy-paste
  artifact.
- **Recommendation**: Add the two extra conditions or a one-line
  "(sufficient on conformant data; the full test is P3)" comment.

### [I3] Enum drift guard: blind spots and a docstring omission
- **Location**: scripts/check-enum-drift.py:2-24,108-117;
  probe-extract-check/TESTING.md:14; docs/ui-views.md:238-244
- **Issue**: (1) TESTING.md:14 restates five `kind` values on one line
  but the file is outside the scanned set (README + docs/), despite
  being advertised as a live reference doc at README:29. (2) One-value-
  per-row tables (ui-views' status filter table :238-244) are invisible
  to the per-line rule and carry no `<!-- enum-ok -->`. (3) The module
  docstring omits the `is-disabled` dead-value check, and the
  `<!-- enum-ok -->` marker silences that check too, not just
  enumerations.
- **Recommendation**: Add `probe-extract-check/TESTING.md` to the
  scanned set (with an `enum-ok` marker on line 14 or a KB link),
  mention the dead-value check in the docstring. The multi-line-table
  blind spot is inherent to the per-line rule; accept it.

### [I4] CHANGELOG [Unreleased] names files a sibling bullet in the same section archives
- **Location**: CHANGELOG.md:21-22
- **Issue**: "Aligned `docs/merge-algorithm.md`, `docs/mappings-spec.md`
  … with ADR-006" precedes the bullet recording those files' archival.
  Accurate as history, but a reader of the released section will follow
  three dead paths. Cosmetic ordering artifact.
- **Recommendation**: Optionally annotate "(since archived, see below)".

### [I5] Untracked session artifact: docs/verification-statuses-pr-comments.md
- **Location**: docs/verification-statuses-pr-comments.md (untracked;
  the repo's only non-ignored untracked file)
- **Issue**: Pre-posting working copy of a review delivered to PR #24
  (merged 2026-07-01, 10 inline comments posted); the reviewed document
  left this repo in 0.4.0; its one open question ("does `is-hidden`
  exist?") is answered by `kb/engineering/schema.md:179`; it uses the
  pre-rename `is-disabled` field name throughout. Same class as the I6
  artifacts the plan ordered deleted.
- **Recommendation**: Delete.

### [I6] ui-views internal drifts (three small ones)
- **Location**: docs/ui-views.md:189-190,201,354; :272-284; :110-111
- **Issue**: (1) Crate-map and source-linking read `source.package` /
  `source.repo`/`source.commit`, but the views target merged files
  where `source` is replaced by `inputs` — as :329-336 itself states;
  the fallbacks need `inputs[].source`. (2) The "additional parameters"
  table proposes `view`/`kind`/`status` params probegraph already ships,
  and the key-parameters table omits `focus`, which `--emit-focus`
  (src/main.rs:76-78) makes a first-class interface. (3) "no colour
  when `verification-status` is absent" collapses VeriLib's white
  (in-scope, unspecified) vs no-bar (no verification intent)
  distinction; the five status→colour pairs themselves match exactly.
- **Recommendation**: Small wording fixes on the next ui-views pass.

### [I7] consumer-guide completeness nits
- **Location**: docs/consumer-guide.md:11, 53, 109-110, 190, 216
- **Issue**: The common-optional-fields list omits `status-origin` and
  `maps-to`/`mapped-from` — the guide never mentions correspondence
  records although ui-views builds a view on them; "sorry detection"
  understates probe-lean's kernel-based taint walk (P16); the
  `.verilib/probes/` default is qualified for probe-aeneas (cwd
  fallback without a project root); `probe summary` accepts any atom
  file, not only merged; :216 is the guide's only repo-relative link in
  an otherwise absolute-URL document.
- **Recommendation**: Fold into the next consumer-guide touch-up; none
  is individually urgent.

### [I8] Illustrative code-names don't resolve in the linked example files
- **Location**: docs/SCHEMA.md:53; docs/consumer-guide.md:88,94
- **Issue**: `probe:curve25519-dalek/4.1.3/scalar/Scalar#add()` and
  `…/field/reduce()` match the KB's own simplified illustration
  (schema.md:244-248) but exist in no shipped example; real keys carry
  full module paths and impl segments. A consumer trying the examples
  against the linked file finds nothing.
- **Recommendation**: Either mark as schematic or quote a real key. If
  the KB illustration changes, that is a KB decision — flag, don't fix
  unilaterally.

### [I9] docs/SCHEMA.md states extension-field rules the normative KB doesn't
- **Location**: docs/SCHEMA.md:33-38 vs kb/engineering/schema.md:165-167
- **Issue**: Four normative-sounding rules (must-not-conflict,
  kebab-case, must-ignore, omit-when-empty) have no KB counterpart,
  in a file that declares the KB authoritative at :13.
- **Recommendation**: Promote to the KB (human decision) or reword as
  non-normative guidance.

### [I10] Sibling-repo staleness (out of this repo's scope; file issues there)
- **Location**: ../probe-aeneas docs/USAGE.md:64-67 + README.md:5;
  ../probe-leanblueprint README.md:24; ../probe-verus .github/workflows/ci.yml
- **Issue**: probe-aeneas still describes extract as producing
  "cross-language dependency edges" (retired by ADR-006);
  probe-leanblueprint's README cites property "P26", which no longer
  exists in the hub's properties.md; probe-verus CI never runs its
  `tests/extract_check.rs` (see W2/Real bugs).
- **Recommendation**: One issue per sibling repo.

### [I11] Two procedural nits
- **Location**: probe-extract-check/TESTING.md:238-239;
  docs/testing-guide.md:153-158
- **Issue**: TESTING.md's "Adding a new fixture" steps 3/4 are
  inverted — at step 3 the named test doesn't exist yet, so the filter
  silently matches nothing and exits 0. testing-guide credits
  `src/types.rs` with stub detection; its single test is
  field-preservation, `test_is_stub` lives in `commands/merge.rs`.
- **Recommendation**: Swap the steps (or point step 3 at the CLI);
  reword the attribution.

## Real bugs (code, not docs — do not document around)

1. **JSON Schema + masking test** (W0): `singleToolAtomsEnvelope`
   rejects composed single-tool envelopes the KB legalizes;
   `tests/schema_validation.rs:217-250` validates a synthetic shape
   probe-aeneas doesn't emit. Scheduled: merge-soundness plan PR 3a.
2. **`golden_verus_micro_categorized_deps`** asserts nothing about
   `requires-dependencies` despite the fixture carrying it (W3.1).
3. **probe-verus CI** never builds/runs `tests/extract_check.rs`
   (`cargo test --lib` + three explicit `--test` targets that don't
   include it) — sibling repo.
4. **`propagate.rs` upgrade-only** vs P23's recomputation semantics —
   part of the known ADR-006 rollout (W1), not new.

## Overlap map (topic × canonical home, post-cleanup)

| Topic | Canonical | Remaining copies / risk |
|-------|-----------|-------------------------|
| Stub predicate | properties.md P3 / schema.md#stubs | consumer-guide prose (accurate) + two 1-of-3 code snippets (I2) — highest residual drift risk |
| JSON-Schema coverage prose | docs/schema-validation.md | README:24 + SCHEMA.md:114 already drifted (W9); five files mention coverage |
| probegraph doc links | probegraph repo | duplicated inline + table in ui-views; only the table copy is dead (W8) |
| Status/kind/language enums | kb/engineering/schema.md | guarded by check-enum-drift.py; blind spots in I3 |
| Envelope fields, merged variant | kb/engineering/schema.md | all live docs delegate — clean |
| extract-check tests | probe-extract-check/TESTING.md | single home — clean (accuracy findings W2/W3 notwithstanding) |

## Structural gaps

1. **No cross-repo/external link validation.** `check-kb-links.sh`
   skips `https://` links and only walks `kb/` for markdown links;
   every dead-link finding this round (W8, the fixed W9-class, I1) is
   in that blind spot. Extending the script to check `docs/*.md`
   relative links is cheap; external GitHub links would need a
   networked CI step (optional, could be a scheduled job).
2. **Drift-guard scope** (I3): `probe-extract-check/TESTING.md` is a
   README-advertised live doc outside the scanned set.
3. **The schema's structural gap** (W0) has no test expressing the
   KB's `source`-XOR-`inputs` rule against real producer output —
   closed by merge-soundness PR 3a's "test m" when it lands.

## Verified clean

- Previous report's executed verdicts all hold: 6 live docs + 8
  archived (7/8 outcome headers accurate and their superseding
  artifacts exist — the 8th is W11); zero live links into the archive;
  no case collisions; root scratch gone; .gitignore matches the plan
  decisions; docs-cleanup-plan's Status paragraph verified against git
  history claim-by-claim.
- README: all 12 relative links resolve; Usage block matches the clap
  parser flag-for-flag (merge/project/enrich/summary, defaults
  included); per-tool table — all six siblings have both linked docs on
  origin/main with correct org/branch; §JSON Schema matches the
  schema's oneOf.
- CI: fmt/clippy/kb-links/enum-drift/test jobs run what the docs claim;
  both drift-guard invocations pass; enum value lists in the guard
  match schema.md exactly (15 kinds, 4 languages, 5 statuses).
- schema-validation.md: "Registered schema strings" reproduces the
  schema's patterns verbatim; all six "not yet registered" strings
  genuinely match no pattern; "only this repo validates" re-verified
  (zero jsonschema deps across all six siblings); Rust example matches
  the pinned jsonschema 0.28 API; curl URL path correct.
- consumer-guide: envelope example field-for-field identical to the KB;
  all six extract command lines verified against sibling USAGE docs;
  probe-leanblueprint/probe-aeneas/probe-vcvio prose accurate
  (blueprint signals, aeneas-config.yml, --lean/classification);
  example-files table — all four files exist on origin/main with the
  documented schema strings.
- ui-views: correspondence-record language matches ADR-006; status
  colour mapping complete over all five values and identical to
  VeriLib's canonical table; language partition covers the 4-value enum
  exactly; four of five probegraph links resolve.
- testing-guide: no stale counts anywhere; every described suite and
  named behavior exists; per-repo table verified (TESTING.md presence,
  probe-lean command verbatim, leanblueprint's absence).
- probe-extract-check/TESTING.md: all 65 test names resolve one-for-one
  in both directions (no orphans either way); fixture file lists and
  atom counts exact (15/4/10/3); ignored-test tables match the
  attributes; CLI flags, `-p` short form, and all three exit codes
  match main.rs; quick-start commands all run (30 lib + 27 active / 8
  ignored integration).
- CHANGELOG [Unreleased]: all spot-checked claims true (files exist,
  features present, #70/#72 entries accurate).
- merge-soundness plan/review/ADR-006/ambiguity-report cross-citations:
  all cited working files now tracked (C3 from the previous report is
  resolved).
