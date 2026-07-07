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


---

## Colour key

One colour per atom, from its verification status (the convention proposed in the probes deck):

- <span style="color:#808080">Grey</span> — in scope but not yet specified
- <span style="color:#C99A00">Yellow</span> — translated or generated, but unspecified
- <span style="color:#2563EB">Blue</span> — a stated spec, a condition that is not itself proved
- <span style="color:#E8710A">Orange</span> — incomplete proof (a `sorry` or `assume`)
- <span style="color:#43A047">Light Green</span> — verified locally, some dependency still open
- <span style="color:#1B5E20">Dark Green</span> — transitively verified: it and everything it depends on
- <span style="color:#7C3AED">Purple</span> — trusted (an axiom or an assumed spec)
- <span style="color:#D32F2F">Red</span> — error: does not compile, or verification fails
- White — nothing to grade: a definition, or outside the verification scope

<!-- The diagram images follow this key. They are generated from the .mmd sources in img/ with mermaid-cli: `mmdc -i img/<name>.mmd -o img/<name>.png -b white`. -->

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

Extrinsic specs via spec theorems on Aeneas-generated Lean translations. Grounded in `baif/curve25519-dalek-lean-verify`, three `FieldElement51` methods from `src/backend/serial/u64/field.rs`.

![w:1100](img/aeneas.png)

Color flows right-to-left: a Rust function is <span style="color:#1B5E20">Dark Green</span> only when its translation has a proved spec.

- `mul` is <span style="color:#1B5E20">Dark Green</span>: its translation carries the proved `mul_spec` (`Specs/.../FieldElement51/Mul.lean`).
- `conditional_swap` is <span style="color:#C99A00">Yellow</span>: it is translated but has no spec, so there is nothing yet to verify against.
- `zeroize` is <span style="color:#808080">Grey</span>: no translation at all, so it is outside the verification effort.

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

## Spec pattern: secure-messaging (AEAD from Encrypt-then-MAC)

Grounded in `baif/secure-messaging`. The construction and the property definitions are `def`s; the correctness and security claims are theorems.

![w:700](img/vcvio.png)

- `etmAEAD`, `Correct`, `guessAdvantage` are definitions: White, nothing to prove on their own.
- `etmAEAD_correct` and `etmAEAD_security` are <span style="color:#1B5E20">transitively verified</span> theorems that prove the construction meets `Correct` and bounds `guessAdvantage`.
- Every node's subgraph names the file (all under `SecureMessaging/`) where the atom lives.

---

## Spec pattern: Loom/Velvet

Specs are **inline** — `requires`/`ensures` are part of the method declaration.

![w:550](img/loom.png)

`loom_solve` generates and discharges VCs automatically, so verified methods go straight to <span style="color:#1B5E20">Dark Green</span>. Unspecified helpers (<span style="color:#808080">Grey</span>) are the only nodes without inline annotations.

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
