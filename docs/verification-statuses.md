# Verification Statuses

This document defines verification statuses for software verification projects across multiple proof frameworks (Rust only, Rust with Verus, Lean only, Rust with Lean and Aeneas).

## Atom Kinds

Atoms are classified into two kinds based on their role in verification:

| Kind | Description | Examples |
|----------|-------------|----------|
| **Implementation** | Executable code that can have specs attached | Rust functions, Verus exec-defs, Aeneas-generated Lean `def`s |
| **Specification** | Logical statements that define or prove properties | Verus spec-defs and `proof fn`, Lean `theorem`, `lemma`, non-translation `def`s |

Key distinction: **Implementations** can have specifications attached to them. **Specifications** cannot—they ARE the specs.

## Status Dimensions

### Verification Status (applies to both kinds)

| Status | Meaning |
|--------|---------|
| `transitively-verified` | Verified and all transitive dependencies are also verified or trusted ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)) |
| `verified` | Compiles successfully, all proofs discharged (but at least one transitive dep may be unverified/failed) |
| `unverified` | Has sorries, admits, or warnings |
| `failed` | Has compile errors |
| `trusted` | Axiomatically assumed (e.g., `axiom`, `#[verifier::external_body]`) |
| `null` | Not subject to verification (test functions, constants) |

The distinction between `"transitively-verified"` and `"verified"` is computed by `probe enrich` (reverse-BFS contamination over the dependency graph). probe-verus and probe-aeneas run this enrichment automatically as the last step of `extract`.

### Specification Status (applies to implementations only)

| Status | Condition |
|--------|-----------|
| `specified` | Has associated specs (`specs` list is non-empty) |
| `unspecified` | No associated specs (`specs` list is empty or null) |

Specifications are always `unspecified` by definition—they cannot have specs attached to them.

## Color Mapping

Colors provide visual feedback based on verification progress. The scheme follows a progression from untracked to fully verified, with a separate branch for trusted/axiomatic items.

**Verification scope:** Green comes in two strengths depending on whether explicit contamination (`"unverified"` or `"failed"`) exists in the transitive dependency closure:

- **Transitively verified** (`verification-status: "transitively-verified"`): The function is verified and no transitive dependency is explicitly `"unverified"` or `"failed"`. Dependencies with missing `verification-status` (untracked Rust functions, spec functions) and dependencies absent from the atom map are transparent — they do not block transitive scope. Trusted (axiomatic) dependencies are also transparent.
- **Locally verified** (`verification-status: "verified"`): The function is verified, but at least one transitive dependency is explicitly `"unverified"` or `"failed"`.

| Color | Status | Meaning |
|-------|--------|---------|
| **Dark Green** | `transitively-verified` | Function is verified and no transitive dependency is explicitly unverified or failed |
| **Light Green** | `verified` | Function is verified but at least one transitive dependency is explicitly unverified or failed |
| **Dark Blue** | Specified, specs validated | Has specifications and those specs have been proven correct |
| **Light Blue** | Specified, specs not validated | Has specifications written but they haven't been fully proven |
| **Light Cyan** | Translated | Translated (e.g., Rust→Lean via Aeneas) but not yet specified |
| **White** | Tracked, not yet specified | Tracked for verification but not yet specified (in Aeneas: not yet translated; in Verus/Lean: no spec written) |
| **Purple** | Trusted | Intentionally assumed correct—either axiomatic (Lean `axiom`) or excluded from verification (Verus `#[verifier::external_body]`) |
| **Grey** | Untracked/disabled | Not subject to verification (`null`), disabled, or excluded |

### Color Priority Rules

The progression from least to most verified:

```
Grey → White → Light Cyan → Light Blue → Dark Blue → Light Green → Dark Green
```

Purple (Trusted) is a special branch—it indicates intentional axiomatic assumptions rather than incomplete work.

**For Implementations:**

1. Untracked/disabled → Grey
2. Tracked, not yet specified → White
3. Translated, no specs → Light Cyan
4. Specified, specs not validated → Light Blue
5. Specified, specs validated → Dark Blue
6. Locally-scoped verified → Light Green
7. Transitively verified → Dark Green
8. Trusted (axiom or intentional exclusion) → Purple

