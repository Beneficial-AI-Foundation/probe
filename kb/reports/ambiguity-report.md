---
auditor: ambiguity-auditor
date: 2026-09-28
repo: probe (hub)
kb: kb/ (own KB)
scope: branch la/merge-soundness-pr1-adr006 (ADR-006 spec change, PR 1 of the
  merge soundness fix plan), audited as working tree; note this branch amends
  the KB ahead of the code — see C1/C2 for the merge-order framing
status: 2 critical (both scheduled, merge-order constraints), 3 warnings, 2 info
---

## Critical

Both critical findings are the known code-vs-spec divergences this branch
exists to schedule: PR 1 lands the KB first (KB is source of truth), and the
code PRs (3a → 2 → 3b → 3c, per merge-soundness-fix-plan.md §9) bring the
implementation into conformance. They are listed so the divergence window is
explicit, not because PR 1 can or should fix them.

### [C1] Enrichment implementation contradicts P23 on both branches
- **Location**: src/commands/propagate.rs:14-16, 127-150, 152-166
- **Issue**: The implementation makes missing-status atoms **opaque** to
  contamination (propagation continues only through callers in the verified
  set) and is upgrade-only (a contaminated atom arriving as
  `transitively-verified` keeps its label). This contradicts the revised P23
  (path-based recomputation) **and** the default-branch P23, whose key rule
  already said missing-status atoms are "transparent".
- **Evidence**: `v [verified] → helper [no status] → bad [unverified]`:
  contamination stops at `helper`; `v` is labelled `transitively-verified`.
  Confirmed independently by the 2026-09-28 probe-verus producer audit
  (counterexample reproduced against the pinned v0.4.0 propagate.rs).
- **Recommendation**: PR 2 (BFS recomputation over the unified seed set).
  Merge-order constraint: PR 3a (authority boundary) must land before PR 2,
  per the plan.

### [C2] `merge --mappings` injects dependency edges, contradicting revised P13
- **Location**: src/commands/merge.rs:133-161
- **Issue**: The code adds mapped code-names to `dependencies` (existence-
  checked, both directions) — exactly the semantics revised P13/ADR-006
  supersede. The `@kb:` annotation at merge.rs:75 now points at the new P13,
  so the annotation and the code it annotates disagree until PR 3b lands.
- **Evidence**: `test_mappings_add_cross_language_edges` (merge.rs:629) pins
  the old behavior.
- **Recommendation**: PR 3b (record attachment, edge-injection removal, test
  replacement). Until then the KB describes target behavior; regenerated
  artifacts must wait for the version gate (PR 3a) + PR 3b.

## Warnings

### [W1] probe-verus spec-position dependency arrays are empty in real artifacts
- **Location**: ../probe-verus/src/lib.rs:1526-1547 (`classify_call_location`);
  evidence in ../probe-verus/examples/verus_curve25519-dalek_4.1.3.json
- **Issue**: P15 documents `requires-dependencies`/`ensures-dependencies` as
  categorized subsets, but the 2026-09-28 producer audit found **zero** atoms
  in the entire dalek example carrying either array — the location
  classification is degenerate (everything lands in `Inner`). The
  decomposition equality holds vacuously; the spec-position closure of a
  trusted atom is unobservable, which weakens the ADR-006 Decision 4 audit
  trail (the spec-position counterexample is unrealizable partly *because* of
  this defect).
- **Recommendation**: File in the probe-verus rollout issue (§8). Not a hub
  KB defect; recorded in ADR-006 Decision 4.

### [W2] Presentation decks still describe edge injection
- **Location**: docs/lightning-talk-probes.md:51,121; docs/slides23-31.md:36
  (deliberately local-only since 2026-09-29: gitignored per the docs
  cleanup plan's decision 4, so unverifiable from a clean clone — the
  line references apply to the local working copies)
- **Issue**: Both say merge "adds/stitches cross-language edges".
  Contradicts ADR-006 once PR 3b lands.
- **Recommendation**: Update when the decks are next regenerated; they are
  not normative and not part of PR 1.

