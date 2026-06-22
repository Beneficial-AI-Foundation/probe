# Verification Statuses

Defines the per-atom status fields (from the tool schemas) and the color scheme derived from them. Color counts are produced by [`scripts/count-colors.sh`](../scripts/count-colors.sh); this document and that script must agree.

## Atom kinds

| Kind | Description | Examples |
|------|-------------|----------|
| **Implementation** | Executable code that can have specs attached | Rust functions, Verus exec-defs, Aeneas-generated Lean `def`s |
| **Specification** | Logical statements that define or prove properties | Verus spec-defs and `proof fn`, Lean `theorem`/`lemma`, non-translation `def`s |

Implementations can have specs attached; specifications cannot — they *are* the specs (always `unspecified`).

## Status fields

### `verification-status`

| Value | Meaning |
|-------|---------|
| `transitively-verified` | Verified, and every transitive dependency is verified or trusted ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)) |
| `verified` | Verified locally, but some transitive dependency is `unverified`/`failed` |
| `unverified` | Has sorries, admits, or warnings |
| `failed` | Compile/verification errors |
| `trusted` | Axiomatically assumed (`axiom`, `#[verifier::external_body]`, `admit()`) |
| `null` | Not subject to verification (tests, constants, external stubs) |

The `transitively-verified` vs `verified` split is computed by `probe enrich` (reverse-BFS contamination); probe-verus and probe-aeneas run it as the last step of `extract`.

What `"verified"` asserts is defined by each tool's schema and differs by pipeline:

- **Aeneas/Lean** — [derived from the primary spec theorem](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md#rust-specific-fields): the spec's status *is* the function's status; no spec ⇒ `"unverified"`. So `"verified"` always implies a proven spec.
- **Verus** — [mapped from the proof run](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#verification-status-mapping) (`success → "verified"`), independent of spec presence.

### `specified`

An implementation is `specified` if its `specs` list is non-empty, else `unspecified`.

## Colors

Each implementation gets one color from its (spec status, verification status). Colors apply only to project Rust functions (`language: "rust"`, non-empty `code-path`); external-crate stubs (`code-path: ""`) are excluded.

| # | Color | Meaning |
|---|-------|---------|
| 1 | Grey | Not subject to verification, disabled (`is-disabled: true`), or a test |
| 2 | White | Tracked, but not yet translated (Aeneas) / no spec yet (Verus, Lean) |
| 3 | Yellow | Translated (Rust→Lean via Aeneas) but not yet specified |
| 4 | Light Blue | Has a spec, not yet validated — *placeholder, see below* |
| 5 | Dark Blue | Has a validated spec |
| 6 | Light Green | Spec proven; some transitive dep is `unverified`/`failed` |
| 7 | Dark Green | Spec proven; no transitive dep is `unverified`/`failed` |
| — | Purple | Trusted: intentionally axiomatic or excluded |

Progression: Grey → White → Yellow → Light Blue → Dark Blue → Light Green → Dark Green. Purple is a separate branch (intentional assumption, not incomplete work).

Green requires a *proven spec*, so it is always a subset of Dark Blue — never just "the code compiles".

**Specifications** (not implementations) only ever take Light Green / Dark Green (same transitive rule) or Purple (axiomatic).

> **Light Blue is currently unused.** A spec is present in the JSON only if it is on `main` (passed PR review), so every present spec is treated as validated → Dark Blue. Light Blue is reserved for future spec-invalidation support.

### Framework differences

The ladder above is the Aeneas case; others use a subset:

- **Rust only** — no verification framework: everything is Grey.
- **Verus** — spec and proof are one verifier pass, so the Blue tiers don't apply (no "specified but unproven" state), and there is no translation step (no Yellow): functions go White (no pre/post) → Green. Trusted = `#[verifier::external_body]` / `admit()`; Grey = `#[test]`.
- **Lean** — full ladder except Yellow (no translation step). White = no specs yet. Trusted = `axiom` or `*External.lean`.
- **Aeneas** — full ladder. Each Rust function maps to one Lean translation and inherits its primary spec's status; Yellow = translated, no spec. Trusted = external stub or a trusted translation.

### Counting

```bash
scripts/count-colors.sh input.json   # auto-detects probe-aeneas / probe-verus extract JSON
```

Scoped to project functions (`code-path != ""`); Grey includes tests. Grey, White, Yellow, Dark Blue, and Purple (implementations) partition the total; axiom specs are counted separately. Green is an overlay within Dark Blue (the proven subset).

Counting Green requires both a verified status *and* a non-empty spec — redundant for Aeneas (status is spec-derived) but a deliberate guard for Verus (status is proof-derived). It keeps `green ≤ dark_blue` regardless of how upstream computes the status.

## Open questions

1. Should `failed` get its own color (red), distinct from `unverified`?
2. For Aeneas, `is-relevant == !is-disabled`; should the redundant `is-relevant` be dropped? See [probe-aeneas#20](https://github.com/Beneficial-AI-Foundation/probe-aeneas/issues/20).
