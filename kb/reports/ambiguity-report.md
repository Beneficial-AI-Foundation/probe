---
auditor: ambiguity-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB)
scope: branch la/merge-soundness-pr2-enrich-recomputation (merge-soundness PR 2,
  issue #78 — enrichment recomputation, summary consumer contract), delta audit
  over the PR 3a report; KB surfaces this PR implements re-checked against the
  code
status: 1 critical (scheduled, merge-order constraint, carried over), 3 warnings, 2 info
---

## Critical

### [C1] ~~Enrichment implementation contradicts P23~~ — CLOSED by this branch
- **Resolution**: `enrich_verification_status` (src/commands/propagate.rs) now
  implements P23's path-based recomputation: one BFS from the unified seed set
  (`failed`/`unverified` + every `status-origin`-bearing atom), transparency
  through missing-status atoms, trusted-boundary precedence
  (`trusted` AND origin absent), labels set fresh with unconditional demotion
  of marked `transitively-verified`. The PR 1 counterexample
  (`v [verified] → helper [no status] → bad [unverified]`) is a regression
  test (`test_contamination_flows_through_missing_status`). Retained in the
  report this round so the closure is on record; drops off next run.

### [C2] `merge --mappings` injects dependency edges, contradicting revised P13
- **Location**: src/commands/merge.rs (edge-injection block; tests pinning it)
- **Issue / Evidence**: unchanged from the PR 1 report.
- **Recommendation**: PR 3b (next in the plan §9 order). Its merge-order
  precondition — PR 2's recomputation, which μ's re-enrichment reuses — is
  satisfied by this branch.

## Warnings

### [W1] probe-verus spec-position dependency arrays are empty in real artifacts
Carried over unchanged (producer defect, filed in the probe-verus rollout
issue; recorded in ADR-006 Decision 4). Not a hub KB defect.

### [W2] Presentation decks still describe edge injection
Carried over unchanged (local-only decks, updated when next regenerated).

### [W3] Remaining spec-ahead-of-code statements, all scheduled
- **Location**: kb/engineering/schema.md § Authority validation ("merge
  **re-enriches** the atoms category", line ~306) and § Correspondence records
  (union rule); properties.md P4/P5 (law statements), P8 (extended arrays,
  project boundary), P9 (dedup clause), P13/P27 (record attachment/union),
  P15 (projection trims categorized arrays), P23 ("merge re-enriches",
  projection enrich-then-trim); glossary `projection` ("recomputed on the full
  input before trimming"); kb/tools/probe-project.md `prepare = enrich ∘
  normalize` step; kb/tools/probe-merge.md phases 2/4/5.
- **Issue**: these describe PR 3b/3c behavior. After PR 2 the *enrichment*
  statements — P23's recomputation definition, blocker-seed and
  trusted-boundary rules, P16's hub enrichment note, ADR-006 Decision 3's
  unary-boundary preparation for `probe enrich`, Decision 9's summary
  contract, the glossary's `blocker seed`/`trusted (boundary)`/
  `transitively-verified` definitions, and kb/tools/probe-summary.md in its
  entirety — are implemented and verified against the code; they leave the
  divergence list.
- **Recommendation**: none beyond executing the plan's remaining PRs; listed
  so the window stays explicit.

## Info

### [I1] ADR-003 body retains superseded application wording
Carried over (accepted: historical record behind a supersession banner).

### [I2] Version-gate producer names duplicated between ADR-006 and code
Carried over (by design, Decision 10; unit-test matrix is the drift guard).

## Fixed during this audit

(Doc/spec contradictions found by this round's code-quality pass, recorded
here because they were KB-adjacent staleness:)

- kb/engineering/architecture.md hub module inventory: `src/commands/propagate.rs`
  was absent and the `summary.rs` line predated even the three-list contract —
  both rewritten to the implemented behavior.
- README quick-start, docs/consumer-guide.md, docs/testing-guide.md, CLAUDE.md:
  upgrade-pass wording and three-list summary descriptions updated to
  recomputation / four lists.

## Verified clean (this PR's KB surfaces)

- **P23 ↔ propagate.rs**: the executable definition, seed quantifier ("any
  atom carrying `status-origin`" — including status-less marked atoms, now
  pinned by test), trusted-boundary precedence, whole-atom trust, missing-dep
  trusted fallback, non-candidate untouchability, determinism and idempotence
  all match the implementation.
- **ADR-006 Decision 2** (marker semantics) and **Decision 3** (recomputation
  + carrier preparation at unary boundaries, enrich side) ↔ code: consistent;
  `prepare_atoms` implements `enrich ∘ normalize` and `cmd_enrich` applies it
  after authority validation.
- **Decision 9 / kb/tools/probe-summary.md ↔ summary.rs**: four-partition
  contract, membership conjunction, kernel-taint locality, blueprint
  exclusion (both partitions and `depended_upon`), wire field names, CLI
  shape, stderr statistics — all verified true, with the KB's documented JSON
  field names pinned by a new wire-contract test.
- **schema.md `status-origin` row ↔ schemas/atom-envelope.schema.json**: the
  executable schema now constrains the marker to exactly the two documented
  values (positive + negative fixtures).
- **Glossary**: `blocker seed`, `carrier`, `trusted (boundary)`,
  `transitively-verified`, `status-origin` — defined, cross-linked, and
  consistent with the implementation landed here; no new undefined terms were
  introduced by this PR (its code comments use the glossary vocabulary).
- `./scripts/check-kb-links.sh` and the enum drift guard pass.
