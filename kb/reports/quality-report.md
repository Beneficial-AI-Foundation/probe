---
auditor: code-quality-auditor
date: 2026-04-03
status: 0 critical, 0 warnings, 5 info
---

## Summary by property (P1–P20)

| Property | Scope of this audit | Result |
|----------|---------------------|--------|
| P1 Envelope completeness | `probe summary` output | **Satisfied** — `SummaryEnvelope` wraps payload ([C1] resolved; see verification below) |
| P2 Atom identity / unique keys | Hub types + merge + summary | Satisfied (`BTreeMap` keys; summary reads same model) |
| P3 Stub detection | `summary.rs` | Satisfied (`Atom::is_stub()`); `@kb` present |
| P4–P5 Merge associativity / identity | `merge.rs` | Satisfied (unchanged; tested) |
| P6–P7 Atom / specs-proofs merge | `merge.rs` | Satisfied (unchanged) |
| P8 Code-name normalization | `merge.rs` | Satisfied (unchanged) |
| P9 Provenance | `merge.rs` / `load_atom_file` / summary | Satisfied; summary forwards `inputs` from loaded envelope via `load_atom_file` |
| P10 Extensions through merge | `merge.rs` + `summary.rs` | Satisfied; summary reads `extensions` for `verification-status` |
| P11–P13 Translation rules | `merge.rs` | Satisfied (unchanged); summary N/A |
| P14 Deterministic output | `summary.rs` | Satisfied for payload ordering: `BTreeMap` iteration; lists follow sorted code-name order. Envelope `timestamp` is wall-clock (same pattern as `merge.rs`) |
| P15 Dependency completeness | probe-verus extract | Fixtures unchanged; not hub-owned |
| P16 Verification status mapping | probe-verus | Unchanged |
| P17 Schema category consistency | `merge.rs` / loaders | Satisfied; summary only loads atoms-category files |
| P18 Lean `specified` | N/A | N/A to files audited |
| P19 Cross-repo path deps | Not re-scanned | No change indicated |
| P20 Language from kind | `summary.rs` (`is_rust_exec`) | Satisfied; `@kb` links [schema.md](../engineering/schema.md#language-assignment-for-verus-atoms) |

## Critical

**None.** Previous **[C1]** is **resolved**.

**Verification**: `src/commands/summary.rs` defines `SummaryEnvelope` with `schema: "probe/summary"`, `schema-version: "2.0"` (via `schema_version` field + serde rename), `tool` (`Tool` struct), `inputs: Vec<InputProvenance>` populated from `load_atom_file` provenance, RFC3339 `timestamp`, and `data: SummaryResult`. This matches the Schema 2.0 merged-envelope variant ([schema.md](../engineering/schema.md#merged-envelope-variant)) and [P1](../engineering/properties.md#p1-envelope-completeness).

## Warnings

**None.** Previous findings addressed:

- **[W1]**: Seven unit tests in `summary.rs` cover partition size, stubs, test heuristic, Verus spec/proof exclusion, unverified exclusion, test-dep non-disqualification, and depended-upon vs entrypoint. `cargo test` passes.
- **[W2]**: `kb/engineering/architecture.md` lists `src/commands/summary.rs`, hub **Subcommands**: `merge`, `summary`.
- **[I1]** (prior): `@kb` annotations added on `summary.rs` for [P1](../engineering/properties.md#p1-envelope-completeness), [P3](../engineering/properties.md#p3-stub-detection-is-structural), [P14](../engineering/properties.md#p14-deterministic-output), [P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope), and [language assignment](../engineering/schema.md#language-assignment-for-verus-atoms).
- **[I2]** (prior): `src/main.rs` `Summary` docs match implementation and mention schema `probe/summary`.

## Info

### [I3] Glossary-aligned terminology (residual, low)
[product/spec.md](../product/spec.md) now documents **Entrypoint analysis** and `probe summary` under Core capabilities. A dedicated [glossary](../engineering/glossary.md) term for "entrypoint" remains optional for discoverability.

### [I4] `probe-verus` extract fixtures align with P20
No change; still consistent with [P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope) and [schema.md](../engineering/schema.md#language-assignment-for-verus-atoms).

### [I5] `probe/summary` listed under "Registered schema values" in schema.md
Implementation and [probe-summary.md](../tools/probe-summary.md) use `schema: "probe/summary"`, registered in [schema.md](../engineering/schema.md#registered-schema-values) under Analysis.

### [I6] `kb/tools/index.md` table includes `probe-summary.md`
[kb/index.md](../index.md) links [probe-summary.md](../tools/probe-summary.md); [tools/index.md](../tools/index.md) file table includes the row.

### [I7] Optional: envelope shape not covered by JSON-schema tests
`tests/schema_validation.rs` validates representative tool envelopes but not a `probe/summary` output. Low risk given struct-driven serialization; add a fixture test if regressions on P1 are a concern.
