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

Colors provide visual feedback based on verification and specification status.

| Color | When Used |
|-------|-----------|
| **Grey** | `null` or `trusted` |
| **Green** | `verified` |
| **Blue** | `unverified`/`failed` and (`specified` OR is a specification) |
| **White** | `unverified`/`failed` and `unspecified` (implementations only) |

Future: `trusted` will become **Purple** instead of Grey.

### Color Priority Rules

**For Implementations:**

1. `null` → Grey
2. `trusted` → Grey
3. `verified` → Green
4. `unverified`/`failed` + `specified` → Blue
5. `unverified`/`failed` + `unspecified` → White

**For Specifications:**

1. `null` → Grey
2. `trusted` → Grey
3. `verified` → Green
4. `unverified`/`failed` → Blue

Specifications skip the White color because they have no specification status—unverified specifications are always Blue.

## Framework-Specific Behavior

### Rust Only

All functions are **Grey** (`null`). No formal verification is performed.

### Rust with Verus

**Implementations (exec-defs):**

| Condition | Verification | Specification | Color |
|-----------|--------------|---------------|-------|
| Has `requires`/`ensures`, proofs complete | `verified` | `specified` | Green |
| Has `requires`/`ensures`, has `assume` | `unverified` | `specified` | Blue |
| Subject to verification, but no pre/post conditions yet  | `unverified` | `unspecified` | White |
| `#[verifier::external]` | `trusted` | — | Grey |
| `#[test]` function | `null` | — | Grey |

**Specifications (`proof fn`, spec-defs):**

| Condition | Verification | Color |
|-----------|--------------|-------|
| Proofs complete | `verified` | Green |
| Has `assume` or incomplete | `unverified` | Blue |
| Trusted assumption | `trusted` | Grey |

### Lean Only

**Implementations (`def` with associated theorems):**

| Condition | Verification | Specification | Color |
|-----------|--------------|---------------|-------|
| Has specs, all proven | `verified` | `specified` | Green |
| Has specs, contains `sorry` | `unverified` | `specified` | Blue |
| Subject to verification, but no specs | `unverified` | `unspecified` | White |
| Not subject to verification | `null` | `unspecified` | Grey |

**Specifications (`theorem`, `lemma`, standalone `def`):**

| Condition | Verification | Color |
|-----------|--------------|-------|
| Compiles successfully | `verified` | Green |
| Contains `sorry` | `unverified` | Blue |
| `axiom` declaration | `trusted` | Grey |

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
- External function stubs → `trusted` (Grey)

**Color assignment:**

| Condition | Verification | Specification | Color |
|-----------|--------------|---------------|-------|
| Primary spec proven | `verified` | `specified` | Green |
| Primary spec has `sorry` | `unverified` | `specified` | Blue |
| No primary spec, has translation | `unverified` | `unspecified` | White |
| No primary spec, no translation | `unverified` | `unspecified` | White |
| External stub | `trusted` | — | Grey |

## Open Questions

1. Should `failed` have its own color (red) distinct from `unverified`?
2. How to identify the "primary spec"? Options:
   - Naming convention (e.g., `foo_spec` for function `foo`)
   - Annotation system in Lean
   - First spec in declaration order
