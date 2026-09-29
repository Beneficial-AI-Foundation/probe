---
auditor: ambiguity-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB)
scope: branch la/merge-soundness-pr3a-authority-boundary (merge-soundness PR 3a,
  issue #76 — authority boundary), delta audit over the 2026-09-28 PR 1 report;
  KB surfaces this PR implements re-checked against the code
status: 2 critical (both scheduled, merge-order constraints, carried over), 3 warnings, 2 info
---

## Critical

Carried over from the 2026-09-28 PR 1 audit. Both are the known code-vs-spec
divergences the plan (merge-soundness-fix-plan.md §9) schedules: the KB landed
first, and the code PRs bring the implementation into conformance. PR 3a (this
branch) closes the *authority-boundary* portion of that window; these two
remain.

### [C1] Enrichment implementation contradicts P23 on both branches
- **Location**: src/commands/propagate.rs (upgrade-only pass; missing-status
  atoms opaque to contamination)
- **Issue / Evidence**: unchanged from the PR 1 report
  (`v [verified] → helper [no status] → bad [unverified]` mislabels `v`).
- **Recommendation**: PR 2. Its merge-order precondition — the authority
  boundary of PR 3a — is satisfied by this branch: `probe enrich` now
  validates its input (projection rejection + version gate) before reaching
  the recomputation function.

### [C2] `merge --mappings` injects dependency edges, contradicting revised P13
- **Location**: src/commands/merge.rs (edge-injection block; tests pinning it)
- **Issue / Evidence**: unchanged from the PR 1 report.
- **Recommendation**: PR 3b. The prerequisite the PR 1 report named —
  "regenerated artifacts must wait for the version gate (PR 3a)" — is now in
  place.

## Warnings

### [W1] probe-verus spec-position dependency arrays are empty in real artifacts
Carried over unchanged (producer defect, filed in the probe-verus rollout
issue; recorded in ADR-006 Decision 4). Not a hub KB defect.

### [W2] Presentation decks still describe edge injection
Carried over unchanged (local-only decks, updated when next regenerated).

### [W3] Remaining spec-ahead-of-code statements, all scheduled
- **Location**: kb/engineering/properties.md P8 (unary-boundary normalization,
  extended arrays), P9 (dedup clause), P15 (projection trims categorized
  arrays), P23 (recomputation, merge re-enriches, projection
  enrich-then-trim); glossary `projection` ("recomputed on the full input
  before trimming"); kb/tools/probe-project.md step "prepare = enrich ∘
  normalize"; kb/tools/probe-merge.md phases 2/4/5.
- **Issue**: these describe PR 2/3b/3c behavior. After PR 3a the *authority*
  statements (probe-merge.md phase 1 steps 3–4, schema.md § Authority
  validation, P23's authority-boundary paragraph, glossary `version gate`,
  `probe/projected-atoms` in Registered schema values) are implemented and
  verified true against the code — they leave the divergence list.
- **Recommendation**: none beyond executing the plan's remaining PRs; listed
  so the window stays explicit.

## Info

### [I1] ADR-003 body retains superseded application wording
Carried over (accepted: historical record behind a supersession banner).

### [I2] Version-gate producer names are duplicated between ADR-006 and code
- **Location**: kb/decisions/006-correspondence-records.md Decision 7 table;
  src/authority.rs `GATE_FLOORS`/`PROBE_GATE_MIN`/`PROBE_GATE_CEILING`.
- **Note**: the constants are intentionally compiled in (Decision 10 —
  composers link them via their pinned hub dependency), so the duplication is
  by design; the unit-test matrix pins each number, which is the drift guard.
  No action.

## Fixed during this audit

- kb/engineering/schema.md § Authority validation: added the validator
  edge-case contract (fail-closed missing/malformed `tool`, unparsable
  versions on gated names, presence-based `projection: null`, unknown names
  pass, atoms-only scope) — it was decided in the PR 1 cross-model review and
  implemented+tested in PR 3a but recorded only in the fix plan.
- kb/engineering/architecture.md hub module inventory: added
  `src/authority.rs` (the shared validator the KB already specced had no home
  in the component list).
- docs/schema-validation.md: registered-strings list updated to the
  provenance-shape branch split (`probe/projected-atoms`, `probe-vcvio/extract`
  no longer listed as unregistered) — found by the code-quality pass, noted
  here because it was a doc/spec contradiction.

## Verified clean (this PR's KB surfaces)

- Glossary `version gate`, `projection`, `blocker seed`, `status-origin`,
  `carrier`, `correspondence record` — defined, cross-linked, and (for the
  authority terms) consistent with the implementation landed here.
- ADR-006 Decision 7 table ↔ src/authority.rs constants — identical values
  (probe-lean 0.16.0, probe-aeneas 0.21.0, probe-leanblueprint 0.11.0,
  probe-vcvio 0.2.0, probe 0.5.0 ≤ v < 1.0.0 + `merge-atoms` rejection).
- kb/tools/probe-merge.md Phase 1 ordering (validate at load, then category
  consistency, then structural provenance flatten) matches cmd_merge.
- P9 structural-detection clause matches parse_envelope; the "inventories must
  survive loading" requirement is now enforced fail-loudly (malformed `inputs`
  errors).
- schema.md § Registered schema values / § Projection metadata match the
  emitted `probe/projected-atoms` envelope (schema-version 3.1, projection
  block) and the executable schema's branches.
- `./scripts/check-kb-links.sh` and the enum drift guard pass.

## Post-audit delta (2026-09-29, review commit 9507b3c)

- kb/engineering/schema.md structural-detection paragraph now records the
  ambiguous-provenance and empty-`inputs` rejections (implemented and
  tested in the same commit — spec and code moved together).
- schema.md § Bumping the interchange schema-version repointed at
  `parse_envelope` (`src/types.rs`): both references to the
  `src/commands/propagate.rs` check were stale after PR 3a rerouted
  `cmd_enrich` through the shared load path.
- Rejected review suggestion (uniform 3.1 stamping of hub outputs)
  recorded in merge-soundness-fix-plan.md §10 settled decisions.
