---
auditor: ambiguity-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB)
scope: branch la/merge-soundness-pr3c-collision-warnings-projection vs
  origin/main (merge-soundness PR 3c, issue #82 — specs/proofs intra-input
  collision warnings, provenance dedup, minItems 2→1, P8 categorized-array
  normalization, P15 projection trim, carrier preparation in probe project,
  ValidatedAtomFile.projected, 0.5.0 contract release, round-trip tests).
  Per §2b: audited against origin/main's KB text; main already carried the
  PR 3b spec-ahead statements (the previous report's W3 window: P9 dedup,
  P15 trim clause, P23 project enrich-then-trim, glossary projection entry,
  probe-project.md steps 4–5), which this branch's code implements. The
  branch's own KB edits (P8 specs/proofs paragraph + project rejection
  clause, probe-merge.md Phase 2 addition, probe-project.md Step 2 rejection
  sentence) land in the same PR as the implementing code — same-PR
  consistency is the relevant check, and it holds; no cross-PR merge-order
  constraint arises.
status: 0 critical, 2 warnings, 2 info
---

## Critical

### [W3 of the PR 3b report] ~~Remaining spec-ahead-of-code statements, all PR 3c~~ — CLOSED by this branch
- **Resolution**: every item in the window is now implemented and verified
  against the code: P8's extended arrays (`CATEGORIZED_DEPENDENCY_ARRAYS` in
  src/types.rs, normalized in `normalize_atoms`, matching P8's five-entry
  list exactly), P9's dedup clause (`dedup_provenance`, applied in
  `merge_atom_files` and `cmd_merge`), P15's projection trim
  (src/commands/project.rs, same filter as `dependencies`, non-string
  entries untouched), P23's projection enrich-then-trim (`prepare_atoms` on
  authoritative inputs in `cmd_project`, skipped for projected inputs via
  the new `ValidatedAtomFile.projected` flag), and the glossary/tool-page
  statements those properties anchor.
- **The known interim window is verified closed**: `load_mappings`
  normalizes endpoints and `prepare_atoms` normalizes atom keys before seed
  matching, so a dotted-key legacy atom now selects its normalized seed —
  pinned by tests/roundtrip.rs::`projection_seeds_match_normalized_keys`
  over the real binary (passes; full `cargo test` green, 205 tests).
  Retained this round so the closure is on record; drops off next run.

No new criticals. The branch's new KB text (P8 ¶2 project-rejection
extension, P8 ¶3 specs/proofs warn-and-count, probe-merge.md Phase 2,
probe-project.md Step 2) *extends* origin/main's text — main's P8 was silent
on `probe project`'s collision handling and on specs/proofs collisions — and
each clause was verified against the implementing code: `cmd_project`
rejects on non-empty `PrepareStats::dropped_atoms` exactly as `cmd_enrich`
does; `normalize_generic` warns and feeds `stats.conflicts` while keeping
last-wins (P7), pinned by unit test and by
tests/roundtrip.rs::`specs_intra_input_collision_warned_and_counted`.

## Warnings

### [W1] probe-verus spec-position dependency arrays are empty in real artifacts
Carried over, re-verified: ADR-006 Decision 4 still records that the
producer audit found `requires-dependencies`/`ensures-dependencies` empty in
real probe-verus artifacts, and no probe-verus change lands with this PR.
Now mildly sharper: this PR's P8 normalization and P15 trimming of exactly
those arrays are vacuous on real Verus output until the producer emits them
(the hub-side machinery is tested on synthetic atoms). Producer defect, not
a hub KB defect; tracked on the probe-verus side.

### [W2] Presentation decks still describe edge injection
Carried over, re-verified: on branch `la/probes-slides`,
`docs/slides-lean-verification-landscape.md` line 135 still says "computed
from the cross-language edges the merge flattened through the proof graph" —
the pre-ADR-006 edge-injection semantics. Local-only decks, updated when
next regenerated; not on this PR's surface.

## Info

### [I1] P9's "not … in what order" vs first-occurrence inventory order
- **Location**: kb/engineering/properties.md P9 ¶2; src/types.rs
  `dedup_provenance`.
- **Issue**: P9 says the `inputs` array records which sources were composed,
  "not how many times or in what order". The implementation deduplicates but
  keeps first-occurrence order, so `μ(A,B)` and `μ(B,A)` emit
  differently-ordered (set-equal) inventories. That is consistent with the
  laws — P4 is stated "over provenance as a deduplicated source inventory",
  i.e. compared as sets — and deterministic per P14 (order is a function of
  argument order), but a reader could take the phrase as mandating a
  canonical (sorted) array.
