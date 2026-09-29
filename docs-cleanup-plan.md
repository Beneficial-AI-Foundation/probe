# Docs cleanup plan

Executes the findings of `kb/reports/docs-report.md` (docs-auditor,
2026-09-28, 3 critical / 11 warnings / 6 info). Read that report first —
it has the evidence and exact line references; this plan only adds
execution order, grouping, and the decisions that need a human.

Status: PR A merged (issue #66, PR #67; the whole #63 → #65 → #67 stack
landed on main 2026-09-29). PR B + housekeeping merged (issue #68,
PR #69). PR C+D executed (issue #70, branch `la/docs-cleanup-pr-cd`);
during it, W6's "verify again" check confirmed no sibling repo has
added jsonschema validation, and `probe-vcvio/extract` was added to the
unregistered-schema list alongside the report's five. Remaining: the
optional drift guard. C and D were folded into one PR: they share
`README.md`/`docs/consumer-guide.md` and the split was only by report
finding IDs.

## Ground rules

- The KB is normative. Every fix below points docs at
  `kb/engineering/schema.md` / `properties.md`; never re-enumerate an
  enum in `docs/`.
- Issue first, then draft PR with "Closes #N" (lacra's convention).
- Archive = `git mv` into `docs/archive/` + prepend an outcome header
  (what happened to the proposal, what supersedes it, date). Never bare-rm
  content that only exists there.
- Re-verify each report claim against the code before applying — if the
  report and the code disagree, the code wins.
- Docs-only changes: no version bump; add CHANGELOG entries under
  `[Unreleased]`; run `./scripts/check-kb-links.sh` + `cargo test` before
  each commit.

## Out of scope — do NOT fix here

- **W1 (ADR-006 spec/code inversion)**: `src/commands/merge.rs` still
  injects edges; `detect_category` has no `probe/projected-atoms` arm.
  That is the merge-soundness plan's PRs 3a/3b
  (`merge-soundness-fix-plan.md` §9). Touching merge/enrich code here
  would collide with that PR train.
- Anything that changes KB spec *semantics*. Link fixes and folding
  already-approved content are fine; new spec content is not.

## PR A — fix the live reference docs (C1, W3, I4, I5 + SCHEMA.md nit)

One issue + PR. All files stay; content corrected.

1. **`docs/ui-views.md`** (C1, the big one):
   - Rewrite lines ~56-59 and ~288: "mapping edges" → correspondence
     records (`maps-to`/`mapped-from`), matching line 31 and ADR-006.
     Cross-language linkage is a derived consumer view over records; if
     the UI draws them, they are record-derived links, not dependency
     edges.
   - Replace the restated enums with KB links: `language` set (line ~16),
     kind tables (~42-48), status lists (~89-90, ~207-214). Keep only the
     UI-owned mapping (color/toggle per value) and extend it to cover all
     five statuses incl. `trusted` and `transitively-verified`. Respect
     the two-channel colour convention (bar=verification, dot=checking;
     no blue) — source of truth is VeriLib's atoms_roles_statuses doc.
   - Fix the Rust-view contradiction: view partition by `language`
     means `proof`/`spec` atoms are `verus`, not `rust` — say explicitly
     which views show them.
   - Fix the stub predicate (two places): all three P3 conditions, or
     link the glossary/schema definition instead of restating.
   - Qualify `probe-*/atoms` as legacy in the schema-dispatch section and
     prefer structural detection (`inputs` present = composed).
2. **`docs/envelope-rationale.md`** (W3): trim ~60%.
   - Keep: "Why an Envelope Is Needed", "What the Envelope Should NOT
     Do" — unique rationale, exists nowhere else.
   - Retitle without "2.0"; delete the false "Today these files are bare
     JSON dictionaries" premise and all "Proposed" framing (it shipped).
   - Replace Field Reference, Merged Envelope Variant, Package
     Versioning, Folder Structure sections with links into
     `kb/engineering/schema.md` (they duplicate it field-for-field and
     have drifted; `translations/` contradicts `.verilib/mappings/`).
   - Delete the Rollout section (complete; contradicted by the KB's
     lockstep-bump runbook).
   - Fix examples: `probe-lean/atoms`+`atomize` → `probe-lean/extract`.
   - Then repoint `docs/SCHEMA.md`'s "full field reference" sentence
     (Design Rationale section) at `kb/engineering/schema.md#envelope-fields`,
     keeping the envelope-rationale link for rationale only.
3. **`docs/SCHEMA.md`** (I5 nit): "probe-verus = Rust" → note the
   kind→language rule (one clause).
4. **`src/commands/propagate.rs:64`** (I4): fix the dangling `@kb:`
   anchor → `kb/engineering/schema.md#common-optional-fields`.
   Optional: teach `scripts/check-kb-links.sh` to validate `@kb:` anchors
   (it currently only checks md links — this would have caught it).

## PR B — archive the historical docs (C2, W10)

One issue + PR. Pure `git mv` + outcome headers; report has the outcome
facts verified against git history.

1. `docs/probes-overview-slides.md` → archive. Header: July-2026
   presentation; colour/status half moved to VeriLib engineering docs
   (CHANGELOG 0.4.0); `disabled` renamed `untracked`; the spec-less⇒
   disabled rule was decided the other way (P24/P25); status vocabulary
   superseded by properties.md. Check whether `docs/image.png` /
   `image-1.png` (tracked, referenced only by this deck) should move to
   `docs/archive/` with it.
2. `docs/structure/001_motivation.md` → archive. Outcome: implemented as
   verilib-cli (`../verilib-cli`).
3. `docs/structure/002_requirements.md` → archive. Outcome: partly
   shipped in verilib-cli (structure-root, frontmatter, spec certs);
   `veri-name` never built; `scip:`/`panto:` naming reversed to `probe:`;
   spec-status vocabulary superseded by the single `verification-status`
   contract.
4. `docs/structure/003_implementation.md` → archive. Header MUST state:
   "Schema of probe outputs" section superseded by
   `kb/engineering/schema.md` (pre-envelope bare dicts, `scip:` scheme,
   `specified`/boolean-`verified` fields — none exist). Highest-risk
   stale content in the repo.
5. `docs/web-cli-probe/001_motivation.md` → **decision needed** (see
   below): delete (matches sibling 002's deletion in the ADR-005 sweep)
   or archive with outcome header (plan items all shipped or dropped).
6. Remove the now-empty `docs/structure/` and `docs/web-cli-probe/`
   dirs; grep the repo for links to the moved files (report says zero
   inbound — re-verify).

## PR C — extract-check + testing docs (W4, W5, W7, W8, part of W6)

One issue + PR.

1. Fold `docs/extract-check-design.md` lines ~6-18 ("What 'correct'
   means" properties table) + the three-layer framing into
   `probe-extract-check/TESTING.md` as a preamble; delete the rest of
   the file (every count is stale; TESTING.md's tables are the correct
   versions). Update inbound links: README.md, docs/consumer-guide.md.
2. `probe-extract-check/TESTING.md`: drop hand-maintained totals
   (replace with "run `cargo test -p probe-extract-check`"), keep the
   per-test semantic tables (verified 100% accurate), fix the Layer-2
   header, document exit codes (2 = parse failure, 1 = errors or
   warnings without `--allow-warnings`, 0 = clean) and the `-p` short
   flag.
3. `docs/testing-guide.md`: drop stale counts (7→9 schema, 15→22 merge
   unit); add the missing suites (tests/propagate.rs, propagate/project/
   summary/types unit tests); "translation-based edge creation" →
   mapping terminology; note probe-leanblueprint has no TESTING.md;
   point the schema-validation section at `docs/schema-validation.md`
   directly (not the README).
4. `docs/schema-validation.md` (W6): scope headings to this repo (only
   probe validates today — verify again in case siblings added it);
   state actual schema coverage (`probe/summary`, `probe/mappings`,
   `probe/projected-atoms`, `probe-lean/viewify`,
   `probe-leanblueprint/summary` are NOT registered in the JSON schema's
   oneOf). README §JSON Schema (W7): keep one pointer line, drop the
   restated prose.

## PR D — user-facing gaps (W2, W9, I5 README)

Folded into PR C (one issue + PR covering both; see Status).

1. README: add `probe project` to the CLI list and Usage block
   (`--mappings` required, `--forward-depth` default 2,
   `--reverse-depth` default 0, `-o` default `projected.json`,
   `--emit-focus` — re-verify against src/main.rs); fix the
   build→run copy-paste gap (`cargo run -- merge …` or an install line).
2. `docs/consumer-guide.md`: fix the probe-lean example row
   (`lean_ExampleProject_0.1.0.json`; the Curve25519Dalek file is
   probe-aeneas's); soften "All examples use curve25519-dalek"; add a
   one-line `probe project` mention in the progress section.
3. probe-vcvio listing in README + consumer-guide tool tables: add it,
   labelled as a proof of concept (decision 2, 2026-09-29).

## Housekeeping — tracking + scratch (C3, W11, I1, I2, I3, I6)

Mostly `git add`/moves; some need decisions. Can ride along with PR B.

1. **Track the merge-soundness files** (C3): `merge-soundness-fix-plan.md`,
   `merge-soundness-review.md`, `docs/archive/merge-soundness-fix-plan-full.md`.
   ADR-006 and the tracked ambiguity report cite them; the plan is live
   and drives the open PR train.
2. Move `kb/reports/*-probe-aeneas.md` (untracked) to the probe-aeneas
   repo (W11) — create `kb/reports/` there or give the auditor skills an
   output-dir override. Rescue first: the "probe#58 before probe-aeneas
   0.20.0 conformance" merge-order note goes on that issue if still open.
3. Root scratch (I1): remove `test.json` (5.7 MB), `summary_*.json`,
   `dots.pdf`, the three "Signal Shot Launch" files (one is 0 bytes);
   move presentation material to wherever the decks live (not this
   repo). Extend `.gitignore`: `.vscode/`, `scripts/__pycache__/`,
   root-level scratch JSON.
4. Assets (I2): tracked orphans `docs/assets/{atom-bar,atom-dot,shapes-roles}.png`
   and untracked `docs/assets/{division,pipeline}.{mmd,png}` have zero
   referrers — move with the decks or delete (`division.mmd`'s
   facts-vs-palette split is drawn nowhere else).
   Status 2026-09-29: resolved — all deck assets untracked and
   gitignored with the decks (kept on disk); the earlier untrack failure
   was a global `git rm` deny rule, since narrowed to allow `--cached`.
5. `docs/probe-dispatch-plan.md` (I3, untracked, unique content:
   marker-file dispatch design, the live Schema 2.0-verilib vs 3.0-hub
   skew, deployment gaps) — **decision needed**: track it (its own text
   proposes `kb/tools/probe-dispatch.md` + an ADR) or distill into an
   issue.
6. Session artifacts (I6): `docs-probe-consistency-review.md`,
   `schema-vs-kb-audit.md` are fully applied — delete.
   `pr-63-review-comments.md`: first confirm all 7 items were
   posted/applied to PR #63, then delete.
7. Slide decks `docs/lightning-talk-probes.md`, `docs/slides23-31.md`
   (cited by ambiguity-report W2) — **decision needed**: track here or
   move to engineering-docs and amend that finding. Note both still
   describe edge injection; they must be updated when PR 3b lands
   regardless of home.

## Optional — drift guard (structural gap 1)

Small CI check: the status/kind/language enum value strings may appear in
exactly one file (`kb/engineering/schema.md`); grep fails the build
otherwise. Would have caught C1/C2 at commit time. Do last; keep it
dumb (fixed string list, allowlist for the glossary/ADRs if needed).

## Decisions needed from lacra (collected)

All decided 2026-09-29:

1. `docs/web-cli-probe/001_motivation.md`: **archived** with outcome
   header.
2. probe-vcvio in the public tool tables: **list it now, labelled as a
   proof of concept** (README ecosystem table + consumer-guide tool
   table, in PR C+D).
3. `docs/probe-dispatch-plan.md`: **kept local-only** (gitignored). Not
   needed for the probe-merge work (it covers upload→probe routing, not
   the merge algorithm). Promote to `kb/tools/` + ADR later if wanted.
4. Slide decks (`lightning-talk-probes.md`, `slides23-31.md`):
   **kept local-only** (gitignored), together with their untracked
   `docs/assets/` diagrams.
5. Signal Shot / dots.pdf: **deleted** (they live where they were
   authored, not in this repo).

## Suggested order

~~A~~ → B (+housekeeping riding along) → C+D → optional drift guard.
Each remaining PR branches off `main`; all are independent of the
merge-soundness PR train and touch no spec content. After each PR: run
`/docs-auditor` verdicts off the list, and re-run the ambiguity auditor
only if KB files changed.