### [W3] ADR-003 body retains superseded application wording
- **Location**: kb/decisions/003-mappings-design.md:19,61 (Context,
  Consequences)
- **Issue**: The body still speaks of "cross-language dependency edges". A
  supersession banner was added at the top; the body is kept as a historical
  record per ADR practice, but a reader skipping the banner can be misled.
- **Recommendation**: Accept (historical record). The banner names ADR-006
  and the exact boundary (application vs generation semantics).

## Info

### [I1] Literal `3.0` values in examples are correct, not stale
- **Location**: kb/engineering/schema.md envelope examples; architecture.md
  data-flow diagram
- **Note**: Producers keep emitting `schema-version: "3.0"` (the 3.1 bump is
  hub-side; 3.x is additive), so example envelopes showing `"3.0"` are
  accurate. Generic references to the format family were normalized to
  "Schema 3.x" during this audit.

### [I2] Categorical framework now labeled as analogy
- **Location**: kb/engineering/categorical-framework.md (intro, laws,
  closure claims; at audit time `docs/categorical-framework.md`, moved
  by #65)
- **Note**: The DOTS/SSProve tables are explicitly labeled architectural
  analogies; laws restated on the carrier per P4/P5. Non-normative doc,
  consistent with the KB after this change set.

## Fixed during this audit

- probe-verus tool page said enrichment "upgrades" (now: recomputes).
- architecture.md and product/spec.md described probe-lean statuses as
  warning-based sorry detection (now: kernel-based, per P16; External-module
  trust excludes proofs; `externally_verified` added to glossary/trust-base
  entries and P22, with the consumer mapping added to
  scripts/summarize_extract.py `TRUST_LABELS`).
- probe-summary.md "verified lemmas" said "Verus proof/spec atoms" only
  (code and test o include Lean declarations; now stated, with the blueprint
  exclusion).
- Undefined load-bearing terms added to the glossary: `correspondence
  record`, `status-origin`, `blocker seed` (disambiguated from projection
  `seed set`), `carrier`, `version gate`; `trusted`/`trust base` entries
  updated.
- Stale anchors after property retitles (P4/P5/P13/P23) repointed across
  kb/, docs/, and `@kb:` annotations in src/; `./scripts/check-kb-links.sh`
  passes clean.
- Load-bearing "Schema 3.0" references normalized to "Schema 3.x".

## Verified clean

Property set resolved from kb/engineering/properties.md at audit time:
P1–P10, P13–P17, P19, P21–P25, P27 (P11/P12/P18/P20/P26 migrated per
ADR-005 — confirmed against the Single-probe invariants section).

- **P1** consistent with schema.md 3.x wording after fix.
- **P2, P3, P6, P7, P14, P17, P19, P21, P24, P25** — unchanged by this
  change set; spot-checked wording against schema.md and glossary; no
  contradictions found.
- **P4/P5** — carrier formulation consistent across properties.md,
  probe-merge.md, categorical-framework.md, ADR-006 (μ/F_M, same law set).
- **P8** — extension-array list matches schema.md's extension documentation
  (requires/ensures/body/type/term + dependencies-with-locations + record
  targets + mapping endpoints).
- **P9** — inventory/structural-detection wording consistent between
  properties.md, schema.md, ADR-006; blueprint scope note present.
- **P10 vs P27** — carve-out is explicitly scoped (records only); no
  contradiction with P10's whole-extension preservation.
- **P13, P23, P27** — internally consistent and consistent with ADR-006,
  schema.md, glossary, and the tool pages (checked pairwise: seed
  definition, trusted-boundary precedence, unconditional demotion,
  projection labels, code-atom scoping, summary contract).
- **P15** — scope and preservation clauses consistent with
  probe-project.md's trim step (W1 tracks the producer-side defect).
- **P16** — kernel-based contract consistent with probe-lean.md and the
  glossary; the Verus table unchanged and consistent with P24.
- **P22** — table extended (`attested`); consumer mapping updated in the
  same change.
- Cross-file link integrity: `./scripts/check-kb-links.sh` — all KB links OK.
