---
auditor: code-quality-auditor
date: 2026-09-30
repo: probe (hub)
kb: kb/ (own KB). The branch amends P8 only (adds `probe project` to the distinct-real rejection list; adds the specs/proofs collision-warning paragraph). Audited against both the branch text and origin/main's; all other property text is identical to origin/main. See I1.
scope: branch la/merge-soundness-pr3c-collision-warnings-projection vs origin/main (commit 5623ae9, merge-soundness PR 3c, issue #82: specs/proofs intra-input collision warnings, provenance dedup, schema `inputs` minItems 2→1, P8 normalization and P15 projection trim of the categorized dependency arrays, carrier preparation in `probe project` with fail-closed collision rejection, `ValidatedAtomFile.projected`, 0.5.0 contract release, tests/roundtrip.rs), plus the uncommitted working tree (glossary `carrier preparation` entry, probe-merge.md stats cell, probe-project.md loader name, two new roundtrip tests). Re-audited 2026-09-30 11:22 after the fix delta for W1–W3 and I3 (src/commands/project.rs, two more roundtrip tests, schema description, probe-project.md, glossary, schema.md, CHANGELOG)
status: 0 critical, 0 warnings, 2 info (W1, W2, W3, I3 resolved; I2 open by decision; I4 new and fixed during the re-audit)
---

## Critical

None.

## Warnings

None open. The three below are kept for the record, each marked with its resolution.

### [W1] ✅ RESOLVED — `probe project` recomputes enrichment without the fail-closed `status-origin` check
- **Location**: src/commands/project.rs:243–247 (the authoritative branch that calls `prepare_atoms`), compare src/commands/propagate.rs:294 (`cmd_enrich` calls `validate_status_origins` before `prepare_atoms`)
- **Issue**: this PR makes `probe project` a recomputation boundary: it runs `prepare = enrich ∘ normalize` on authoritative inputs. It does not apply the out-of-enum `status-origin` rejection that the other enrichment boundaries apply. schema.md:159 lists only merge/enrich/summary as rejecting at load, so the doc doesn't contradict the code. But project now does the same recomputation as enrich and writes the marker into a view that the executable schema forbids. Enrichment is presence-based, so the malformed marker still seeds and no label is laundered. The actual harm is that project emits schema-invalid output, which `probe summary` then rejects.
- **Evidence**: reproduced with the built binary. A `probe-verus/atoms` input with `"status-origin": "bogus"` gave: `probe project … → exit 0`, and the output contains `"status-origin": "bogus"`; `probe summary out.json → exit 1` ("carries invalid status-origin \"bogus\""); `probe enrich` on the same input gave exit 1.
- **Recommendation**: call `validate_status_origins(&loaded.atoms)` before `prepare_atoms` in `cmd_project` (on the authoritative branch at least), add project to the schema.md:159 enforcement list, and add a CLI test next to `project_rejects_distinct_real_normalization_collision`.
- **Resolution**: `cmd_project` now calls `validate_status_origins` on every input, projected or not, before any preparation (src/commands/project.rs:235–241). schema.md:159 now lists `probe project`, and probe-project.md Step 2 states the check. New test: `project_rejects_invalid_status_origin` (exits non-zero, no output). I re-ran the reproduction against the rebuilt binary: the authoritative input exits 1 with "carries invalid status-origin" and writes no output. A projected input with a bogus marker also exits 1. That path has no dedicated test, but it runs the same unconditional call.

### [W2] ✅ RESOLVED — Already-projected inputs skip P8 normalization before seed matching
- **Location**: src/commands/project.rs:243–245 (the `loaded.projected` branch passes `loaded.atoms` through unnormalized), and src/commands/project.rs:45/50 (`project_atoms` exact-key seed lookup); kb/engineering/properties.md:78 (P8 ¶1)
- **Issue**: P8 ¶1 says unary recomputation boundaries (`probe enrich`, `probe project`) apply normalization to their input "before enrichment, seed matching, or trimming", with no carve-out for projected inputs. Mapping endpoints are always normalized at load, but on a projected input the atom keys are not. A dotted key therefore selects zero seeds, which is the dotted-key seed regression this PR fixes on the authoritative path. The KB contradicts itself here: probe-project.md Step 2 and the glossary `carrier preparation` entry skip the *whole* prepare step for projections, normalization included. Only inherited labels actually need protecting, and normalization alone doesn't change labels. This is a warning, not critical, because of reachability. Hub-produced 0.5.0 projections are already normalized, and pre-0.5.0 `probe` outputs are rejected by the version gate. The only way to hit it is a projection stamped with an ungated `tool.name`, or a hand-edited one.
- **Evidence**: reproduced. I took a real 0.5.0 `probe/projected-atoms` output, renamed its key to `probe:app/1.0/f().`, and re-projected it with the mapping `probe:app/1.0/f() → …`. The run printed `Input is a projection: labels inherited` and `Warning: 0/2 mapping keys found`, reported `Seeds: 0`, `Atoms out: 0`, and exited 0.
- **Recommendation**: either normalize projected inputs with `normalize_atoms` (keys and arrays, no enrichment; reject distinct-real collisions as on the authoritative path), or amend P8 ¶1 to scope project's normalization to authoritative inputs. The first option matches the current P8 text.
- **Resolution**: projected inputs now go through `normalize_atoms` (keys, arrays and record targets; no enrichment), and both branches share the collision rejection (src/commands/project.rs:251–291). probe-project.md Step 1/2 and the glossary say that views are only normalized. New test: `project_normalizes_keys_of_projected_input`. I re-ran the reproduction: `Keys normalized: 1`, `Seeds: 1`, output keeps f's inherited `verified` label, and `labels inherited` is still printed. A projected input with a distinct-real collision exits 1 ("refusing to project"), confirmed manually; there is no dedicated test for that path. Side effect: `normalize_atoms` also validates correspondence-record shapes, so a view with malformed `maps-to`/`mapped-from` is now rejected. That is fail-closed and consistent with the other boundaries.

### [W3] ✅ RESOLVED — Stale `inputs` description in the executable schema's merged branch
- **Location**: schemas/atom-envelope.schema.json:67
- **Issue**: the merged/projected-atoms branch still says `"one entry per input source (≥2 for merge, ≥1 for project)"`. After P9 dedup, `probe merge` can legitimately write a one-entry inventory: merging the same file twice, or two inputs from the same source, collapses to one entry. The specs/proofs branch description (line ~113) was updated in this PR, but this one was missed. Its `minItems` is already 1, so only the prose is wrong.
- **Evidence**: `test_merge_atom_files_dedups_provenance` (src/commands/merge.rs) asserts `provenance.len() == 1` for `merge_atom_files(&[a, a])`.
- **Recommendation**: reword it the same way as the specs/proofs branch ("a deduplicated source inventory (P9) — one entry per distinct source"). I didn't edit it because it is outside kb/.
- **Resolution**: now reads "Provenance: deduplicated inventory of input sources (≥1; merging inputs from the same source yields a single entry)." This matches `minItems: 1` and the dedup behaviour.

## Info

### [I1] (open, no action needed) P8 amendment: code also conforms to origin/main's text (soft merge-order note)
- **Location**: kb/engineering/properties.md:80 and :82 (branch) vs `git show origin/main:kb/engineering/properties.md` P8
- **Issue**: the branch adds `probe project` to the list of distinct-real rejection sites, plus a paragraph saying specs/proofs intra-input collisions are warned about and counted. origin/main's P8 already names `probe project` as a unary recomputation boundary (¶1), and requires rejection "at every recomputation boundary". Main's P23 (line 233) already says project recomputes before trimming. The project rejection is therefore required by main's text too. The specs/proofs warning is additive and doesn't contradict main's P7. This is a clarification, not a code-to-amendment dependency, so it is not a warning.
- **Recommendation**: none. The amendment lands with the implementation in this PR.

### [I2] (open by decision) "Keeping last" within one specs/proofs input means BTreeMap key order
- **Location**: src/commands/merge.rs:619 (`normalize_generic` warning), kb/engineering/properties.md:82, kb/tools/probe-merge.md:49
- **Issue**: P7's "last one wins" is about input position. Within a single input there is no input order, so "last" is the lexicographic order of the pre-normalization keys: the most-dotted alias (`f().`) beats the normalized key (`f()`). The result is deterministic (P14 holds) and the unit test comment documents it, but the KB says only "keeps the category's last-wins rule" and never says which of two colliding keys wins.
- **Recommendation**: optionally add "(the later key in sorted order, i.e. the dotted alias)" to P8's specs/proofs paragraph or to probe-merge.md.

### [I3] ✅ RESOLVED — CHANGELOG has no empty `[Unreleased]` above `[0.5.0]`
- **Location**: CHANGELOG.md:8
- **Issue**: the release workflow (`.cursor/rules/release-commit.mdc`, Path B step 2) says to move `[Unreleased]` under the version header "leaving an empty `[Unreleased]` above it". The branch replaced the header and didn't add a new one.
- **Recommendation**: add `## [Unreleased]` above `## [0.5.0] - 2026-09-30`.
- **Resolution**: an empty `## [Unreleased]` now sits above `[0.5.0]` (CHANGELOG.md:8). The 0.5.0 `probe project` entry was also updated to describe the status-origin rejection and view normalization.

### [I4] ✅ FIXED DURING RE-AUDIT — probe-project.md Step 2 heading said "authoritative inputs only"
- **Location**: kb/tools/probe-project.md:36
- **Issue**: after the W2 fix, Step 2 also applies to projected inputs (normalization plus the status-origin check), but the heading still read "Prepare the carrier (authoritative inputs only)".
- **Resolution**: heading changed to "Prepare the carrier (enrichment on authoritative inputs only)". No link targets the old anchor (checked with `rg step-2`), and `./scripts/check-kb-links.sh` passes.

## Fixed during this audit

- kb/tools/probe-project.md Step 2 heading (re-audit; see I4).

- kb/engineering/glossary.md `carrier preparation` (new, uncommitted entry): it said prepare is "applied per input inside merge (μ)". That contradicts P4's definition of μ (properties.md:38), where normalization runs per input and enrichment runs once after all inputs are combined. Reworded to "Merge (μ) applies the two halves separately — normalization per input, enrichment once after all inputs are combined (P4); the unary recomputation boundaries apply it whole". `./scripts/check-kb-links.sh` passed before the edit. The edit adds one link to an existing anchor (`#p4-merge-associativity-on-the-carrier`).

## Verified clean

Checked on the changeset (commit 5623ae9 plus the working tree):

- **P1** envelope completeness: `ValidatedAtomFile` still carries provenance; project and summary destructure it unchanged. Round-trip tests validate real outputs (tests/roundtrip.rs).
- **P3** stub detection: untouched. Collision classification still uses `Atom::is_stub`.
- **P4 / P5**: the μ factoring is unchanged. Provenance dedup provides the "deduplicated source inventory" P4's laws are stated over. `envelope_idempotence` in tests/roundtrip.rs re-merges merged output with one input and checks that `data` and `inputs` are unchanged. The law suites (tests/merge_laws.rs, merge.rs law tests) pass.
- **P6** first-wins with stub replacement: the conflict loop is untouched by this PR.
- **P7** specs/proofs last-wins: `merge_generic_maps` keeps last-wins across inputs and within an input. Collisions now increment `stats.conflicts`, printed at merge.rs:874 (`test_generic_intra_input_collision_counted`).
- **P8**: the categorized arrays (`CATEGORIZED_DEPENDENCY_ARRAYS`, src/types.rs) are normalized in `normalize_atoms`. Non-string entries pass through (`test_categorized_dependency_arrays_normalized`). The distinct-real collision is rejected at merge, enrich and now project (`project_rejects_distinct_real_normalization_collision`: exits non-zero, names both keys, writes no output). Seed matching on authoritative inputs runs over normalized keys on both sides. The projected-input path is now normalized too (W2 resolved).
- **P9**: `dedup_provenance` (src/types.rs, `@kb` annotated) runs at both merge boundaries: `load_atom_inputs`, which covers `merge_atom_files` and `merge_atom_files_raw`, and `cmd_merge` for all categories. Comparison uses full structural equality, including flattened `source` extensions (`dedup_provenance_collapses_identical_entries_only`). First-occurrence order keeps output deterministic for a given argument order (P14). P9 disclaims order semantics, so this is not a violation. Downstream probes pin `v0.4.0` and don't use the changed `load_validated_atom_file` signature (checked by grep over ../probe-*).
- **P10**: categorized arrays are edited in place inside `extensions`. No extension is dropped by normalization or trimming.
- **P13 / P27**: records untouched by this PR. Projection selection is dependency-only: `test_selection_is_dependency_only` shows a record-only path doesn't select `f`. The record-shape validation still runs on project's authoritative path via `prepare_atoms` → `normalize_atoms`.
- **P14**: all new containers are BTreeMap or Vec. Dedup preserves input order and trimming preserves array order.
- **P15**: `project_atoms` trims the categorized arrays with the same `included` filter as `dependencies` and leaves non-string entries alone (`test_categorized_arrays_trimmed_with_dependencies`). Normalization keeps the set equality `dependencies` = ∪ subsets.
- **P17**: category detection and enforcement unchanged.
- **P23**: project recomputes on the full authoritative graph before seed matching and BFS. The stale-label regression is covered by a roundtrip test. Projected inputs, in both the `probe/projected-atoms` and legacy `projection`-field forms, inherit labels (`project_inherits_labels_of_legacy_format_projection`; `projected` flag in src/authority.rs). Enrich and merge still reject projections on the projection predicate, not the version gate (`projection_readable_by_consumers_rejected_by_recomputation`).
- **ADR-006 Decision 7 / version gate**: Cargo.toml `version = "0.5.0"` equals `PROBE_GATE_MIN` (src/authority.rs:53), and the ceiling is unchanged. Real merge outputs pass the hub's own gate in enrich, summary and project (roundtrip tests). The schema.md:63 example is stamped `0.5.0`, matching the manifest.
- **Executable schema**: merged specs/proofs `inputs` `minItems` is 1, with an updated description. The same-source specs merge collapses to one entry and still validates (roundtrip test). The merged-atoms branch was already `minItems: 1` (its description is W3).
- **Docs**: probe-merge.md Phase 2 and stats table, probe-project.md Step 1 (`load_validated_atom_file`) and Step 2 (collision rejection), architecture.md:30–33 (project runs the gate only, and propagate hosts carrier preparation) and README all match the code. `./scripts/check-kb-links.sh` reports "All KB links OK".
- **P2, P16, P19, P21, P22, P24, P25**: not touched by this changeset. I spot-checked that no changed file affects them.

Suite (re-audit, after the fix delta): `cargo test` (probe crate) passed 209 tests with 0 failures (lib 133, authority 19, merge 13, merge_laws 4, propagate 13, roundtrip 11, schema_validation 16). `cargo test --workspace` passed 266 with 0 failures. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check` are clean.