- **Recommendation**: optional one-clause clarification in P9 ("the emitted
  array keeps first-occurrence order; inventory equality is set equality").
  Not blocking — the executable behavior contradicts no stated law.

### [I2] Merged-envelope `schema-version` differs by producer generation
Carried over unchanged from the PR 3b report: hub merged/projected output is
stamped `3.1` while producers emit `3.0`; mixed stamps in one pipeline are
the expected steady state per schema.md's hub-side-minor-bump policy.
Re-checked against `cmd_merge`/`cmd_project` (both stamp 3.1) and
`parse_envelope` (accepts any 3.x).

## Fixed during this audit

- kb/engineering/glossary.md: added the `carrier preparation` entry — the
  term (`prepare = enrich ∘ normalize`) is load-bearing in ADR-006,
  architecture.md, probe-project.md Step 2, and this release's CHANGELOG,
  but was undefined in the glossary (whose own rule is that every domain
  term is defined there; same gap the PR 3b audit closed for "raw staging
  primitive"). Entry names the boundaries that prepare, the projected-input
  exemption, and the fail-closed collision rule.
- kb/tools/probe-project.md Step 1 and the input-restriction bullet named
  `load_atom_file()` as the loader; `cmd_project` has routed through
  `load_validated_atom_file()` since PR 3a, and this PR makes the
  distinction semantic (the validated loader now returns the `projected`
  flag Step 2 depends on). Corrected both references and noted the flag.
- kb/tools/probe-project.md `last-updated` bumped 2026-09-28 → 2026-09-30
  (Step 2 substantively edited by this branch).
- kb/tools/probe-merge.md stats table: the specs/proofs "Conflicts" cell
  said only "overrides, incoming kept", but this PR makes the same stat
  also count intra-input post-normalization collisions (the page's own
  Phase 2 documents this); the cell now names both feeds. The atoms cell
  was already accurate ("intra-input distinct-real collisions are errors,
  not counts").
- `./scripts/check-kb-links.sh` and the enum drift guard re-run clean after
  the edits.

## Verified clean (this delta)

- **P7 (last-wins) × P8 (collision surfaced)**: the new P8 ¶3 and
  probe-merge.md Phase 2 state warn-and-count with last-wins kept;
  `normalize_generic` implements exactly that (warning text names P7/P8;
  collisions feed `stats.conflicts`); pinned at unit level
  (`test_generic_intra_input_collision_counted`) and over the real binary
  (roundtrip `specs_intra_input_collision_warned_and_counted`, which also
  checks the printed "Conflicts: 1").
- **P8 array list vs code**: P8's "Normalization is applied to" list names
  the five categorized arrays; `CATEGORIZED_DEPENDENCY_ARRAYS` matches
  entry-for-entry, and `normalize_atoms` applies it with the documented
  non-string pass-through.
- **P8 distinct-real rejection at all three boundaries**: merge (per input,
  `collision_error` with 1-based input prefix), enrich (`cmd_enrich` on
  `dropped_atoms`), project (`cmd_project`, same rule and message shape) —
  matching P8 ¶2's amended three-boundary enumeration; probe-project.md
  Step 2 and schema.md § Authority validation agree.
- **P9**: `dedup_provenance` (first occurrence kept; entries differing in
  flattened `source` extensions stay distinct — pinned by
  `dedup_provenance_collapses_identical_entries_only`) applied at both
  merge boundaries; structural composed-detection unchanged; schema.md's
  "deduplicated source inventory" clause and the executable schema's new
  `inputs` description + `minItems: 1` state the same rule; envelope
  idempotence pinned over the real binary (roundtrip).
- **P14**: dedup is order-deterministic; projection BFS unchanged over
  BTreeMap/BTreeSet; no new HashMap-iteration serialization.
- **P15**: preservation clause ("trims the categorized extension arrays
  with the same filter") is now code (project.rs) with the decomposition
  regression (`test_categorized_arrays_trimmed_with_dependencies`);
  probe-project.md Step 5 and the properties bullet agree.
- **P23**: probe-project.md Step 2, ADR-006 Decision 6, schema.md § 369,
  and the glossary `projection` entry all state prepare-before-trim +
  inherited labels for projected inputs; `cmd_project` implements both
  branches (the `projected` flag), with the Verus-shaped stale-label
  regression pinned over the real binary
  (`projection_recomputes_enrichment_before_trimming`).
- **P27 inertness in projection**: "never traversed by projection BFS" is
  pinned by `test_selection_is_dependency_only` (the pre-ADR-006
  fabricated-edge reachability is not reproduced); probe-project.md Step 4
  states the same.
- **0.5.0 contract release coherence**: Cargo.toml `0.5.0` =
  `PROBE_GATE_MIN` = ADR-006 Decision 7 table row = schema.md's merged
  example = CHANGELOG 0.5.0 header; the round-trip suite proves the hub's
  real outputs pass its own gate (only true from this release) and that a
  real projection is rejected on the projection predicate specifically,
  not the gate.
- **Enumerated lists vs code**: probe-merge.md stats table matches
  `print_stats` labels (after the specs-cell fix above); probe-merge.md's
  "two or more" input arity matches the CLI (`num_args = 2..`).
- **Glossary consistency**: `projection`, `seed set`, `carrier`,
  `raw staging primitive`, `version gate`, `correspondence record` all
  match the implemented semantics; `carrier preparation` added (above).
- **ADR-005 ownership**: nothing single-probe leaked into shared files;
  the P15 scope note still correctly scopes the decomposition to
  subset-carrying producers.
- Full `cargo test` green (205 tests across unit + integration suites,
  including the 7 round-trip tests over real binary outputs);
  `./scripts/check-kb-links.sh` and `scripts/check-enum-drift.py` pass.
