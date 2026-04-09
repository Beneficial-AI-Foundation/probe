---
auditor: code-quality-auditor
date: 2026-04-07
scope: probe-lean after `trusted-reason` (v0.4.5)
status: 0 critical, 1 warning, 8 info
---

## Critical

None

## Warnings

- **README example `tool.version` is stale:** The “Example Output” JSON in [README.md](../../../probe-lean/README.md) still shows `"version": "0.2.0"` while [lakefile.toml](../../../probe-lean/lakefile.toml), [ProbeLean/Version.lean](../../../probe-lean/ProbeLean/Version.lean), [docs/USAGE.md](../../../probe-lean/docs/USAGE.md), and [docs/SCHEMA.md](../../../probe-lean/docs/SCHEMA.md) are aligned on **0.4.5**. This undermines the documented single-source-of-truth story for newcomers reading the README first.

## Info

- **P16 vs implementation:** [Extract.lean](../../../probe-lean/ProbeLean/Extract.lean) `mapVerifyStatus` maps sorry outcomes to `verified` / `unverified` / `failed` as in [P16](../engineering/properties.md#p16-verification-status-mapping). `trustedReason` returns `some "axiom"` when `kind == axiom`, else `some "external"` when `codePath.endsWith "External.lean"`, else `none` — matching [probe-lean.md](../tools/probe-lean.md) (axiom precedence over external when both apply). `unifyAtom` sets `verificationStatus` to `trusted` iff `trustedReason` is `some`, and assigns `trustedReason` in parallel; non-trusted atoms omit both `verification-status` and `trusted-reason` in JSON via [Types.lean](../../../probe-lean/ProbeLean/Types.lean) `ToJson` (optional fields omitted when `none`).

- **P16 KB surface:** Canonical [properties.md](../engineering/properties.md#p16-verification-status-mapping) documents `verification-status` only; `trusted-reason` is specified in [probe-lean.md](../tools/probe-lean.md), [schema.md](../engineering/schema.md), and [glossary.md](../engineering/glossary.md). No contradiction — consider a one-line cross-reference under P16 for readers who use properties.md alone.

- **Version consistency (0.4.5):** `lakefile.toml`, generated `ProbeLean/Version.lean`, `CHANGELOG.md` [0.4.5] entry, `docs/USAGE.md` and `docs/SCHEMA.md` examples, and [Tests/Main.lean](../../../probe-lean/Tests/Main.lean) `testVersionConsistency` / `ProbeLean.version` checks are aligned. Stale tooling doc: [docs/manual-test-schema-2.0.md](../../../probe-lean/docs/manual-test-schema-2.0.md) still lists `tool.version` **0.1.0** in checklist rows (pre-existing drift; same class of issue as README).

- **Tests:** `lake build tests` and `.lake/build/bin/tests` report **303 passed, 0 failed** (verified 2026-04-07). [Tests/Main.lean](../../../probe-lean/Tests/Main.lean) covers `trustedReason` precedence and negatives, `unifyAtom` `trustedReason` / override behavior, and example JSON invariants (`all trusted atoms have trusted-reason`, valid reason strings, non-trusted atoms omit the field).

- **TESTING.md drift:** [TESTING.md](../../../probe-lean/TESTING.md) still describes `testTrustedStatus` only in terms of `isTrustedAtom` and older `unifyAtom` bullets; it does not mention `trustedReason` or the six checks in `testExampleJsonVerificationStatus` (table still says “3” for that function).

- **Serialization / P14:** [Types.lean](../../../probe-lean/ProbeLean/Types.lean) emits `trusted-reason` only when `trustedReason` is `some`, immediately after `verification-status` when present — stable field order for trusted atoms. [testDeterminismInvariants](../../../probe-lean/Tests/Main.lean) uses pairwise sorted key checks for example `data` (prior audit’s bogus `keysSorted` issue is resolved).

- **`--skip-verify`:** When verification is skipped, `proofEntry` is absent so `verificationStatus` stays `none` for non-trusted atoms; trusted atoms still get `trusted` + `trusted-reason`. Matches “absent when skipped” language in schema docs.

- **viewify:** [View.lean](../../../probe-lean/ProbeLean/View.lean) / [Loader.lean](../../../probe-lean/ProbeLean/Loader.lean) continue to load `AtomsOutput` (`Atom`); unified-only fields are not required for molecules. No change needed for `trusted-reason` on that path.