**For Specifications:**

1. Locally-scoped verified → Light Green
2. Transitively verified → Dark Green
3. Trusted (axiomatic) → Purple

Specifications skip Grey, White, Light Cyan, and Blue tiers because they are not translated or specified—they ARE the specs. If a spec exists, it's already tracked.

## Framework-Specific Behavior

### Rust Only

Functions are **Grey** (untracked) — no formal verification framework is in use, so no verification will be performed.

### Rust with Verus

**Implementations (exec-defs):**

In Verus, spec validation and proof happen atomically in a single verifier pass — you cannot have "specs validated but function not verified." Therefore Blue tiers (which represent the gap between spec authoring and proof) do not apply. Blue tiers are meaningful for Lean/Aeneas where spec authoring and proof are separable steps.

| Condition | Color |
|-----------|-------|
| Proofs complete, no explicit `"unverified"` / `"failed"` in transitive deps | Dark Green |
| Proofs complete, at least one transitive dep is explicitly `"unverified"` / `"failed"` | Light Green |
| Subject to verification, but no pre/post conditions yet | White |
| `#[verifier::external_body]` (intentionally excluded) | Purple |
| `#[test]` function | Grey |

**Specifications (`proof fn`, spec-defs):**

| Condition | Color |
|-----------|-------|
| Proofs complete, no explicit `"unverified"` / `"failed"` in transitive deps | Dark Green |
| Proofs complete, at least one transitive dep is explicitly `"unverified"` / `"failed"` | Light Green |
| Trusted assumption (intentionally axiomatic) | Purple |

### Lean Only

**Implementations (`def` with associated theorems):**

| Condition | Color |
|-----------|-------|
| Has specs, all proven, no explicit `"unverified"` / `"failed"` in transitive deps | Dark Green |
| Has specs, all proven, at least one transitive dep is explicitly `"unverified"` / `"failed"` | Light Green |
| Has specs, specs validated (proven) | Dark Blue |
| Has specs, specs not yet validated | Light Blue |
| Subject to verification, but no specs | White |
| Not subject to verification | Grey |

**Specifications (`theorem`, `lemma`, standalone `def`):**

| Condition | Color |
|-----------|-------|
| Compiles successfully, no explicit `"unverified"` / `"failed"` in transitive deps | Dark Green |
| Compiles successfully, at least one transitive dep is explicitly `"unverified"` / `"failed"` | Light Green |
| `axiom` declaration (intentionally axiomatic) | Purple |

### Rust with Lean and Aeneas

Aeneas translates Rust functions to Lean definitions, which can then be specified and verified.

**Structure:**
- Each Rust function → one Lean translation (`def`)
- A translation may call other translations (e.g., loop bodies)
- Each translation may have one or more specs (theorems)
- One spec is the **primary spec**; others are auxiliary lemmas

**Status inheritance:**
- A Lean translation is `specified` if it has a primary spec
- The Rust function inherits status from its Lean translation
- External function stubs → `trusted` (Purple)

**Color assignment:**

| Condition | Color |
|-----------|-------|
| Primary spec proven, no explicit `"unverified"` / `"failed"` in transitive deps | Dark Green |
| Primary spec proven, at least one transitive dep is explicitly `"unverified"` / `"failed"` | Light Green |
| Primary spec written and validated | Dark Blue |
| Primary spec written but not validated | Light Blue |
| Has translation, no spec | Light Cyan |
| Tracked, no translation | White |
| External stub (intentionally excluded) | Purple |
| Untracked/disabled | Grey |

## Open Questions

1. Should `failed` have its own color (red) distinct from `unverified`?
2. How to identify the "primary spec"? Options:
   - Naming convention (e.g., `foo_spec` for function `foo`)
   - Annotation system in Lean
   - First spec in declaration order
3. Should Translated use Light Cyan or Light Purple? Light Cyan distinguishes it more clearly from the Blue tiers; Light Purple groups it visually closer to Trusted/Purple.
