---
auditor: code-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB). Property text audited at origin/main (4b633e0) per common skill §2b. PR #83 amends P7 (one "last = input order" sentence), P8 (first paragraph: "the unary boundaries that normalize — `probe enrich`, and `probe project` on any input"; project rejects any input; categorized arrays are sets in canonical form; specs/proofs intra-input tie-break), P10 ("up to P8 normalization"), P15 ("the categorized subsets are sets") and P27 ("every boundary that normalizes"). All amendments ship in PR #83 alongside the code that depends on them, so every dependency on them is a merge-order constraint internal to that PR (W1, I1, I2).
scope: PR #83 (https://github.com/Beneficial-AI-Foundation/probe/pull/83), branch la/merge-soundness-pr3c-collision-warnings-projection, commits 5623ae9..8ee5952 over origin/main 4b633e0, plus the uncommitted audit follow-ups. Those are src/commands/project.rs (public `PreparedProjectionInput` / `prepare_projection_input`, `cmd_project` routed through it, `project_atoms` precondition documented, P8/P23 annotations on `prepare_projection_input`, a P15 annotation on `project_atoms`, P9/P1 annotations moved to `cmd_project`, in-function step comments renumbered to the doc's Steps 3–6, four unit tests), kb/tools/probe-project.md (algorithm reordered to seven steps matching the code, Key source files row), kb/engineering/glossary.md (`carrier preparation` names the new function; new entries alias, normalization collision, distinct real atom, key order, boundary that normalizes), kb/engineering/properties.md (P8 first paragraph) and CHANGELOG.md (0.5.0 Changed entry). Covers src/commands/{merge,project,propagate,summary}.rs, src/authority.rs, src/types.rs, schemas/atom-envelope.schema.json, Cargo.toml, CHANGELOG.md, kb/engineering/{properties,glossary,schema}.md, kb/tools/{probe-merge,probe-project}.md, merge-soundness-fix-plan.md, and the docs/code they touch (ADR-006, architecture.md, README, docs/, sibling-repo callers of the changed library API).
status: 0 critical, 1 warning, 3 info
---

## Critical

None.

## Warnings

### [W1] Categorized-array canonicalization conforms to P5/P10 only under the amended P8 (merge-order constraint, PR #83)
- **Location**: src/commands/merge.rs:367-387 (normalize + `sort_by_cached_key` + `dedup` inside `normalize_atoms_reporting_collisions`); kb/engineering/properties.md:84 (P8 "categorized dependency arrays … are sets" paragraph), :107 (P10 "up to P8 normalization"), :143 (P15 "the categorized subsets are sets") — all PR #83 text.
- **Issue**: Every boundary that normalizes (merge per input, enrich, project on both branches) now reorders and deduplicates `requires-`/`ensures-`/`body-`/`type-`/`term-dependencies`. origin/main's P5 says `μ(A, ∅) = A` *exactly* on the carrier, where the carrier is "normalized, enrichment-consistent atom maps", and origin/main's P8 defines normalization as trailing-dot stripping only. Under that text an atom map whose categorized arrays are unsorted or duplicated is on the carrier, yet merging it with nothing changes it — a literal P5 violation. origin/main's P10 ("MUST be preserved through merge") likewise does not license reordering an extension value. Only the amended P8/P10/P15 (canonicalization is part of normalization, so such a map is off-carrier and `μ(A, ∅) = enrich(normalize(A))` applies) make the code conformant.
- **Evidence**: `arr.sort_by_cached_key(|entry| match entry.as_str() { Some(name) => (false, name.to_string()), None => (true, entry.to_string()) }); arr.dedup();` (merge.rs:381-385). `git show origin/main:kb/engineering/properties.md` P8: "Normalization strips trailing `.` characters"; no set semantics for the categorized arrays anywhere in that file. Practical impact is nil for current producers: probe-verus emits these fields from `BTreeSet<String>` and probe-lean from its sort-dedup helpers, so their output is already canonical. `p15_decomposition_survives_merge_then_project` (tests/roundtrip.rs, committed in 8ee5952) pins the behavior: `body-dependencies` is written as `["…g()", "…h()"]` from an unsorted, duplicated, dotted input.
- **Recommendation**: Land the code and the property amendments together (they are already in the same PR; do not split merge.rs from properties.md in a squash or cherry-pick). No code change needed.

