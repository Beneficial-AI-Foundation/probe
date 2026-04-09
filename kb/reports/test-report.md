---
auditor: test-quality-auditor
date: 2026-04-07
scope: probe-lean — `trusted-reason` field (post-introduction audit)
repo: /home/lacra/git_repos/baif/probe-lean
test_file: Tests/Main.lean
status: 0 critical, 1 warning, 7 info
ci: lake build tests && .lake/build/bin/tests → 303 passed, 0 failed
---

## Critical

_None._ No blocking issues found for `trusted-reason` behavior or the tests added for it.

## Warnings

1. **`UnifiedAtom` JSON serialization of `trusted-reason` is not round-trip tested.** `testUnifiedAtomJson` exercises `verification-status` and optional-field preservation but never sets `trustedReason` on a constructed `UnifiedAtom`, never asserts that `Lean.toJson` emits `"trusted-reason"`, and never checks `fromJson?` restores `"axiom"` / `"external"`. The schema and `Types.lean` instances are straightforward, but a regression (omitting the field on write, wrong key name, or parse default) would not be caught by unit tests—only indirectly via `testExampleJsonVerificationStatus` on the committed example extract.

## Info

1. **`trustedReason` (pure logic)** is covered in `testTrustedStatus`: axiom → `"axiom"`, `*External.lean` def/theorem → `"external"`, axiom in external file → `"axiom"` (precedence over external), normal def → `none`. Aligns with `ProbeLean/Extract.lean`.

2. **`unifyAtom` + `UnifiedAtom.trustedReason`** is covered for the main matrix: axiom and external (with and without proof entry), normal def (with and without proof). Status and reason are asserted together for `unified1`–`unified5`.

3. **Symmetry gap (low severity):** For `unified6`–`unified9` (axiom/external with sorries or failure proof entries), tests assert `verification-status` becomes `trusted` but do **not** assert `trustedReason` remains `some "axiom"` / `some "external"`. Behavior is fixed by construction (`reason := trustedReason atom` independent of proof), so this is consistency/documentation coverage rather than a logic hole.

4. **Example extract JSON** (`examples/lean_Curve25519Dalek_0.1.0.json`): `testExampleJsonVerificationStatus` asserts every `trusted` atom has `trusted-reason`, values are only `"axiom"` or `"external"`, and non-trusted atoms omit the key. Good integration coverage for real output shape.

5. **`viewify` / `StubEntry`:** No `trusted-reason` on the view/stub path (extract-only field). No gap for the stated feature.

6. **Edge cases not singled out in tests:** e.g. path not ending with `External.lean` but containing it, or case variants of the suffix—`trustedReason` uses `String.endsWith "External.lean"` only; only positive custom paths (`ModelsExternal.lean`, `CustomExternal.lean`) appear under `isTrustedAtom`, not negative controls.

7. **Collateral:** The earlier defective P14 assertion (`data` keys always passing) is **not** present in the current tree: `testDeterminismInvariants` uses a pairwise lexicographic check (`keysPairwiseSorted`) on `data` keys. Out of scope for `trusted-reason`, but confirms the regression called out in the previous report is addressed here.

## Coverage summary table (`trusted-reason`)

| Concern | Tests | Coverage | Notes |
|---------|-------|----------|-------|
| `trustedReason` classification | `testTrustedStatus` (5 assertions) | **Strong** | Axiom, external, precedence, non-trusted. |
| `unifyAtom` → `UnifiedAtom.trustedReason` | `testTrustedStatus` (6 assertions on `unified1`–`unified5`) | **Strong** | With/without proof entry for trusted and normal atoms. |
| Trusted + sorry/failure overrides | `testTrustedStatus` (`unified6`–`unified9`) | **Partial** | Status only; `trustedReason` not re-asserted. |
| Real extract JSON invariants | `testExampleJsonVerificationStatus` (3 assertions) | **Strong** | Present/valid/absent rules on full example file. |
| `ToJson` / `FromJson` for `trusted-reason` | `testUnifiedAtomJson` | **Gap** | No constructed atom with `trustedReason`; no round-trip. |
| Docs / schema alignment | Manual | **Strong** | `docs/SCHEMA.md`, `README.md` describe field; tests match schema wording. |

## Resolution of previous findings (probe-lean, prior `test-report.md`)

| ID | Previous finding | Status |
|----|------------------|--------|
| Critical | `testDeterminismInvariants` `data` keys check always true (`keys.all`) | **Resolved** in current `Tests/Main.lean` — pairwise sorted check on `data` object keys. |
| W1 | No double-`extract` / byte-identical JSON regression | **Open** (unchanged; not part of this audit). |
| W2 | P14 sorting on real JSON | **Improved** — array sorts + fixed key-order assertion; double-extract still absent (see W1). |
