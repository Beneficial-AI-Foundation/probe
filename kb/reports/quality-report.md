---
auditor: code-quality-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB; audited against main — the branch does not modify properties.md/schema.md/ADR-006, so working-tree and default-branch property text are identical)
scope: branch la/merge-soundness-pr2-enrich-recomputation vs main (merge-soundness PR 2, issue #78 — enrichment recomputation, summary consumer contract, carrier preparation at cmd_enrich)
status: 0 critical, 1 warning, 1 info
---

## Critical

None.

## Warnings

### [W1] Staged KB clauses not yet implemented (plan-sequencing, pre-existing on main; narrowed by this PR)
- **Location**: kb/engineering/properties.md P4/P5/P8/P9/P13/P15/P23/P27 vs src/commands/{merge,project}.rs
- **Issue**: PR 1 (#63) landed the full ADR-006 spec ahead of the code. PR 2 closes the enrichment-side clauses (see Verified clean); the remainder stays scheduled in merge-soundness-fix-plan.md §9:
  - P23/P4 "merge re-enriches" (merge_atom_maps still returns unenriched output — architecture.md:31 and kb/tools/probe-merge.md describe the target state); P13/P27 correspondence records (merge still injects mapping edges, src/commands/merge.rs:135-171); P5 identity on the carrier; raw staging primitive → **PR 3b**.
  - P8 extended array coverage (`requires-dependencies` etc.); P9 provenance dedup; P15 categorized-array trim in project; project's `prepare`-then-trim (the shared `prepare_atoms` landed here; project does not call it yet) → **PR 3c**.
- **Evidence**: plan §9 table (landing order 1 → 3a → 2 → 3b → 3c); ADR-006 Decision 10 (hub PRs may land before the contract release; the §3f gate rejects this build's own 0.4.0-stamped output until 0.5.0 ships with the completed semantics).
- **Recommendation**: none for this PR — tracked by the plan.

## Info

### [I1] Transition-window usability: hub output self-gated until 0.5.0 (carried from PR 3a)
- **Location**: src/authority.rs (probe gate interval) vs Cargo.toml (0.4.0)
- **Issue**: until the 0.5.0 contract release, `probe merge`/`probe project` output is rejected by the four gated commands. Specced behavior (ADR-006 Decisions 7/10), recorded so it isn't read as a bug.

## Fixed during this audit

- README.md quick-start: `probe enrich` described as an upgrade pass and `probe summary` as three lists — rewritten to recomputation wording (avoiding enum-value restatement per the drift guard) and four lists.
- kb/engineering/architecture.md hub module inventory: `src/commands/propagate.rs` was missing entirely, and the `summary.rs` line ("entrypoints and verified dependencies") predated even the three-list contract — both rewritten to the implemented behavior.
- CLAUDE.md project structure: summary.rs line gains the `imported` partition.
- docs/consumer-guide.md: `probe summary` partition list extended with imported-verified.
- docs/testing-guide.md: enrich-integration test description updated from "upgrades" to recomputation, covering the new downgrade/status-origin/carrier-preparation tests.

## Verified clean

Checked on the changeset, at these locations:

- **P23 (core of this PR)** — `enrich_verification_status` (src/commands/propagate.rs) implements the path-based definition exactly: one seed set (`is_seed`: explicit `failed`/`unverified` OR any `status-origin`, matching the P23 quantifier "any atom carrying status-origin", including marked atoms with no verification-status at all); one reverse BFS whose propagation continues through every non-boundary caller (missing-status atoms transparent by construction) and stops at `is_trusted_boundary` (= `trusted` AND origin absent — the executable precedence from §3b, whole-atom trust over the unified `dependencies` set); labels set fresh over both `verified` and `transitively-verified` candidates (stale labels downgraded; marked `transitively-verified` rewritten unconditionally since every marked atom is a seed); seeds keep their base status (labeling touches candidates only, so `failed`/`unverified`/`trusted` are never rewritten); missing deps still treated as trusted with warnings; recomputation is a function of graph + base statuses (enrichment writes only `verified`/`transitively-verified`, neither of which feeds `is_seed`/`is_trusted_boundary`, so it is idempotent by construction — plus the retained idempotency tests). Unit tests cover plan §3 items a, b, c (incl. the spec-position variant), d, g, h, i, j, k; integration tests cover downgrade, status-origin blocking, and dotted-alias contamination end to end.
- **P23 carrier preparation** — `prepare_atoms` (propagate.rs) composes the shared `normalize_atoms` (P8) with enrichment; `cmd_enrich` applies it after authority validation, and prepare's normalize-first ordering coincides with μ's per-input ordering for a single input, per ADR-006 Decision 3.
- **P27 (inertness half)** — correspondence records never contaminate: the BFS traverses `dependencies` only; pinned by `test_maps_to_records_do_not_contaminate`. (Union half is PR 3b, W1.)
- **ADR-006 Decision 9 / kb/tools/probe-summary.md** — `summarize_atoms` computes all four partitions over code atoms: blueprint atoms skipped and excluded from `depended_upon` (entrypoint classification unaffected by binding edges — pinned by test); `imported_verified` membership is the specified conjunction (translation origin AND verified status), translated non-verified statuses appear in no list, `kernel-taint` stays local — pinned by the 2×5 status-matrix test; the four lists are disjoint by construction and serialize under the KB's documented snake_case field names; plan §3 test o present.
- **P1** — `SummaryEnvelope` unchanged apart from the `data` payload gaining a field; enrich preserves the input envelope structure (raw JSON round-trip with `data` replaced).
- **P3** — `Atom::is_stub` untouched; summary's entrypoint non-stub criterion unchanged.
- **P6/P7** — conflict rules untouched; `normalize_atoms` only changed visibility (`pub(crate)`) with identical behavior.
- **P8** — normalization now runs at the enrich boundary (new); rule itself unchanged (trailing-dot strip on keys, `dependencies`, `dependencies-with-locations`).
- **P9/P17/P19** — untouched (no loader, category, or dependency changes; Cargo.toml diff empty).
- **P10** — enrichment writes exactly one extension key (`verification-status`); all other extensions, including `status-origin` and correspondence records, pass through unchanged.
- **P14** — BTreeMap/BTreeSet/VecDeque-over-sorted-insertion throughout the new BFS; summary lists inherit BTreeMap order (sorted); `test_deterministic_output` retained.
- **P16** — the hub-side enrichment note ("recomputation, blocker seeds") is now implemented, closing that W1 sub-item; the `status-origin` stamping obligations are producer-side.
- **P24/P25** — untouched (producer-side).
- **Executable schema** — `status-origin` constrained to exactly `{"translation", "kernel-taint"}` (schemas/atom-envelope.schema.json), with positive (probe-vcvio fixture, pre-existing) and new negative validation tests.
- **Architecture** — recomputation logic stays in `propagate.rs`; summary remains read-only analysis (no enrichment on read, per the §10 settled decision); the bare-map API (`enrich_verification_status`, `prepare_atoms`) remains authority-unaware as documented, with authority validation staying at the envelope boundary (`cmd_enrich` validates before preparing).
- **Docs** — CLI help (src/main.rs Enrich/Summary), README, CHANGELOG, consumer guide, testing guide, architecture module list, and CLAUDE.md agree with the code after the fixes above; `./scripts/check-kb-links.sh` and `scripts/check-enum-drift.py` pass.