## Resolved during this audit cycle

- **[W2] `project_atoms` library path skipped carrier preparation** (found at 903c1b8+WT, resolved in the working tree after 8ee5952). Projection-input preparation is now the public `prepare_projection_input(atoms, projected)` (src/commands/project.rs:49-69). It runs `validate_status_origins` (:53), then `normalize_atoms` on a view (:55) or `prepare_atoms` on an authoritative input (:62). A library caller can therefore reach the normalize-only branch, which used to be `pub(crate)` only. `cmd_project` has no inline preparation left and calls it (:296), so the CLI and library share one path. `project_atoms`'s doc comment states the precondition (:73-75). Four tests cover it (project.rs:967-1034): `test_prepare_authoritative_normalizes_and_recomputes`, `test_prepare_projected_normalizes_and_inherits_labels`, `test_prepare_rejects_invalid_input_on_both_branches` (collision and bogus `status-origin`, both branches) and `test_prepare_then_project_matches_normalized_seed`. The precondition is documented rather than type-enforced; see I3.
- **[I4] `@kb:` annotation targeting and probe-project.md step mapping** — resolved in two rounds.
  - Final round: the P9/P1 annotations moved off `project_atoms` onto `cmd_project` (project.rs:257-258, above `pub fn cmd_project` at :260), which builds the envelope and passes provenance through (:352-371). `project_atoms` now carries only P14 and P15 (:83-84).
  - Final round: the in-function step comments now match probe-project.md — `// Step 3: Build seed set` (:94), `// Step 4: Build reverse adjacency index` (:118), `// Step 5: BFS forward` / `backward` (:131, :152), `// Step 6: Filter atoms and trim dependencies` (:172).
  - First round: `prepare_projection_input` gained P8 and P23 anchors (project.rs:33-34), alongside the Step 2 and glossary ones (:31-32).
  - First round: `project_atoms` gained a P15 anchor (:84) for the categorized-array trim it does (:186-197).
  - First round: kb/tools/probe-project.md now follows the code's order. Step 1 is load and version gate. Step 2 is prepare (`prepare_projection_input`, project.rs:296). Step 3 is load mappings and build the seed set (`load_mappings` after preparation in `cmd_project`, seed matching at the top of `project_atoms`). Steps 4–7 are reverse index, BFS, filter/trim, write.
  - The Key source files row says `project_atoms()` covers "seed matching and Steps 4–6", which matches its body (seeds, reverse index, BFS, trim; envelope writing stays in `cmd_project`).
  - The glossary `boundary that normalizes` entry now limits rejection to atom inputs. It says specs/proofs inputs are normalized but warned about and resolved by key order, which matches `normalize_generic` (merge.rs:615-660).

## Info

