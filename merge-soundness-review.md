---
title: Merge soundness and consistency review
date: 2026-09-22
status: findings — not yet triaged into issues
scope: probe merge (src/commands/merge.rs), enrichment (src/commands/propagate.rs), KB properties P4–P15, P23
---

# Merge soundness and consistency review

Review of `probe merge` and its interaction with enrichment against the KB
(`kb/engineering/properties.md`, `kb/tools/probe-merge.md`). One genuine
soundness bug, several spec/implementation inconsistencies. Ordered by
severity.

## 1. Stale `transitively-verified` labels after merge with mappings (soundness bug)

Enrichment (P23) runs at extract time, per tool. `probe merge --mappings`
adds new cross-language dependency edges afterwards
(`src/commands/merge.rs:132-168`), but `cmd_merge` never re-runs enrichment
and neither the KB nor the docs require running `probe enrich` on the merged
output.

Re-running `probe enrich` would not fix it: `enrich_verification_status`
only upgrades. Step 5 (`src/commands/propagate.rs:152-166`) counts
contaminated verified atoms as "local" but never rewrites an existing
`transitively-verified` back to `verified` (`is_verified` puts
`transitively-verified` atoms in the verified set, and only
non-contaminated atoms are written).

Consequence: a cross-language edge from a `transitively-verified` atom to
an `unverified`/`failed` atom makes the label false by P23's own definition
("every transitively reachable dependency is verified or trusted"), and no
tool in the pipeline can correct it.

Fix direction: make enrichment recompute status from scratch (downgrade
when contaminated), and either re-enrich inside `cmd_merge` when mappings
are applied or mandate `probe enrich` after `merge --mappings` in the KB.

## 2. Mapping application violates P15 as written

P15: `dependencies` must equal the union of the categorized subsets
(`requires-`/`ensures-`/`body-dependencies`; `type-`/`term-dependencies`),
"never a superset". Mapped edges are inserted into `dependencies` only
(`src/commands/merge.rs:163`), so merged-with-mappings output always breaks
the equality.

P15 is prefixed "for probe-verus `extract` output", so it is arguably
scoped to extractor output — but the "MUST always equal" wording is
absolute, and a consumer recomputing the union will disagree with merged
files. Resolve by scoping P15 explicitly to extractor output in the KB, or
by recording mapped edges in a separate field.

## 3. No staleness or provenance-consistency check in merge

ADR-002 gives the envelope `source.commit` and `timestamp` for staleness
detection, but merge never compares commits or timestamps across inputs.
Freshness is proxied entirely by argument order:

- Specs/proofs are last-wins (P7, justified as "re-running overrides stale
  results") — nothing verifies the last file is the fresher one. Wrong
  argument order silently overrides fresh proof results with stale ones
  (a `conflicts` stat and exit code 0).
- Atoms are first-wins; same exposure in the other direction.
- Atoms from commit X merge silently with proofs from commit Y.

Fix direction: warn (or fail without `--force`) when inputs' provenance
disagrees on repo/commit, or when a last-wins override has an older
timestamp than the entry it replaces.

## 4. P10 cannot deliver the leanblueprint contract

Merge picks whole atoms and never unions fields. Real-vs-real conflicts
drop the incoming atom's extensions entirely; stub replacement drops the
base stub's extensions. P10 as implemented means "the winning atom's
extensions survive".

But `kb/engineering/properties.md` (single-probe invariants, per ADR-005)
delegates probe-leanblueprint's obligation — additive blueprint fields
survive merge — to P10. If a blueprint-enriched atom map and the plain
probe-lean map contain the same real atom, first-wins discards one side's
fields wholesale: either the blueprint annotations or the verification
statuses, depending on argument order.

Fix direction: decide whether merge should union extensions on conflicting
real atoms (field-level merge), or whether the leanblueprint flow must
never re-merge with its own input; state the decision in the KB.

## 5. P4 (associativity) and P5 (identity) claimed unconditionally, hold only for plain merge

- With `--mappings`, grouping matters: the P13 existence check runs against
  the current invocation's merged key set, so
  `merge(merge(A,B,m), C, m)` and `merge(A,B,C,m)` can differ — edges
  dropped in the inner merge are only retried if mappings are re-passed,
  and edges added in the first pass become dependencies that get
  mapping-chased in the second.
- P5 identity holds only up to normalization: dotted keys are rewritten
  even when merging against an empty input.
- Merge is not idempotent at the envelope level: provenance entries
  duplicate when the same source appears twice.

`kb/tools/probe-merge.md` §Categorical framework and P4/P5 state the laws
without these caveats. Fix direction: caveat the KB (laws hold for plain
merge, up to normalization, modulo provenance) or make the implementation
match (dedup provenance; document mapping non-associativity).

## 6. Silent, inconsistent within-file key collisions after normalization

If `f()` and `f().` both exist in one input and both are real atoms,
`normalize_atoms` keeps the first and drops the second with no warning and
no conflict count (`src/commands/merge.rs:60-68`). `normalize_generic` has
the opposite policy — unconditional insert, last-wins
(`src/commands/merge.rs:215`) — also uncounted. Two uncounted, mutually
inconsistent collision policies, both silently losing data.

## Minor observations

- Conflict detection is key-only: byte-identical atoms from overlapping
  extracts count as "conflicts" (noise), while genuine divergence is only
  a stderr warning with exit code 0. Content-equality comparison would
  separate the two.
- P8 normalization covers `dependencies` and `dependencies-with-locations`
  but not the other code-name-bearing extension arrays
  (`requires-dependencies` etc.); legacy dotted names there would silently
  break the P15 union.

## Design question (not a bug)

Mapping edges attach to the *callers* of a mapped atom, never to the atom
itself: Rust `main` gains a dependency on `Lean.reduce`, but Rust `reduce`
never depends on its own translation. A leaf function with no callers gets
no cross-language linkage at all, and a Rust function's own status is never
influenced by its Lean counterpart's proof status. If the intent is "this
Rust function is verified via its Lean proof", the current edge placement
does not express it.
