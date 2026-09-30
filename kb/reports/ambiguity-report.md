---
auditor: ambiguity-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB, resolved via common §1.1)
scope: branch la/merge-soundness-pr3c-collision-warnings-projection (PR #83)
  at 8ee5952, plus uncommitted working-tree edits — properties.md P8 first
  paragraph ("the unary boundaries that normalize"), five new glossary
  entries (alias, normalization collision, distinct real atom, key order,
  boundary that normalizes — scoped to atom inputs) and an updated
  `carrier preparation`, the new public
  `prepare_projection_input` in src/commands/project.rs with P8/P23 `@kb:`
  annotations (P15 on `project_atoms`, P9/P1 moved to `cmd_project`),
  probe-project.md's algorithm renumbered to Steps 1–7 with seed matching
  in Step 3 and `project_atoms`'s step comments renumbered to match, and the 0.5.0
  CHANGELOG entry for `prepare_projection_input`.
  Per §2b the branch is not the default branch; property text was compared
  against `origin/main` (Cargo 0.4.0; `git log HEAD..origin/main` is empty).
  On main, P7 has no input-order sentence, P8 has no `probe project` clause,
  no set-semantics paragraph and no specs/proofs collision paragraph, P10 has
  no normalization carve-out, P15 has no set sentence, and P27 names only
  merge and enrich as validating boundaries. All of these are amendments
  carried by PR #83 itself.
status: 0 critical, 2 warnings, 1 info
---

## Critical

None. No KB file states a rule that the amended properties or the code
contradict. The cross-repo wording conflict in W2 is classed as a warning;
the reasoning is in the finding.

## Warnings

### [W1] Merge-order constraint: the code conforms only to property amendments unmerged on main
- **Location**: kb/engineering/properties.md:74 (P7), :80, :82, :84, :86 (P8), :107 (P10), :143 (P15), :320 (P27); src/commands/merge.rs:301-309, 367-385, 625-660; src/commands/project.rs:49-68, 172-198
- **Issue**: Against `origin/main`'s properties.md, these behaviours of the branch have no spec: `probe project` normalizing and rejecting distinct-real collisions and malformed records on any input, projected or not (main's P8 and P27 name only merge and enrich; main's P8 first paragraph calls project a "recomputation boundary" with no projected-input case); sorting and deduplicating the categorized dependency arrays (main's P8, P10 and P15 say nothing about their order, and main's P10 has no normalization carve-out); and the specs/proofs intra-input collision warning with a key-order tie-break (main's P7 says only "last one wins"). The code conforms only to the branch text.
- **Evidence**: `git diff origin/main -- kb/engineering/properties.md` shows every cited passage as an addition or rewrite. The :80 rewrite is still uncommitted. All other passages are in commits on PR #83.
- **Recommendation**: The spec and the code change are in the same pull request ([PR #83](https://github.com/Beneficial-AI-Foundation/probe/pull/83)), so the constraint holds as long as they merge together. Do not split the KB commits from the code. The set semantics for the categorized arrays (P8 :84, P15 :143) are a new hub-level statement about producer-emitted fields and need explicit human sign-off as a spec refinement, per CLAUDE.md. W2 is relevant to that sign-off.

### [W2] probe-lean's schema says `dependencies` can repeat a name; hub P15 says it is a deduplicated union whose duplicates carry no meaning (tracked: probe-lean#116)
- **Location**: kb/engineering/properties.md:139 (P15, "deduplicated union"), :143 (P15, "their order and duplicates carry no meaning"); ../probe-lean/docs/SCHEMA.md:153
- **Issue**: probe-lean documents that `dependencies` deduplicates "by declaration identity before private mangling is stripped, so two distinct private declarations that print to the same name can appear twice; this is permitted and silent". So a string duplicate in probe-lean's output can stand for two different declarations. The hub says the opposite about the same field, and P8 normalization now drops such duplicates from `type-`/`term-dependencies` too. The "deduplicated union" wording predates PR #83. The "duplicates carry no meaning" sentence is new in it.
- **Evidence**: The quoted lines. On the hub side, `Atom.dependencies` is a `BTreeSet<String>` (src/types.rs:131), and merge.rs:385 dedups the categorized arrays. Neither side has changed.
- **Why not critical**: The hub code, P15 and P8 agree with each other. The disagreement is between the hub contract and one producer's description of its own output, and the colliding declarations already share one code-name, which is the P2 identity problem tracked in probe-lean#88. If probe-lean consumers rely on the duplicate as a signal (SCHEMA.md says `tools/audit/compare-extract.py` reports it as a diagnostic), the hub's dedup silently removes it.
- **Status**: Tracked in [probe-lean#116](https://github.com/Beneficial-AI-Foundation/probe-lean/issues/116), which links [probe-lean#88](https://github.com/Beneficial-AI-Foundation/probe-lean/issues/88). It is waiting on the probe-lean owner. It stays a warning until either probe-lean's SCHEMA.md:153 or P15 is reconciled. Following ADR-005, either probe-lean states that the hub treats all dependency arrays as sets, or P15 names this case as an accepted loss.

## Info

### [I1] ADR-006 keeps "recomputation boundary" wording that leaves out `probe project` (a deliberate historical snapshot)
ADR-006:79 ("every recomputation boundary rejects a malformed record shape"), :160-163 ("Unary recomputation boundaries (`probe enrich`, `probe project`) apply `prepare` … to their single authoritative input") and :170-172 ("rejected at every recomputation boundary: `probe merge` … exactly as `probe enrich`") are narrower than P8 and P27. The user decided to leave the decision record untouched. P8, P27 and the glossary are normative and state the full rule, so this is a note for readers of the ADR, not a gap in the spec.

## Resolved in this pass

- **Previous I2** (`project_atoms` step comments used the old numbering): the comments now read "Step 3: Build seed set" (project.rs:94), "Step 4: Build reverse adjacency index" (:118), "Step 5: BFS forward" (:131), "Step 5: BFS backward" (:152), "Step 6: Filter atoms and trim dependencies" (:172). These match probe-project.md Steps 3 (:37), 4 (:42), 5 (:46) and 6 (:53). The P9/P1 `@kb:` annotations moved from `project_atoms` to `cmd_project` (project.rs:257-258), which is where provenance and the envelope are written. `project_atoms` keeps P14/P15 (:83-84).

## Resolved in the previous pass

- **Previous I2** (`boundary that normalizes` overstated its checks for specs/proofs and used the undefined "recomputation boundaries"): glossary.md:99 now says "applies P8 normalization to its atom input and therefore enforces its fail-closed checks on it", lists "`probe merge` per atoms input", and adds "Specs/proofs inputs to `probe merge` are normalized too, but a collision there is warned about and resolved by key order, not rejected". That matches `normalize_generic` (merge.rs:625-660) and P8 :86. The comparison now reads "wider than the boundaries that recompute enrichment", which needs no separate definition. The one remaining use of "unary recomputation boundaries" (glossary.md:79, `carrier preparation`) names its members inline (`probe enrich`, and `probe project` on authoritative inputs), so it is self-defining.
- **Previous I3** (probe-project.md step order vs the code): Step 2 (:33-35) now prepares the carrier before Step 3 (:37-40) loads the mappings and builds the seed set "from the prepared (normalized) atom data". That is the order `cmd_project` uses: it calls `prepare_projection_input` first, then passes the prepared map to `project_atoms`. The Key source files row (:24) now credits `project_atoms` with "seed matching and Steps 4–6", which matches its body (project.rs:94-198). The Step 2 anchor is unchanged, so the `@kb:` link at project.rs:31 still resolves, and nothing else in `kb/`, `docs/` or README links to probe-project step numbers.
- **Annotations**: `prepare_projection_input` now carries `@kb:` P8 and P23 (project.rs:33-34), and `project_atoms` carries P15 (:84), the property its categorized-array trim implements (:186-197).

Resolved in earlier passes (unchanged, still verified): P8's first-paragraph "recomputation boundary" wording (properties.md:80), the five missing glossary terms (glossary.md:81-99), schema.md normalization staleness (schema.md:290, :312), the "every recomputation boundary" wording in P8, P27, the glossary and schema.md (properties.md:82, :320; glossary.md:59; schema.md:221), the set semantics being stated only in P8 (properties.md:107, :143), the non-string sort order (properties.md:84), probe-merge.md naming only enrich (probe-merge.md:47), and probe-project.md not mentioning re-canonicalization (properties.md:84).

## Verified clean

- **P7 (:74)**: "last" means input-file order, and the intra-input carve-out points to P8. probe-merge.md:51, schema.md:290 and glossary `key order` (:95) agree.
- **P8 (all four paragraphs)**: The rejection list (merge per atoms input, enrich, project on any input) matches the code and the glossary entry `boundary that normalizes`. `normalize_atoms` (merge.rs:301) is the strict entry point, `prepare_atoms` (propagate.rs:221) calls it, and `prepare_projection_input` (project.rs:49) reaches it on both branches. The canonical-form rule matches merge.rs:367-385. The specs/proofs warning format in probe-merge.md:51 matches merge.rs:653-655.
- **Glossary normalization entries (:81-99)**: `distinct real atom` matches merge.rs:402-405, and `key order` matches merge.rs:634-657. The `carrier preparation` links resolve.
- **P10 (:107)**, **P15 (:141-147)**: consistent. Project trims the categorized arrays with the `dependencies` filter (project.rs:186-197), as probe-project.md Step 6 (:56) says, and `p15_decomposition_survives_merge_then_project` (tests/roundtrip.rs) checks it.
- **P23 / carrier preparation**: `prepare_projection_input` validates `status-origin` first and then enriches authoritative inputs only. That matches probe-project.md Step 2 (:35) and glossary.md:79. `project_atoms`'s doc (project.rs:73-75) states that its input must come from `prepare_projection_input`.
- **P27 (:320)**: the validating-boundary list includes project, and record validation runs inside `normalize_atoms_reporting_collisions` (merge.rs:391), which both project branches reach.
- **P4/P5, P6, P14**: unaffected.
- **CHANGELOG**: the 0.5.0 "Changed" entry (CHANGELOG.md:25) describes `prepare_projection_input` accurately.
- **KB links**: `./scripts/check-kb-links.sh` was reported passing on this tree and not re-run for this report. The only `@kb:` target near the renumbering, the Step 2 heading, is unchanged.
- **Staleness stamps**: properties.md, glossary.md and probe-project.md carry `last-updated: 2026-09-30`, which matches their working-tree edits. The other stamps match git history.
