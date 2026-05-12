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
| `verified` | Compiles successfully, all proofs discharged |
| `unverified` | Has sorries, admits, or warnings |
| `failed` | Has compile errors |
| `trusted` | Axiomatically assumed (e.g., `axiom`, `#[verifier::external]`) |
| `null` | Not subject to verification (test functions, constants) |

### Specification Status (applies to implementations only)

| Status | Condition |
|--------|-----------|
| `specified` | Has associated specs (`specs` list is non-empty) |
| `unspecified` | No associated specs (`specs` list is empty or null) |

Specifications are always `unspecified` by definition—they cannot have specs attached to them.

## Color Mapping

Colors provide visual feedback based on verification progress. The scheme follows a progression from untracked to fully verified, with a separate branch for trusted/axiomatic items.

**Verification scope:** Green comes in two strengths depending on how far we look for sorries:

- **Transitively verified**: The function body is sorry-free AND all its transitive dependencies are also sorry-free. Trusted (axiomatic) dependencies count as verified for this purpose — they are intentional assumptions, not incomplete work.
- **Locally-scoped verified**: The function body itself is sorry-free, but at least one transitive dependency is explicitly unverified or failed.

| Color | Status | Meaning |
|-------|--------|---------|
| **Dark Green** | Transitively verified | Function is verified and no transitive dependency is explicitly unverified or failed |
| **Light Green** | Locally-scoped verified | Function is verified but at least one transitive dependency is explicitly unverified or failed |
| **Dark Blue** | Specified, specs validated | Has specifications and those specs have been proven correct |
| **Light Blue** | Specified, specs not validated | Has specifications written but they haven't been fully proven |
| **Light Cyan** | Translated | Translated (e.g., Rust→Lean via Aeneas) but not yet specified |
| **White** | Tracked, not yet specified | Tracked for verification but not yet specified (in Aeneas: not yet translated; in Verus/Lean: no spec written) |
| **Purple** | Trusted | Intentionally assumed correct—either axiomatic (Lean `axiom`) or excluded from verification (Verus `#[verifier::external]`) |
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
| Proofs complete, all transitive deps sorry-free | Dark Green |
| Proofs complete, transitive deps not checked | Light Green |
| Subject to verification, but no pre/post conditions yet | White |
| `#[verifier::external]` (intentionally excluded) | Purple |
| `#[test]` function | Grey |

**Specifications (`proof fn`, spec-defs):**

| Condition | Color |
|-----------|-------|
| Proofs complete, all transitive deps sorry-free | Dark Green |
| Proofs complete, transitive deps not checked | Light Green |
| Trusted assumption (intentionally axiomatic) | Purple |

### Lean Only

**Implementations (`def` with associated theorems):**

| Condition | Color |
|-----------|-------|
| Has specs, all proven, all transitive deps sorry-free | Dark Green |
| Has specs, all proven, transitive deps not checked | Light Green |
| Has specs, specs validated (proven) | Dark Blue |
| Has specs, specs not yet validated | Light Blue |
| Subject to verification, but no specs | White |
| Not subject to verification | Grey |

**Specifications (`theorem`, `lemma`, standalone `def`):**

| Condition | Color |
|-----------|-------|
| Compiles successfully, all transitive deps sorry-free | Dark Green |
| Compiles successfully, transitive deps not checked | Light Green |
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
| Primary spec proven, all transitive deps sorry-free | Dark Green |
| Primary spec proven, transitive deps not checked | Light Green |
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
