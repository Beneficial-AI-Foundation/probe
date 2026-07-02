# Atom roles, statuses, and colors

Each atom has a **role** (implementation, spec, proof, or definition), a **verification status**, and a **color** derived from the two. [`scripts/count-colors.sh`](../scripts/count-colors.sh) implements the scheme described here; see [Counting](#counting).

Coloring depends on the kind of project. A **verification project** (Verus or Aeneas) pairs implementations with specs and asks whether each implementation verifies. A **generic Lean project** (mathlib-style) has no such pairing and can only ask whether a theorem is proved. After some terminology below, the two are described separately.

## Proved vs verified

The word "verified" hides two different claims.

- A **theorem is proved** when its body is free of `sorry`/`admit`, relative to an intended [trust base](../kb/engineering/glossary.md#trust-base). Proving stands on its own: it establishes a logical statement and says nothing about any implementation.
- An **implementation verifies against a spec**: this needs a spec chosen for it and a proof that it meets that spec. Verus checks this directly, since the spec lives on the Rust function and Verus proves the body meets it. In an Aeneas project it is established indirectly: Aeneas transpiles the Rust function to Lean, and a primary-spec theorem about that translation is proved in Lean. Aeneas itself only transpiles; the proving happens in Lean.

So `verification-status: "verified"` means "verifies against its spec" for an implementation, and simply "proved" for a bare theorem. This is also what separates the two project types: in  verification projects the goal is to verify implementations against their specs, while in generic Lean projects, the goal is to prove theorems.

## Status fields

### `verification-status`

| Value | Meaning |
|-------|---------|
| `transitively-verified` | Verified or proved, and every transitive dependency is verified or trusted ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)) |
| `verified` | Verified or proved locally, but some transitive dependency is `unverified`/`failed` |
| `unverified` | Has a `sorry` (Lean) or `assume()` (Verus), or warnings. An `admit()` is `trusted`, not `unverified` |
| `failed` | Compile or verification errors |
| `trusted` | Axiomatically assumed (`axiom`, `#[verifier::external_body]`, `assume_specification`, `admit()`) |
| absent | Not subject to verification (tests, constants, external stubs); the field is omitted, counted as `absent` |

The `transitively-verified` vs `verified` split is computed by `probe enrich` (reverse-BFS contamination), run as the last step of `extract` in probe-verus and probe-aeneas.

What `"verified"` asserts differs by pipeline:

