---
marp: true
theme: default
paginate: true
---

# Lean Verification Landscape

**Specs as First-Class Citizens**

How probe-lean could discover specifications across different Lean project types

---

## The problem

In **Verus**, specs are syntactically explicit: `requires`, `ensures`, `proof fn`.

In **Lean 4**, specs hide in **types**, **attributes**, **naming**, and **module structure** — no single syntax marks "this is a spec".

**How could probe-lean find them?**

![w:1100](img/legend.png)

---

## The Lean project spectrum

| Project type | Example | Spec/impl split? |
|---|---|---|
| Pure math | Mathlib, FLT | No — theorems are the end product |
| Verified algorithm | lean-zip | Yes — native Lean vs. spec theorems |
| Formal reference spec | cedar-lean | Yes — Lean IS the spec |
| Verified library | Std `HashMap` + `LawfulBEq` | Yes — data structure + laws |
| Aeneas translation | baif/dalek-lean | Yes — Rust impl + Lean proofs |
| Crypto protocol | baif/secure-messaging | Yes — scheme + games |

---

## Spec pattern: Aeneas

Extrinsic specs via **`@[progress]`** theorems on Aeneas-generated Lean translations.

![w:1100](img/aeneas.png)

Color flows right-to-left: a Rust function is Dark Blue only when its translation has a spec.

---

## Spec pattern: lean-zip

Three-layer stack: **FFI opaques** / **native implementations** / **specs and proofs**.

![w:700](img/lean-zip.png)

Module paths (`Zip/Native/` vs `Zip/Spec/`) cleanly separate the layers.

---

## Spec pattern: Cedar

**Spec-as-implementation** — the Lean `def`s ARE the specification.

![w:700](img/cedar.png)

White nodes like `intOrErr` — should they have theorems? This is the **denominator problem**.

---

## Spec pattern: VCVio / secure-messaging

Layered: **scheme** → **correctness** → **security** → **invariants**.

![w:700](img/vcvio.png)

White = WIP, Light Green = locally verified invariants that don't yet compose transitively.

---

## Spec pattern: Loom/Velvet

Specs are **inline** — `requires`/`ensures` are part of the method declaration.

![w:550](img/loom.png)

`loom_solve` generates and discharges VCs automatically — verified methods go straight to Dark Green. Unspecified helpers (White) are the only nodes without inline annotations.

---

## The denominator problem

**What is the "base set" for measuring verification progress?**

| Project | Base set | How identifiable? |
|---|---|---|
| Verus/Aeneas | Rust functions | Automatic — `language: "rust"` |
| lean-zip | `def`s in `Zip/Native/` | Semi-automatic — module path |
| Cedar | `def`s in `Cedar/Spec/` | Semi-automatic — module path |
| secure-messaging | Scheme ops + security games | Requires domain knowledge |

probe-lean can identify what *is* specified (`specs` field non-empty), but determining what *should* be specified requires curation, conventions, or attributes.

---

## Discovery tiers

**Tier 1 — Attributes** (most robust)
- `@[spec]` (Std.Do.Triple), `@[progress]` (Aeneas), `@[primary_spec]` (probe-lean)
- Inspectable from the environment, cross-project consistent, linter-enforceable

**Tier 2 — Framework types** (moderate)
- `Triple`, `RelTriple` → spec; `def Correct` returning `Prop` → correctness
- Requires probe-lean to understand framework-specific types

**Tier 3 — Naming conventions** (most fragile)
- `*_spec`, `*_correct`, `*_preserves_*`, `*Inv`
- Already partially used by probe-lean for `primary-spec` inference

**Complement: Verso Blueprint** — project management layer (roadmap, dependencies, progress). Code-level attributes and blueprint are complementary, not alternatives.

---
