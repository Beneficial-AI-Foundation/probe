---
auditor: ambiguity-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB, resolved via common §1.1)
scope: uncommitted working-tree changeset on branch
  la/merge-soundness-pr3c-collision-warnings-projection (PR #83), addressing
  PR #83 review comments — P7 "last" = input-file order, P8 project rejects
  distinct-real collisions on any input, P8 categorized dependency arrays are
  sets (sort + dedup after normalization), P8 specs/proofs intra-input
  collision warning + key-order tie-break; kb/tools/probe-merge.md Phase 2 and
  statistics table; strict `normalize_atoms`, `prepare_atoms` strict,
  `PrepareStats::dropped_atoms` removed. Per §2b: branch is not on the default
  branch; P7/P8 were compared against `origin/main` (Cargo 0.4.0), where P7 has
  no input-order clarification and P8 has no project clause, no set-semantics
  paragraph and no specs/proofs collision paragraph. All three P8 additions and
  the P7 sentence are amendments carried by PR #83 itself (committed 5623ae9,
  b33d9d0 + this working tree).
status: 0 critical, 5 warnings, 4 info
---

## Critical

None. No KB file states a rule that the amended P7/P8 or the code contradicts;
the drift found is omission and terminology, listed under Warnings.

## Warnings

### [W1] Merge-order constraint: the code conforms only to P7/P8 amendments unmerged on main
- **Location**: kb/engineering/properties.md:74 (P7), :82, :84, :86 (P8); src/commands/merge.rs:301-309, 381-385, 625-652; src/commands/project.rs:251-269
- **Issue**: Against `origin/main`'s properties.md, three behaviours of this changeset have no spec: `probe project` rejecting distinct-real collisions on *projected* inputs (main's P8 names only merge and enrich), sorting/deduplicating the categorized dependency arrays (main's P8 says nothing about order), and the specs/proofs intra-input collision warning with a key-order tie-break (main's P8 has no specs/proofs paragraph; main's P7 says only "last one wins"). The code is conformant only to the branch text.
- **Evidence**: `git diff origin/main -- kb/engineering/properties.md` shows all four passages as additions; `git log origin/main..HEAD` = 5623ae9, b33d9d0 (both PR #83).
- **Recommendation**: Both sides are in the same pull request (PR #83), so the constraint is satisfied if the working-tree KB edits are committed together with the code changes. Do not split them. The set-semantics paragraph is a new hub-level statement about producer-owned fields (see W5) and needs explicit human sign-off as a spec refinement, per CLAUDE.md.

### [W2] schema.md § Normalization and § Specs and proofs are stale against amended P8
- **Location**: kb/engineering/schema.md:310 (Normalization), :283-288 (Specs and proofs: last-wins)
- **Issue**: The normalization paragraph still gives a narrower rule than P8. It describes the distinct-real collision as "a merge **error**, the same fail-closed rule `probe enrich` applies" but leaves out `probe project` (both input kinds). It doesn't say that the categorized dependency arrays are sorted and deduplicated after normalization. It also has nothing on specs/proofs intra-input collisions, which are warned about, counted in `conflicts`, and resolved by key order. The specs/proofs table (base/incoming → incoming wins) is correct for cross-input merges but gives no hint that intra-input aliases follow a different tie-break.
- **Evidence**: schema.md:310 text; P8 at properties.md:82-86; probe-merge.md:49-51 already carries all three.
- **Recommendation**: Extend schema.md:310 with the project clause, the sort/dedup sentence, and one sentence on the specs/proofs intra-input case that links to P8. Alternatively, trim it to a pointer to P8 so the two can't drift again. Add a note under the specs/proofs table that intra-input aliases are resolved by key order ([P8](../engineering/properties.md#p8-code-name-normalization)).

### [W3] ADR-006 Decision 3 does not mention project's collision rejection (decision record; recommend, don't assume an edit)
- **Location**: kb/decisions/006-correspondence-records.md:160-163 ("Carrier preparation"), :167-180 ("Intra-input collision policy is uniform rejection")
- **Issue**: The intra-input policy paragraph lists the rejecting boundaries as "`probe merge` errors during per-input normalization exactly as `probe enrich` errors on its file". It leaves out `probe project`, which P8 now says rejects any input, authoritative or projected. The carrier-preparation paragraph speaks only of "their single authoritative input". It doesn't record that already-projected inputs are normalized too, with the same rejection, without enrichment. The glossary (`carrier preparation`, glossary.md:79) and probe-project.md Step 2 (:38) do state this.
- **Evidence**: ADR text at the cited lines vs properties.md:82 and glossary.md:79.
- **Recommendation**: The ADR is a decision record. The user should decide whether to append a dated amendment note ("PR 3c: `probe project` applies the same rejection on authoritative and projected inputs; on views a collision would silently pick one atom's inherited label") or leave the ADR as a historical snapshot and rely on P8. Do not rewrite the decision text in place.

### [W4] "Every recomputation boundary" no longer matches the boundaries that reject or validate
- **Location**: kb/engineering/properties.md:82 (P8), :318 (P27 "Validated fail-closed"); kb/engineering/glossary.md:59; kb/engineering/schema.md:221; kb/decisions/006-correspondence-records.md:79
- **Issue**: (a) P8 says distinct-real collisions are "rejected fail-closed at every recomputation boundary", then lists `probe project` on *projected* inputs, which the glossary (glossary.md:79) explicitly says is **not** a recomputation: projected inputs are only normalized. The glossary's own wording, "every boundary that normalizes", is the accurate one. The enumerated list keeps the intended behaviour clear, so this is a terminology inconsistency, not a behavioural contradiction. (b) P27's enumeration of record-shape validation, "every recomputation boundary (`probe merge` per input, `probe enrich`)", leaves out `probe project`. `validate_and_canonicalize_records` runs inside `normalize_atoms_reporting_collisions` (merge.rs:391), which both project branches reach (project.rs:255 directly, :258 via `prepare_atoms`).
- **Evidence**: cited lines; `rg normalize_atoms|prepare_atoms src/` shows the only normalizing call sites are merge (merge.rs:476/480), enrich (propagate.rs:224/291) and project (project.rs:255/258). `probe summary` does not normalize, so "every boundary that normalizes" is exactly these three.
- **Recommendation**: Use "every boundary that normalizes (`probe merge` per input, `probe enrich`, `probe project`)" in P8 and P27. Align glossary.md:59, schema.md:221 and ADR-006:79 the same way (ADR: recommend only, as in W3).

### [W5] The rule that categorized dependency arrays are sets appears only in P8; P15, schema.md, probe-lean.md and P10 don't say it
- **Location**: kb/engineering/properties.md:84 (P8), :105-107 (P10), :132-143 (P15); kb/engineering/schema.md:168-177; kb/tools/probe-lean.md:40; src/commands/merge.rs:381-385
- **Issue**: P8 says the categorized arrays "are sets, like their union `dependencies` ([P15])", but P15 only defines `dependencies` as the union of the subsets. Neither P15 nor the extension-field list in schema.md says the subsets themselves are order-insignificant. The claim is attributed to a property that doesn't make it. Normalization now reorders and deduplicates these producer-owned extension values on **every** atom at every normalizing boundary, not only on colliding aliases. P10 ("Tool-specific extension fields … MUST be preserved through merge") has no normalization carve-out. That was already a latent gap for P8's renaming, and sorting widens it.
- **Evidence**: P15 text at :134-141; schema.md:170-177 ("functions called in `requires` clauses", "(`dependencies` = union of all three)"); merge.rs:381-385 (`sort_by_cached_key` + `dedup` inside the per-atom loop). Producer side: probe-verus emits `BTreeSet<String>` (probe-verus/src/lib.rs:327, 339), so sorting is a no-op there. probe-lean builds plain arrays (e.g. probe-lean/Tools/GenFixture.lean:74). Its order significance is not stated anywhere in the KB, and I did not verify whether its output is already sorted.
- **Recommendation**: Put the set semantics where they are defined: add a sentence to P15 ("each categorized subset is a set; order and duplicates carry no meaning") and to the probe-verus/probe-lean extension lists in schema.md. Add "modulo P8 normalization (renaming, and set canonicalization of the categorized arrays)" to P10. P8 can then cite P15 accurately. Confirm with the probe-lean owner that array order carries no meaning, per ADR-005 producer ownership.

## Info

### [I1] P8 "sorted" is underspecified for non-string entries
properties.md:84 says each array is "sorted and deduplicated". The code sorts strings first, then non-strings by their JSON text (merge.rs:381-384). This rule appears only in a code comment.

### [I2] probe-merge.md Phase 2 names only enrich as the matching unary boundary
probe-merge.md:47 says "the same rule the unary `probe enrich` boundary applies". This is accurate but leaves out `probe project`. Optionally add it or link to probe-project.md Step 2.

### [I3] probe-project.md Step 2 doesn't say that normalizing a view also puts its categorized arrays in canonical order
probe-project.md:38 says projected inputs are "still normalized". Re-projecting a view therefore also sorts and deduplicates its categorized arrays. This is harmless given W5's set semantics, but it isn't stated.

### [I4] Normalization terms used in P7/P8 have no glossary entries
"alias", "distinct real atom", "normalization collision" and the new "key order" are defined inline in P8 but have no glossary.md entries. The first three predate this changeset.

## Verified clean

- **P7 (amended)**: "last" = input-file order. `merge_generic_maps` iterates inputs in argument order, and incoming replaces base (merge.rs:661-690). The intra-input carve-out matches `normalize_generic`.
- **P8 specs/proofs paragraph**: the key-order tie-break is real. Specs/proofs `data` is deserialized into `BTreeMap<String, Value>` (types.rs:544), so file member order is lost, and among `f()`, `f().`, `f()..` the most-dotted alias sorts last and is kept. The warning names the input position, both original keys and the kept key (merge.rs:642-643). The example string in probe-merge.md:51 matches the format exactly, and tests/roundtrip.rs pins it. Collisions feed `stats.conflicts` (merge.rs:670, 675).
- **P8 rejection enumeration vs code**: merge rejects per input with position prefix (merge.rs:476, 480). enrich rejects via strict `prepare_atoms` (propagate.rs:224). project rejects on both branches (project.rs:255, 258). The lossy map is no longer reachable outside the module, since `normalize_atoms_reporting_collisions` is private.
- **P8 set paragraph vs code**: aliases differing only in order or duplicates collapse, and a real difference still rejects (test `test_categorized_arrays_compared_as_sets`).
- **Glossary `carrier preparation`** (glossary.md:79): consistent with the amended P8 and the code, including "every boundary that normalizes" and `prepare_atoms` still in propagate.rs.
- **probe-project.md Step 2** (:38): consistent. Projected inputs are normalized with the same collision rejection, and authoritative inputs are prepared before trimming.
- **probe-merge.md Phase 2 + statistics table**: consistent with P7/P8 and the code. The `normalize_atoms()` entry in the key-source-files table still exists.
- **P15**: sorting/dedup preserves the union equality, and project trimming (Step 5) filters without reordering.
- **P14**: canonical ordering of the arrays only makes output more deterministic.
- **P6**: cross-input first-wins is unchanged.
- **P27**: records still union through benign collisions (the record-shape enumeration drift is in W4).
- **architecture.md**:31, 33: still accurate.
- **Staleness stamps**: all touched and cross-checked KB files carry `last-updated: 2026-09-30`, matching their git history. No stale-stamp findings.
