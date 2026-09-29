---
auditor: code-quality-auditor
date: 2026-09-29
repo: probe (hub)
kb: kb/ (own KB; audited against main — the branch does not modify properties.md/schema.md, so working-tree and default-branch property text are identical)
scope: branch la/merge-soundness-pr3a-authority-boundary vs main (merge-soundness PR 3a, issue #76 — authority boundary: projection rejection, version gate, structural provenance)
status: 0 critical, 1 warning, 2 info
---

## Critical

None.

## Warnings

### [W1] Staged KB clauses not yet implemented (plan-sequencing, pre-existing on main)
- **Location**: kb/engineering/properties.md P4/P5/P8/P9/P13/P15/P23/P27 vs src/commands/{merge,propagate,project}.rs
- **Issue**: PR 1 (#63) landed the full ADR-006 spec ahead of the code, so several KB clauses are ahead of the implementation. Not introduced by this changeset and each is assigned to a scheduled PR in merge-soundness-fix-plan.md §9:
  - P23 "enrichment recomputes" / unified seed set / `status-origin` blocker seeds; P16 enrichment note — `enrich_verification_status` is still upgrade-only (propagate.rs) → **PR 2**.
  - P23/P4 "merge re-enriches"; P13/P27 correspondence records (merge still injects mapping edges, merge.rs:135-171); P5 identity on the carrier; raw staging primitive → **PR 3b**.
  - P8 extended array coverage + unary-boundary normalization; P9 provenance dedup; P15 categorized-array trim in project; project enrich-then-trim → **PR 2/3c**.
- **Evidence**: plan §9 table (landing order 1 → 3a → 2 → 3b → 3c); ADR-006 Decision 10 ("Hub PRs may land before producer releases; the gate enforces the ordering mechanically").
- **Recommendation**: none for this PR — the §3f gate implemented here is the mechanism that makes the interim state safe (pre-contract hub output, including this build's own 0.4.0-stamped merges, is rejected until the 0.5.0 contract release ships with the completed semantics).

## Info

### [I1] Single-tool fallback still fabricates an empty `source`
- **Location**: src/types.rs parse_envelope (single-tool branch)
- **Issue**: a single-tool envelope with a missing/malformed `source` gets a synthesized empty provenance entry rather than an error. Pre-existing behavior, retained; malformed `inputs` on the composed path was made a hard error during this audit (P9).
- **Recommendation**: consider tightening alongside PR 3c's provenance work.

### [I2] Transition-window usability: hub output self-gated until 0.5.0
- **Location**: src/authority.rs (PROBE_GATE_MIN) vs Cargo.toml (0.4.0)
- **Issue**: between this PR and the 0.5.0 contract release, output of `probe merge`/`probe project` (stamped `probe` at 0.4.0) is rejected by all four gated commands. This is the specced behavior (ADR-006 Decision 7/10: pre-contract hub merges are pre-contract evidence), recorded here so it isn't read as a bug.

## Fixed during this audit

- docs/schema-validation.md: registered-strings list contradicted the reworked executable schema (missing `probe/projected-atoms`, `probe-vcvio/extract`, provenance-shape branch split) — updated.
- kb/engineering/architecture.md hub module inventory and CLAUDE.md project structure: added `src/authority.rs` (and the missing `project.rs`/`propagate.rs` lines in CLAUDE.md).
- src/types.rs: malformed `inputs` array now errors instead of silently emptying the inventory (P9).

## Verified clean

Checked on the changeset, at these locations:

- **P1** — projected envelope carries all required fields, schema-version 3.1 (src/commands/project.rs ProjectedEnvelope); merge/summary envelopes unchanged.
- **P3** — `Atom::is_stub` untouched (src/types.rs).
- **P6/P7** — conflict rules untouched (src/commands/merge.rs merge_atom_maps/merge_generic_maps); P4's "projected artifacts rejected, not normalized" now enforced at cmd_merge, merge_atom_files, cmd_enrich (src/authority.rs validate_authority, Recompute scope).
- **P9 (structural detection)** — parse_envelope keys composed detection on the presence of `inputs`, never a schema prefix (src/types.rs); test m (tests/authority.rs aeneas_composed_envelope_inventory_survives_merge) pins the probe-aeneas/extract inventory through load → merge → validate.
- **P14** — validator and loader changes use no unordered iteration; BTreeMap throughout.
- **P16** — status table untouched; the enrichment note's recomputation clause is W1/PR 2.
- **P17** — category-consistency check intact in cmd_merge, and runs after per-input authority validation, matching kb/tools/probe-merge.md phase 1 order.
- **P19** — no dependency changes (Cargo.toml diff is empty).
- **P23 authority-boundary paragraph** — merge and enrich reject both projection formats and pre-contract envelopes; summary and project run the version-gate component only; projections stay readable read-only (tests f, l, n in tests/authority.rs; unit matrix in src/authority.rs tests). Gate table values match ADR-006 Decision 7 exactly (probe-lean 0.16.0, probe-aeneas 0.21.0, probe-leanblueprint 0.11.0, probe-vcvio 0.2.0, probe 0.5.0 ≤ v < 1.0.0 + `merge-atoms` command rejection). Validator edge cases per the plan's §3d contract: missing/malformed `tool` and unparsable versions on gated names reject; `projection: null` counts as present; unknown tool names pass; unknown schema strings keep the detect_category error.
- **P24/P25** — untouched (producer-side).
- **Architecture** — the validator is a single shared function invoked at every hub envelope boundary (no per-command reimplementation); bare-map APIs (`merge_atom_maps`, `enrich_verification_status`) remain authority-unaware as documented; read-only vs recomputation scopes match ADR-006 Decision 6/7.
- **Docs** — CLI help (src/main.rs), CHANGELOG, kb/tools/probe-merge.md phases, kb/tools/probe-project.md output schema, and kb/engineering/schema.md registered values all agree with the code after the fixes above; `./scripts/check-kb-links.sh` passes.

## Post-audit delta (2026-09-29, review commit 9507b3c)

The PR #77 cross-model review (codex-critique) landed fail-closed
tightenings after this audit's pass — extensions of the audited behavior,
not regressions of it:

- `parse_envelope` rejects ambiguous provenance (both `source` and
  `inputs` present; previously `inputs` silently won and `source` was
  dropped) and empty `inputs` inventories (previously propagated into
  output violating the executable schema's `minItems: 1`). Specced in
  schema.md's structural-detection paragraph, same commit.
- The version-gate parser accepts ASCII-digit components only (a leading
  `+`, tolerated by `u64::from_str`, no longer parses).
- `load_atom_file` now documents that it bypasses authority validation.
- I1 (single-tool missing-`source` fabrication) is unchanged — still
  scheduled alongside PR 3c's provenance work.