- **Verus (direct):** the spec (`requires`/`ensures`) lives on the Rust function, and Verus [proves the body satisfies it](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#verification-status-mapping). A spec-less function is `is-disabled: true` and carries no status.
- **Aeneas (indirect):** a Rust function is `"verified"` only if it has a Lean translation whose [primary-spec theorem](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md#rust-specific-fields) is proved. A translated function inherits its translation's status, or `"unverified"` when the translation has no primary spec. A function with no translation `is-disabled: true` and carries no status.

### `specified`

An implementation is `specified` if it has a `primary-spec`. This is a verification-project notion; a generic Lean project has no impl/spec pairing, so every atom is `unspecified`. Where the `primary-spec` sits differs:

- **probe-verus:** on the function itself, as its inline `requires`/`ensures` (non-empty is equivalent to `is-disabled: false`).
- **probe-aeneas:** on the function's Lean translation, reached through `translation-name`, not on the Rust function itself.

A pure-Lean project has `specified` atoms only if it adopts a spec convention (an `@[primary_spec]`/`@[progress]`/`@[pspec]`/`@[step]` attribute, a `_spec` suffix, or Aeneas-generated code), which makes it a verification project; probe-lean then reads `primary-spec` from that convention ([P18](../kb/engineering/properties.md#p18-lean-specified-is-derived-not-stored)).

## Colors

Color encodes verification status. Atom roles represent a separate axis, derived from `kind` plus the `translation-name` and `primary-spec` links; language is `rust`, `verus`, or `lean`. VeriLib renders color as node color and role as node shape. `"trusted"` is Purple and `"failed"` is Red in every group. The palette is Grey, Yellow, Orange, Blue, Light Green, Dark Green, Purple, Red, and White. Orange flags an incomplete proof (a `sorry`/`assume`); Blue marks a Verus spec.

A pure-Rust project carries no verification information, so every atom is White. Grey is not used there, since it would imply verification intent.

The color tables depend on the project type, described next.

## Verification projects (Verus and Aeneas)

A verification project pairs each implementation with a spec and asks whether it verifies. Its atoms play four roles:

| Role | What it is | Provable? | Examples | Color |
|------|-----------|-----------|----------|-------|
| **Implementation** | executable code that verifies against a spec | it *verifies*, not "proved" | Rust fn, Verus exec, Aeneas Lean translation `def` | verify status |
| **Spec-as-property** | a stated condition; a definition with no proof of its own | no | Verus `spec fn` | Blue |
| **Proof** | discharges a proof obligation | yes | Verus `proof fn`, Lean supporting `theorem` | proof status |
| **Theorem-spec** | a Lean `theorem` that fuses the property (its type) with its proof (its term) | yes | Lean primary-spec `theorem` | proof status |
| **Definition / type decl** | supporting constructs with nothing to prove | n/a | Lean `def`/`abbrev`/`structure` | White |

These roles map onto Verus's own `spec`, `proof`, and `exec` [modes](https://verus-lang.github.io/verus/guide/modes.html), and their colors follow the proved-vs-verified split: an implementation is colored by whether it *verifies* against its spec; a spec-as-property is only *stated*, so it is Blue; a proof or theorem-spec is colored by whether it is *proved*.

### Implementations: does it verify against its spec?

Implementations are represented by Rust `exec` atoms, plus each Aeneas Lean translation `def` (a `def` that is some `exec`'s `translation-name` target). A translation inherits its Rust function's color and is counted under `lean` with the same status.

| state | color |
|-------|-------|
| Verus unspecified (`is-disabled: true`), or Aeneas not translated | Grey |
| translated or present but unspecified (no `primary-spec`) | Yellow |
| specified, proof incomplete (`"unverified"`) | Orange |
| specified, `"verified"` | Light Green |
| specified, `"transitively-verified"` | Dark Green |
| `"trusted"` | Purple |
| `"failed"` | Red |

Green requires a spec, so a spec-less implementation is never Green: it is Grey (Verus `is-disabled`, or Aeneas not translated) or Yellow (Aeneas translated but unspecified). An implementation `I` with a spec `S` is never Blue because Blue is overwritten by the status of the proof showing that `I` satisfies `S`; an unfinished proof is Orange.

### Specifications: Verus `spec fn`

A Verus `spec fn` (`kind: "spec"`) states a condition and carries no proof obligation, so it is Blue whatever its status, never Green. Aeneas states its conditions inside a theorem-spec (next group) rather than as standalone atoms, so this group is Verus-only.

| condition | color |
|-----------|-------|
| `kind: "spec"` (Verus spec fn) | Blue |
| `"trusted"` (rare) | Purple |
| `"failed"` (does not compile) | Red |

### Proofs and theorem-specs: is it proved?

Verus `proof fn` and every Lean `theorem`, including a theorem-spec (a primary-spec theorem that fuses property and proof). Colored by `verification-status`.

| condition | color |
|-----------|-------|
| `"trusted"` | Purple |
| `"failed"` | Red |
| `"unverified"` (a `sorry`) | Orange |
| `"verified"` | Light Green |
| `"transitively-verified"` | Dark Green |

### Definitions and type declarations

Constructs with nothing to prove: non-translation `def`/`abbrev`/`opaque`/`instance`/`projection`, and `structure`/`inductive`/`class`. A `def` that is an Aeneas translation target is an implementation instead, colored in the first group.

| condition | color |
|-----------|-------|
| `"trusted"` | Purple |
| `"failed"` | Red |
| `"unverified"` (a `def` whose body has a `sorry`) | Orange |
| type decl, sorry-free `def`, or no status | White |

A sorry-free `def` is White, not Green. probe-lean marks it `"verified"` because its body has no `sorry`, but that is not a proved property, and generic Lean cannot tell a spec from an implementation. A `def` with a `sorry` is Orange, so an incomplete obligation stays visible instead of hiding among definitions.

## Generic Lean projects

A generic Lean project (mathlib-style) has no mechanical rule pairing an implementation with its spec, so it cannot use the implementation or specification roles. The only claim it makes is whether a construct is proved, so only two of the groups above apply:

- A `theorem` is colored by proof status, as in [Proofs and theorem-specs](#proofs-and-theorem-specs-is-it-proved).
- Everything else (`def`, `structure`, and so on) is a definition, as in [Definitions and type declarations](#definitions-and-type-declarations).

Grey, Yellow, and Blue never appear: there are no excluded implementations, no translations, and no standalone specs. So a generic Lean project uses only Light Green, Dark Green, Orange, Red, Purple, and White.

## Notes

- **Progression:** Grey, Yellow, Orange, Light Green, Dark Green. Purple (intentional trust) and Red (failure) are separate branches. Blue and White sit off the ladder: Blue is a Verus spec, White is a browse-only atom, a type declaration, or a sorry-free definition.
- **Trusted reasons:** `trusted-reason` is Verus `"admit"`/`"external-body"`/`"assume-specification"`, or Lean/Aeneas `"axiom"`/`"externally_verified"`/`"external"` (from `*External.lean`). By convention a Verus `admit()` is intentional trust (Purple), while an `assume()` is an incomplete proof (`"unverified"`, Orange).
- **Separate axis:** Verus can also attach `spec-labels` (functional-correctness, bounds-safety, and so on) describing *what* a spec is about. That is independent of role, status, and color, and is not used here.

## Counting

[`scripts/count-colors.sh`](../scripts/count-colors.sh) counts a Schema 2.0 atoms file, single-tool or merged. It first drops the atoms VeriLib does not show (external-crate stubs with `code-path: ""`, and atoms flagged `is-hidden`/`is-ignored`/`is-extraction-artifact`), then assigns each remaining atom one color and one of four role groups, broken down by language:

- **Implementations:** Rust `exec` and Aeneas Lean translation `def`s. A translation inherits the color of the `exec` that points to it, so a function and its translation are counted separately (one `rust`, one `lean`), not deduped.
- **Specifications:** Verus `spec fn` (Blue).
- **Proofs and theorem-specs:** Verus `proof fn` and Lean `theorem`.
- **Definitions and type declarations.**

A generic Lean project populates only the last two groups. The groups partition the shown atoms, so `impl + spec + proof + def == shown`. The script warns if the partition fails, if a `translation-name` is dangling, or if a status is unrecognized. A browse-only file (no verification framework and no verification information) is reported as all White with no counts. `specified` is checked before proof status, so an unspecified `exec` is Grey or Yellow even if its status is `"verified"`; Green requires a spec, which relies on `has-spec ⟹ ¬is-disabled` ([P24](../kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope)).

`--per-atom` emits one JSON object per shown atom (`{id, language, group, kind, color}`), so VeriLib can paint node color and shape from the same classification.
