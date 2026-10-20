---
auditor: code-quality-auditor
date: 2026-10-20
repo: probe (hub)
kb: kb/ (own KB). The tree is on branch la/keep-translated-labels. It contains uncommitted KB amendments for issue #92. They are in ADR-006, P16, P23, the glossary (with a new `code atom` entry) and schema.md. They are also in architecture.md, the two indexes and kb/tools/probe-aeneas.md. I compared the property text with origin/main (82ec129) per common skill §2b. The code conforms only to the amended text. See I1.
scope: Round 3. Working-tree changeset vs origin/main 82ec129 for hub issue #92, after the fixes for the round 2 findings. Files changed are src/commands/propagate.rs, src/types.rs, src/commands/summary.rs, src/main.rs, a test in src/commands/merge.rs, tests/merge_laws.rs, tests/roundtrip.rs, Cargo.toml, Cargo.lock, CHANGELOG.md and the KB files above. I also examined the reporting paths in src/commands/{merge,project}.rs, src/authority.rs, README.md, docs/, kb/tools/probe-lean.md and probe-leanblueprint/docs/SCHEMA.md. I did not audit the untracked scratch files or the other reports.
status: 0 critical, 0 warnings, 1 info
---

## Critical

None.

## Warnings

None.

## Info

### [I1] Merge-order constraint: the code conforms only to the unmerged KB amendment
- **Location**: src/commands/propagate.rs:124, kb/engineering/properties.md:224 and 230 (origin/main text)
- **Issue**: On origin/main, P23 says that a marked atom that arrives as `transitively-verified` is rewritten to `verified` unconditionally. The new code violates that text. The code conforms only together with the KB amendments in the same working tree.
- **Recommendation**: Ship the KB amendment and the code in the same PR, which closes #92. Keep the CHANGELOG note that probe-aeneas#78 (probe-aeneas 0.22.0) is necessary to produce copied `transitively-verified` labels. The repo cannot show that the amendment has human intent. The caller says that it does (issue #92).

## Round 2 findings, now resolved

- W1 (doc block): `has_translation_origin` now sits above the `@kb:` line and the doc block of `validate_status_origins` (src/types.rs:158-167). `validate_status_origins` has its doc and its `schema.md#common-optional-fields` link again (src/types.rs:169-180).
- W2 (KB wording): ADR-006:161 and 194-195 and glossary lines 67 and 197 now say "code atom". The new glossary `code atom` entry (glossary.md:57-59) defines it as `language` not `"blueprint"`. This is the same test as `is_kept_copied_label` (propagate.rs:30).
- I2 (comments): propagate.rs:122 and 193 say "code atoms".

## Verified clean

I examined these against the changeset. For P23 I used the amended text (see I1).

- **P3** (stub detection is structural): Stub classification did not change. The new tests use stubs with lines 0,0 and an empty `code-path`.
- **P4** (associativity on the carrier): Enrichment still rewrites only `verified` and `transitively-verified`. Kept labels and the `language` field do not change stub classification or first-wins selection. `laws_hold_with_translated_labels` and `laws_hold_with_translated_labels_and_trust` pass. The trust sweep now has a caller key `c` that depends on `r`. Thus it also covers caller blocking through a translated or trusted `r`.
- **P5** (identity): The fixed-point assertions in the same tests cover it.
- **P6, P10, P13, P27**: Conflict resolution, extension preservation and correspondence records did not change. Records stay out of the BFS.
- **P14** (deterministic output): All new iteration is over `BTreeMap` or `BTreeSet`. The count loop changes only integers.
- **P16** (verification status mapping): The amended Enrichment paragraph agrees with the code.
- **P23** (amended text): Translated code atoms are not candidates (propagate.rs:124). They stay in the reverse index and in the seed set (`is_seed`, presence-based). Their callers stay blocked. A copied `trusted` with a marker is still not a boundary. Kernel-taint and malformed markers are still demoted. Blueprint aggregates are relabeled. Kept labels are counted by label. Enrichment is idempotent. The snapshot caveat (properties.md:243) and schema.md:312 talk about copied labels. probe-aeneas writes copied labels only on Rust code atoms (ADR-006 Decision 2). Thus these passages agree with the code-atom rule.
- **Summary consumer (ADR-006 Decision 9)**: `imported_verified` uses the shared `has_translation_origin` and includes translated `transitively-verified` atoms. Blueprint atoms stay out of every list.
- **`@kb:` annotations**: `is_kept_copied_label`, `has_translation_origin` and `validate_status_origins` each have a correct link. The link check passes.
- **Version gate**: No new threshold is necessary, as schema.md:312 says. authority.rs accepts 0.6.0.
- **Versioning**: Cargo.toml is 0.6.0. CHANGELOG has an `[Unreleased]` `Changed` entry that names the code-atom restriction and the count change. Cargo.lock agrees.
- **Docs**: main.rs help, README.md:60-63, docs/testing-guide.md:151, architecture.md steps 8-10, schema.md (example version 0.6.0, `verification-status` and `status-origin` rows), kb/tools/probe-aeneas.md and kb/tools/probe-lean.md agree with the code.
- **Build checks**: `cargo test` passes. `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt -- --check`, `cargo doc --no-deps` with `-D warnings` and `./scripts/check-kb-links.sh` are clean.

Not audited again because the changeset does not touch them: P1, P2, P7, P8, P9, P15, P17, P19, P21, P22, P24, P25.
