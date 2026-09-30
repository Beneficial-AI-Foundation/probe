---
auditor: ambiguity-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB)
scope: branch la/merge-soundness-pr3b-maps-to-records (merge-soundness PR 3b —
  correspondence records, merge re-enrichment, raw staging, P8 reconciliation),
  delta audit over the PR 2 report; every KB surface this PR touches or
  implements re-checked against the code. Second pass 2026-09-30 over the
  PR #81 review-fix surfaces (P27 canonical form + fail-closed validation:
  P27, schema.md §§ Correspondence records / Mappings file format, ADR-006
  Decision 1, probe-merge.md Phase 2, glossary).
status: 0 critical, 3 warnings, 2 info
---

## Critical

### [C2 of the PR 2 report] ~~`merge --mappings` injects dependency edges, contradicting revised P13~~ — CLOSED by this branch
- **Resolution**: the edge-injection block is removed; `attach_correspondence_records`
  (src/commands/merge.rs) implements P13's unconditional/key-local/set-like
  attachment, with the union rule (P27) on every equal-key resolution and the
  record definitions in the executable schema. The old tests pinning the
  superseded semantics were replaced, not amended. Retained this round so the
  closure is on record; drops off next run.

No new criticals: the P8/P27 text amendments on this branch and the code that
implements them were written together, and the amendment is the reconciliation
the plan's §9 row 3b explicitly mandates (recorded in ADR-006 Decision 3 and
plan §10). The quality report carries the merge-order constraint.

## Warnings

### [W1] probe-verus spec-position dependency arrays are empty in real artifacts
Carried over unchanged (producer defect, filed in the probe-verus rollout
issue; recorded in ADR-006 Decision 4). Not a hub KB defect.

### [W2] Presentation decks still describe edge injection
Carried over unchanged (local-only decks, updated when next regenerated).

### [W3] Remaining spec-ahead-of-code statements, all PR 3c
- **Location**: properties.md P8 (extended arrays `requires-dependencies` etc.;
  project boundary), P9 (dedup clause), P15 (projection trims categorized
  arrays), P23 (projection enrich-then-trim); glossary `projection`
  ("recomputed on the full input before trimming"); kb/tools/probe-project.md
  steps 3–4 and the `prepare = enrich ∘ normalize` step.
- **Issue**: these describe PR 3c behavior. After PR 3b the merge-side
  statements — P4/P5 law statements, P13/P27 attachment and union, P23's
  "merge re-enriches", schema.md § Correspondence records and the
  re-enrichment clause, kb/tools/probe-merge.md phases 2/4/5 and its stats
  table, the glossary's `merge`/`mapping`/`correspondence record`/`carrier`
  entries, and kb/tools/probe-aeneas.md's hub-API description (records-first
  `load_mappings`, derived endpoint indexes) — are implemented and verified
  against the code; they leave the divergence list.