### [I1] `probe project` rejecting collisions and malformed records on already-projected inputs rests on the P8/P27 amendments (merge-order constraint, PR #83)
- **Location**: src/commands/project.rs:54-60 (projected branch of `prepare_projection_input` calls strict `normalize_atoms`, which also runs `validate_and_canonicalize_records`); properties.md P8 first paragraph and rejection sentence, P27 "Validated fail-closed" bullet (PR #83 text).
- **Issue**: origin/main's P8 already classifies `probe project` as a unary recomputation boundary that normalizes before seed matching, so rejection on *authoritative* inputs is required by main text. Only the view case is new. origin/main names rejection at "every recomputation boundary: merge…, enrich…", and P27 validates records at "every recomputation boundary (`probe merge` per input, `probe enrich`)". ADR-006 Decision 6 treats project-on-a-view as a read path ("only recomputation paths refuse them"). The code is strictly more fail-closed than main requires. A view written by `probe project` is already normalized and validated, so only hand-crafted or legacy views can trip it. ADR-006 (kb/decisions/006-correspondence-records.md:79, :167-172) still phrases both rules as "every recomputation boundary" naming merge and enrich; that is a narrower statement, not a contradiction.
- **Evidence**: `if projected { let (atoms, keys_normalized) = crate::commands::merge::normalize_atoms(atoms)?; … }` (project.rs:54-55). The working-tree P8 first paragraph and the glossary entry `boundary that normalizes` state the wider set explicitly; both are PR #83 text, not origin/main.
- **Recommendation**: Keep the P8/P27 amendments in the same PR as the project.rs change. No code change needed.

### [I2] Specs/proofs intra-input tie-break (key order) is specified only by the P7/P8 amendment (merge-order constraint, PR #83)
- **Location**: src/commands/merge.rs:615-660 (`normalize_generic`); properties.md:74 (P7 "'Last' is the order of input files") and :86 (P8 specs/proofs paragraph).
- **Issue**: Resolution behavior (the alias sorting last wins) is unchanged from origin/main, which is silent on intra-input specs/proofs collisions. What is new is the warning, the `stats.conflicts` count, and the KB text that describes the tie-break. The code matches the amended text exactly: one warning per normalized key after the loop, naming every original key and the key actually kept (the last-sorted one). `test_generic_intra_input_collision_counted` covers two and three aliases. Land together.

### [I3] `project_atoms`'s preparation precondition is documented, not type-enforced (accepted as-is)
- **Location**: src/commands/project.rs:73-75, :85-91.
- **Issue**: `project_atoms` still takes `&BTreeMap<String, Atom>`, so a caller can pass an unprepared map and get the dotted-key seed miss and frozen stale labels W2 described. Only the doc comment prevents it. The maintainers deliberately left this as-is to mirror the bare-map merge API (ADR-006: "the bare-map library API is documented as unable to check authority"). There are no callers outside the hub, and the only in-hub production caller (`cmd_project`) goes through `prepare_projection_input`. Recorded so the convention stays visible; not a blocker.
- **Recommendation** (optional, if an external caller appears): have `project_atoms` take `&PreparedProjectionInput` so the compiler checks the precondition.

## Verified clean

Checked against origin/main property text unless noted.

- **P1** (envelope completeness): merged and projected envelopes still carry schema, schema-version, tool, provenance, timestamp, data; the projected envelope keeps its `projection` block (project.rs:352-371). Merged/projected `schema-version` is `"3.1"`.
- **P2** (code-name uniqueness): output maps are `BTreeMap`s keyed by normalized name; strict `normalize_atoms` never returns a map where a distinct atom was silently dropped (merge.rs:301-309). The lossy core `normalize_atoms_reporting_collisions` is private.
- **P3** (structural stub): collision classification and cross-input resolution use `Atom::is_stub()` only (merge.rs:393-406, :486-501); `is_stub` unchanged (types.rs:151).
- **P4** (per-input normalization before conflict resolution; laws over deduplicated provenance): merge.rs:476 and :480 normalize each input, error-prefixed with its position, before the equal-key loop; provenance is deduplicated in both file-level merge paths (`load_atom_inputs`, merge.rs:577; `cmd_merge`, merge.rs:760).
- **P5**: holds on the carrier under the amended P8 (see W1).
- **P6** (first-wins with stub replacement): merge.rs:486-507 unchanged apart from dropped rejection boilerplate.
- **P7** (cross-input last-wins): `merge_generic_maps` cross-input override unchanged (merge.rs:685-692); intra-input collisions now counted (see I2).
- **P8** (normalization and fail-closed rejection): all boundaries route through the single strict helper.
  - Call sites: merge (merge.rs:476/480); enrich (`prepare_atoms`, propagate.rs:223-226); project authoritative (project.rs:62) and projected (project.rs:55), both inside `prepare_projection_input`, which `cmd_project` calls (:296).
  - The categorized arrays are now normalized (merge.rs:367-380). origin/main's P8 list already named them (properties.md:91), but origin/main's code did not normalize them.
  - Keys, `dependencies`, `dependencies-with-locations` code-names, record targets and mapping endpoints are still normalized.
  - The duplicated CLI rejection blocks are gone; error text is the shared `collision_error` (merge.rs:419-431).
- **P9** (deduplicated provenance inventory — already origin/main text): `dedup_provenance` (types.rs:73-81) keeps first occurrence and compares whole entries including flattened `source` extensions; the executable schema's merged `inputs` `minItems` is 1 in both branches (schemas/atom-envelope.schema.json:77, :123).
- **P10**: only the five categorized arrays are rewritten, and only canonicalized (see W1 for the origin/main caveat); project clones atoms and trims only dependency-bearing arrays.
- **P14** (determinism): categorized arrays now have a fixed order (strings byte-sorted, non-strings after by JSON text); project's `retain` preserves that order.
- **P15** (dependency completeness and preservation — preservation clause is origin/main text): normalization applies the same rule to `dependencies` and the subsets, and sort/dedup does not change membership. `project_atoms` trims the subsets with the same `included` filter as `dependencies`, passing non-strings through (project.rs:186-197). It is now annotated P15 (:84). Binary-level coverage: `p15_decomposition_survives_merge_then_project` (tests/roundtrip.rs) asserts the equality after merge, and again after a projection that trims an excluded callee, on both the Verus and Lean shapes.
- **P17** (category consistency): `load_validated_atom_file` still rejects non-atoms categories (authority.rs:203-209); `cmd_merge` category check unchanged.
- **P23** (enrichment): `prepare_projection_input` recomputes on the full authoritative graph before BFS/trim (project.rs:61-68). It never recomputes on views (`enrichment: None`, :54-60), which `test_prepare_projected_normalizes_and_inherits_labels` pins. `prepare_atoms` normalizes before enriching and errors before enrichment on a collision.
- **P27** (records unioned, inert, validated): records validated/canonicalized before the collision comparison (merge.rs:391) on every normalizing boundary including both project branches (amended text; see I1); benign collapses union records; projection BFS reads only `dependencies`.
- **ADR-006 Decision 7** (version gate): Cargo.toml is 0.5.0, the `probe` gate interval is `0.5.0 ≤ v < 1.0.0` (authority.rs:131-150), so the hub's own merge output (stamped `CARGO_PKG_VERSION`) passes its own gate.
- **P13, P16, P19, P21, P22, P24, P25**: not touched by this changeset.
- **Known issues**: properties.md carries no `C<n>` entries.
- **Docs**: the following all match the code.
  - CHANGELOG 0.5.0 lists `PrepareStats::dropped_atoms` under `### Removed` (CHANGELOG.md:61-62). Its Changed entry for `prepare_projection_input` (CHANGELOG.md:25) gives the right order: `status-origin` validation, then normalization with collision and record rejection, then enrichment on authoritative inputs only (project.rs:53-68).
  - kb/engineering/schema.md:312 documents categorized-array sort/dedup and project-on-any-input rejection.
  - kb/tools/probe-merge.md:47-51 and its stats table.
  - kb/tools/probe-project.md Steps 1–7 and its Key source files row.
  - Glossary: `carrier preparation`, `correspondence record`, and the five new entries (`alias`, `normalization collision`, `distinct real atom`, `key order`, `boundary that normalizes`).
  - The reworded P8 first paragraph.
  - Code facts behind the glossary entries: `distinct real atom` means both atoms are non-stub and unequal modulo records after categorized-array canonicalization (merge.rs:367-406). `key order` means the last-sorted alias is kept (merge.rs:634-657). `boundary that normalizes` names exactly the three callers of strict `normalize_atoms`.
  - The remaining "unary recomputation boundaries" phrasings (propagate.rs:205 on `prepare_atoms`, ADR-006:160, glossary `carrier preparation`) describe enrich-including preparation and stay accurate.
  - `load_validated_atom_file`'s return-type change (`LoadResult` → `ValidatedAtomFile`) is not a released-API break: the function was added after 0.4.0.
  - No remaining live references to `dropped_atoms`, "refusing to enrich/project", or `minItems: 2` outside archived plans and `merge-soundness-fix-plan.md`'s historical text. No stale `probe-project.md` step references elsewhere in src/, kb/ or docs/; the only anchor into it (project.rs:31, Step 2) still resolves.
  - Sibling-repo callers of changed functions: probe-aeneas calls `merge_atom_files` only. Its signature is unchanged; provenance is now deduplicated, which its docs' "provenance flattening" still describes.
- **`@kb:` annotations**: `./scripts/check-kb-links.sh` → "All KB links OK."; `scripts/check-enum-drift.py` → "No enum drift in the live docs." Every `@kb:` annotation in project.rs now sits on the code that implements it (P8/P23 on `prepare_projection_input`, P14/P15 on `project_atoms`, P9/P1 on `cmd_project`).
- **Build**: not run in this audit (cargo deliberately not invoked; findings are from reading code). The requester reports `cargo fmt`, `cargo clippy --all-targets -D warnings` and `cargo test --workspace` (273 passed, 0 failed, 8 ignored) clean on this working tree; not independently re-run here.
