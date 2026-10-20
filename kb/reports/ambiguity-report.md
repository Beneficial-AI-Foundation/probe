---
auditor: ambiguity-auditor
date: 2026-10-20
repo: probe (hub)
kb: kb/ (own KB, resolved via common §1.1)
scope: round 3. Branch la/keep-translated-labels (issue #92), uncommitted
  working-tree changes vs origin/main. Focus on `git diff origin/main -- kb/`
  (ADR-006, P16, P23, glossary, schema.md, architecture.md, kb/index.md,
  kb/decisions/index.md, kb/tools/probe-aeneas.md), with contradictions
  searched across all of kb/. Per §2b the branch is not the default branch.
  The diff is a deliberate spec amendment (approved design:
  translation-status-proposal.md), so the auditor checked the amended text
  for internal consistency. origin/main served to find text that the
  amendment did not touch but needs to. Code claims were checked against
  the working-tree src/commands/propagate.rs and src/types.rs.
  `./scripts/check-kb-links.sh` reports all KB links OK.
  Merge-order constraint: see W1.
status: 0 critical, 1 warning, 0 info
---

## Critical

None.

## Warnings

### [W1] Merge-order constraint: probe-aeneas 0.22.0 is named but not released
- **Location**: kb/decisions/006-correspondence-records.md:179,
  kb/engineering/properties.md:247, kb/engineering/architecture.md:109,
  kb/tools/probe-aeneas.md:50
- **Issue**: Four places say the refresh ships "from 0.22.0". The
  probe-aeneas working tree now has `version = "0.22.0"`, but the change is
  uncommitted, no `v0.22*` tag exists, and probe-aeneas#78 is open. The
  coordinator accepts this as a documented constraint: the hub PR merges
  after probe-aeneas 0.22.0 is tagged, and the PR description says so. Per
  the auditor rules this stays a merge-order constraint, not a clean result.
- **Evidence**: `grep '^version' ../probe-aeneas/Cargo.toml` gives
  `version = "0.22.0"`. `git -C ../probe-aeneas status --short` shows
  Cargo.toml, CHANGELOG.md and others modified. `git tag -l 'v0.22*'` is
  empty. `gh issue view 78 -R Beneficial-AI-Foundation/probe-aeneas` gives
  OPEN.
- **Recommendation**: No KB change. Before merging the hub PR, check that the
  probe-aeneas v0.22.0 tag exists.

## Info

None.

## Verified clean

- **Round 2 fixes**:
  - C1: ADR-006:161 and :194-195 say "code atom". glossary.md:67 says "On a
    code atom, a `\"translation\"` label is ... never relabeled".
    glossary.md:197 says "on a code atom with `status-origin:
    \"translation\"`". A search for relabel and "translation atom" wording
    without "code atom" finds no remaining unqualified exemption in kb/.
  - W1: glossary.md:57-59 defines `code atom` as any atom whose `language`
    is not `"blueprint"`. This matches the code (`is_kept_copied_label`:
    `language != "blueprint"`) and summary.rs `is_blueprint`. P23 and P16
    link to it (two `glossary.md#code-atom` links in properties.md).
  - I1: P16 (properties.md:171) is now split into short sentences.
  - I2: translation-status-proposal.md:87 states that the exception covers
    code atoms only.
- **Round 1 fixes**: still in place.
  - The P23 box exception (properties.md:222).
  - Qualified label meanings (glossary.md:200-201, schema.md:157).
  - The ADR-006 Decision 2 relabeling difference (:132-136).
  - The `--skip-enrich` wording and the architecture.md step 10 refresh.
  - The 2026-10-20 stamps and both index notes.
  - The inline "any other marker value" definition.
- **Code vs KB**: kept atoms are excluded only from relabeling candidates.
  They stay in the reverse index and the seed set. A kernel-taint or
  malformed marker and a blueprint aggregate are still demoted. This matches
  P23:224, :230 and ADR-006:118-123.
- **P4** (properties.md:47, categorical-framework.md:71): a kept label
  behaves like a base status (ADR-006:185-187). Enrichment still rewrites
  only `verified`/`transitively-verified` on candidates.
- **Trusted-boundary rule**: copied `trusted` stays a seed, consistent
  across P23, ADR-006, schema.md:159 and the glossary.
- **Decision 9 / probe-summary.md**: `imported_verified` admits both
  translated labels. Blueprint atoms appear in no partition, so the
  blueprint rule changes no summary output.
- **probe-leanblueprint.md and ADR-006:349-356**: marker propagation onto
  aggregates is consistent with relabeling aggregates whatever their marker.
- **schema.md**: example tool version 0.6.0 matches Cargo.toml. The 0.6.0
  compatibility note (:312) is consistent with the unchanged Decision 7
  gate table.
