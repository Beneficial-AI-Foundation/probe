---
auditor: ambiguity-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB, resolved via common §1.1)
scope: branch la/merge-soundness-pr3c-collision-warnings-projection (PR #83)
  at 903c1b8 (commits 5623ae9, b33d9d0, 903c1b8 over origin/main), plus two
  uncommitted working-tree edits — tests/roundtrip.rs
  `p15_decomposition_survives_merge_then_project` and the CHANGELOG 0.5.0
  correspondence-record entry reworded to "Every boundary that normalizes".
  Per §2b the branch is not the default branch; property text was compared
  against `origin/main` (Cargo 0.4.0; `git log HEAD..origin/main` is empty).
  On main, P7 has no input-order sentence, P8 has no `probe project` clause,
  no set-semantics paragraph and no specs/proofs collision paragraph, P10 has
  no normalization carve-out, P15 has no set sentence, and P27 names only
  merge and enrich as validating boundaries. All of these are amendments
  carried by PR #83 itself.
status: 0 critical, 2 warnings, 2 info
---

## Critical

None. No KB file states a rule that the amended properties or the code
contradict. The cross-repo wording conflict in W2 is classed as a warning;
the reasoning is in the finding.

## Warnings

### [W1] Merge-order constraint: the code conforms only to property amendments unmerged on main
- **Location**: kb/engineering/properties.md:74 (P7), :82, :84, :86 (P8), :107 (P10), :143 (P15), :320 (P27); src/commands/merge.rs:301-309, 367-385, 651-655; src/commands/project.rs:134-144
- **Issue**: Against `origin/main`'s properties.md, these behaviours of the branch have no spec: `probe project` rejecting distinct-real collisions and malformed records on any input, projected or not (main's P8 and P27 name only merge and enrich); sorting and deduplicating the categorized dependency arrays (main's P8, P10 and P15 say nothing about their order, and main's P10 has no normalization carve-out); and the specs/proofs intra-input collision warning with a key-order tie-break (main's P7 says only "last one wins"). The code conforms only to the branch text.
- **Evidence**: `git diff origin/main -- kb/engineering/properties.md` shows every cited passage as an addition or rewrite. `git log origin/main..HEAD` lists 5623ae9, b33d9d0 and 903c1b8, all on PR #83.
- **Recommendation**: The spec and the code change are in the same pull request ([PR #83](https://github.com/Beneficial-AI-Foundation/probe/pull/83)), so the constraint holds as long as they merge together. Do not split the KB commits from the code. The set semantics for the categorized arrays (P8 :84, P15 :143) are a new hub-level statement about producer-emitted fields and need explicit human sign-off as a spec refinement, per CLAUDE.md. W2 is relevant to that sign-off.

### [W2] probe-lean's schema says `dependencies` can repeat a name; hub P15 says it is a deduplicated union whose duplicates carry no meaning
- **Location**: kb/engineering/properties.md:139 (P15, "deduplicated union"), :143 (P15, new: "their order and duplicates carry no meaning"); ../probe-lean/docs/SCHEMA.md:153 (last changed 2026-09-18, 97908be)
- **Issue**: probe-lean documents that `dependencies` deduplicates "by declaration identity before private mangling is stripped, so two distinct private declarations that print to the same name can appear twice; this is permitted and silent". So a string duplicate in probe-lean's output can stand for two different declarations. The hub says the opposite about the same field: `dependencies` is a "deduplicated union", and P15's new sentence says duplicates in the categorized subsets carry no meaning. P8 normalization now drops them from `type-`/`term-dependencies` too. The "deduplicated union" wording predates this PR. The "duplicates carry no meaning" sentence is new in it.
- **Evidence**: The quoted lines. On the hub side, `Atom.dependencies` is a `BTreeSet<String>` (src/types.rs:131), so the hub already collapsed `dependencies` duplicates at load before this PR. merge.rs:385 now does the same for the categorized arrays. probe-lean's SCHEMA.md says nothing about array order, so the "sorted" half of P15 does not conflict.
- **Why not critical**: The hub code, P15 and P8 agree with each other. No hub implementer would build anything different after reading them. The disagreement is between the hub contract and one producer's description of its own output, and the colliding declarations already share one code-name, which is a P2 identity question that predates this PR. I am not certain this is only wording. If probe-lean consumers rely on the duplicate as a signal (SCHEMA.md says `tools/audit/compare-extract.py` reports it as a diagnostic), the hub's dedup silently removes that signal.
- **Recommendation**: Get the probe-lean owner's confirmation before PR #83's set sentence merges. Following ADR-005, either probe-lean's SCHEMA.md:153 points out that the hub treats all dependency arrays as sets (so the duplicate does not survive composition), or P15 names this probe-lean case as an accepted loss.

## Info

### [I1] P8's first paragraph and ADR-006 still call `probe project` a "unary recomputation boundary" without qualification
properties.md:80 says "Unary recomputation boundaries (`probe enrich`, `probe project`) apply the same normalization". ADR-006:160-163 says the same and adds "their single authoritative input". For already-projected inputs, `probe project` normalizes but does not recompute (glossary.md:79; probe-project.md:38), and the rest of the KB now uses "boundary that normalizes" for the enumeration (properties.md:82, :320; glossary.md:59, :79; schema.md:221). The P8 sentence is not wrong about which operations run, since normalization does run everywhere it says, but it uses one term for two different sets of boundaries. ADR-006:79 ("every recomputation boundary rejects a malformed record shape") and :170-172 ("rejected at every recomputation boundary: `probe merge` … exactly as `probe enrich`") also leave out `probe project`. ADR-006 is a decision record, so the user should decide whether to append a dated amendment note (e.g. "PR 3c: `probe project` normalizes and rejects on both authoritative and projected inputs; see P8/P27") or leave it as a historical snapshot and rely on P8 and P27. Do not rewrite the decision text in place.

### [I2] Normalization terms used in P7/P8 have no glossary entries
P7 and P8 (properties.md:74, :82-86) use "alias", "distinct real atom", "normalization collision", "key order" and "boundary that normalizes", but the glossary has no heading for any of them (`rg '^## ' kb/engineering/glossary.md`). Each is defined inline in P8, and the first three predate PR #83.

## Dropped from the previous report (resolved)

- **Old W2** (schema.md § Normalization and § Specs and proofs stale against P8): schema.md:312 now covers the categorized-array sort and dedup (citing P15) and `probe project` "(on any input)". schema.md:290 now documents the intra-input specs/proofs warning, the conflict count and the key-order tie-break.
- **Old W4** ("every recomputation boundary" wording): properties.md:82 (P8) and :320 (P27, which now lists `probe project` on any input), glossary.md:59 and schema.md:221 now say "every boundary that normalizes". The working-tree CHANGELOG.md:20 edit brings the 0.5.0 entry in line. The remaining ADR-006 part is folded into I1.
- **Old W5** (set semantics stated only in P8): P15 now states them itself (properties.md:143), P10 has the normalization carve-out (:107), and schema.md:312 points to P15. The unverified probe-lean question is now checked and reported as W2.
- **Old W3** (ADR-006 omits project's rejection): kept, but downgraded and merged into I1. The normative texts (P8, P27, glossary, probe-project.md) all state the rule, so the ADR gap no longer affects what gets implemented.
- **Old I1** (non-string sort order unspecified): P8 (properties.md:84) now says "string entries first in string order, then any non-string entries ordered by their JSON text", which matches merge.rs:381-384.
- **Old I2** (probe-merge.md named only enrich): probe-merge.md:47 now says "the unary `probe enrich` and `probe project` boundaries".
- **Old I3** (probe-project.md didn't say re-projection canonicalizes arrays): P8 (properties.md:84) now defines canonical ordering as part of normalization, so probe-project.md:38's "still normalized" covers it.

## Verified clean

- **P7 (amended, :74)**: "last" means input-file order, and the intra-input carve-out points to P8. probe-merge.md:51 and schema.md:290 say the same.
- **P8 (all four paragraphs)**: The rejection list (merge per input, enrich, project on any input) matches the code: `normalize_atoms` (merge.rs:301) is the strict entry point, `prepare_atoms` (propagate.rs:221) calls it, and project reaches it on both branches. The canonical-form rule matches merge.rs:367-385. The specs/proofs warning format in probe-merge.md:51 matches merge.rs:651-655 ("kept … (key order, P8)").
- **P10 (:107)**: the carve-out lists exactly the two changes normalization makes to extension values (renaming, and canonical order for the categorized arrays).
- **P15 (:141-147)**: Project trims every categorized array with the `dependencies` filter and leaves non-string entries alone (project.rs:130-144), as probe-project.md:55 and :89 say. The working-tree test `p15_decomposition_survives_merge_then_project` checks the union equality after merge and after the trim, on Verus-shaped and Lean-shaped atoms that include dotted aliases, duplicates and unsorted entries. It also checks that `body-dependencies` comes out sorted and deduplicated. This matches the property text.
- **P27 (:320)**: the validating-boundary list now includes project, and the glossary (:59) and schema.md (:221) use the same wording.
- **P4/P5**: P8's statement that a map with unsorted categorized arrays is off the carrier fits P4's definition of the carrier as normalized, enrichment-consistent maps.
- **P6, P14**: cross-input first-wins is unchanged, and canonical array ordering only makes output more deterministic.
- **Glossary `carrier preparation` (:79)** and **probe-project.md Step 2 (:38)**: consistent with P8 and the code.
- **CHANGELOG**: the [Unreleased] section is empty. The 0.5.0 entries (:17, :20, :24, :26-28) match P8, P15 and P27 as amended.
- **Staleness stamps**: properties.md, schema.md, glossary.md, probe-merge.md, probe-project.md and ADR-006 carry `last-updated: 2026-09-30`, which matches their git history. probe-lean.md (2026-09-28) and index.md (2026-09-28) also match their history. Neither was affected by this change, since probe-lean.md:40-41 already points to P15.