- **Sharpened within the window**: kb/tools/probe-project.md step 4 ("seeds …
  normalized, that exist in the (normalized) atom data") now *actively*
  diverges rather than merely leading the code: this PR's `load_mappings`
  normalizes endpoints, but `project_atoms` still matches them against raw
  atom keys, so a dotted-*key* legacy atom selects zero seeds — the interim
  regression plan §3d predicted and assigns to 3c. Land 3c promptly.
- **Recommendation**: none beyond executing PR 3c; listed so the window stays
  explicit.

## Info

### [I1] "identical-modulo-records" defined inline, used in three files
P8 defines distinctness as "judged ignoring `maps-to`/`mapped-from`" and P27,
probe-merge.md Phase 2, and ADR-006 Decision 3 reuse the phrase (probe-merge
paraphrases as "differ beyond their correspondence records"). Consistent at
every use site and anchored by P8's definition; below the glossary-entry bar
since it is a predicate of one property, not a standalone domain term. No
action.

### [I2] Merged-envelope `schema-version` now differs by producer generation
Hub merged output is stamped `3.1` while producers keep emitting `3.0`
(schema.md's stated policy: hub-side minor bump, 3.x additive). Verified
consistent across schema.md (§ current version, version-history row, merged
example — the example was updated during this audit's quality pass), the
executable schema (`^[0-9]+\.[0-9]+$`, both accepted), `cmd_merge`
(stamps 3.1), `probe project` (3.1 since PR 3a), and `parse_envelope`
(accepts any 3.x). Recorded because mixed 3.0/3.1 stamps in one pipeline are
now the *expected* steady state, not drift.

## Fixed during this audit

- kb/engineering/glossary.md: added the `raw staging primitive` entry — the
  term is load-bearing in ADR-006 Decision 3, schema.md, and
  kb/tools/probe-merge.md but was undefined in the glossary (whose own rule is
  that every domain term is defined there). Definition names the concrete
  functions and the off-carrier caveat.
- kb/tools/probe-merge.md Phase 2 lead sentence still said per-input ordering
  "selects which atom wins a post-normalization collision" — stale against the
  same page's (and P8's) new rejection rule one paragraph below; rephrased to
  the alias-collapse-before-cross-input-resolution formulation.
- `last-updated` stamps bumped (2026-09-28 → 2026-09-29) on properties.md,
  schema.md, and probe-merge.md, all substantively edited by this branch.

Second pass (2026-09-30, review-fix surfaces):

- kb/tools/probe-merge.md Phase 2 restated P27's rejected-shape list and had
  already drifted (five items to P27's six — "non-string method" missing).
  Replaced the restated list with a short characterization plus a pointer to
  P27 as the normative list, the same delegate-don't-duplicate rule the enum
  drift guard enforces on the docs surface.

## Verified clean (this delta)

Second pass (2026-09-30, review-fix surfaces):

- **Cross-file agreement on the canonical-form clause**: P27 ("empty `method`
  ... canonicalized to absent", dedup by identity), schema.md § Correspondence
  records ("an empty `method` is canonicalized to absent, and no other fields
  are allowed"), § Mappings file format (validated at load), ADR-006
  Decision 1 (rationale: two encodings of one identity break set union and
  P4), glossary `correspondence record`, probe-merge.md Phase 2, and plan §10
  state the same rule; the executable schema's `minLength: 1` on `method` is
  the same decision in schema form.
- **"Every recomputation boundary"** in the new P27/ADR-006/schema.md
  validation clauses follows P8 ¶2's established construction — enumerated in
  place as merge (per input) + enrich. `probe project` joining at PR 3c (when
  it routes through `prepare_atoms`) is already the W3 window; validation
  rides along automatically because it lives inside `normalize_atoms`.
- **"canonical form"** is a predicate of P27 defined at first use and reused
  consistently — below the glossary-entry bar for the same reason as I1's
  "identical-modulo-records". No action.
- **Claims vs code**: the six-item rejection list in P27 matches
  `validate_and_canonicalize_records` arm-for-arm; the confidence vocabulary
  is pinned code↔schema by `confidence_vocabulary_matches_executable_schema`;
  `load_mappings`' documented load-boundary validation and `""`→absent
  canonicalization match the implementation.

- **Glossary consistency**: `merge`, `mapping`, `correspondence record`,
  `carrier`, `version gate`, `blocker seed`, `trusted (boundary)` — all match
  the implemented semantics; `correspondence record`'s "unioned through every
  merge conflict, inert to enrichment and projection" is now code, not intent.
- **Cross-file agreement** on the new collision rule: properties.md P8 ¶2,
  P27 bullet 1, schema.md § Normalization, probe-merge.md Phase 2, ADR-006
  Decision 3, and plan §10 state the same predicate (distinct-real modulo
  records ⇒ reject; benign ⇒ union) with the same rationale.
- **Enumerated lists vs code**: probe-merge.md stats table matches
  `print_stats` labels exactly (incl. the new Records attached / Enrichment
  rows); the confidence vocabulary is identical across schema.md, the
  executable schema enum, and the mappings-spec table; `status-origin`
  enforcement rows now name all three rejecting consumers (merge added).
- **ADR-005 ownership**: nothing single-probe leaked into shared files by this
  change; probe-aeneas mechanics stay on its page, the hub API contract on the
  hub's.
- `./scripts/check-kb-links.sh` and the enum drift guard pass.
