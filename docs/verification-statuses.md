# Atom roles, statuses, and colors

Each atom has a **role** (implementation, spec-as-property, proof, theorem-spec, or definition), a **verification status**, and a **color** derived from the two. [`scripts/count-colors.sh`](../scripts/count-colors.sh) implements the scheme described here; see [Counting](#counting).

Coloring depends on the kind of project. A **verification project** pairs implementations with specs and asks whether each implementation verifies with respect to its specification: Verus, Aeneas, or a Lean project making explicit which lean constructs are implementations or specifications. A **generic Lean project** (mathlib-style) can only ask whether a theorem is proved. After some terminology below, the two are described separately.

## Proved vs verified

The word "verified" hides two different claims.

- A **theorem is proved** when its body is free of `sorry`, relative to an intended [trust base](../kb/engineering/glossary.md#trust-base). Proving stands on its own: it establishes a logical statement and says nothing about any implementation.
- An **implementation verifies against a spec**: this needs a spec chosen for it and a proof that it meets that spec. In a Verus project, the spec lives on the Rust function. Verus then encodes the spec, the function and the proof as a Z3 formula. In an Aeneas project, the spec lives in a separate Lean file: Aeneas transpiles the Rust function to Lean, and a primary-spec theorem about that translation is proved in Lean.

So `verification-status: "verified"` means "verifies against its spec" for an implementation, and simply "proved" for a bare theorem. This is also what separates the two project types: in verification projects the goal is to verify implementations against their specs, while in generic Lean projects, the goal is to prove theorems.

## Status fields

### `verification-status`

| Value | Meaning |
|-------|---------|
| `transitively-verified` | Verified or proved, and every transitive dependency is verified or trusted ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)) |
| `verified` | Verified or proved locally, but some transitive dependency is `unverified`/`failed` |
| `unverified` | Has a `sorry` (Lean) or `assume()` (Verus), or warnings |
| `failed` | Compile or verification errors |
| `trusted` | Axiomatically assumed: Lean `axiom`; Verus `#[verifier::external_body]`, `assume_specification`, or `admit()` — a Verus `admit()` proof is deliberately accepted as an axiom, never `unverified` |
| absent | Not subject to verification (tests, constants, external stubs); the field is omitted, counted as `absent` |

The `transitively-verified` vs `verified` split is computed by reverse-BFS contamination ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)), run as the last step of `extract` in probe-verus, probe-aeneas, and probe-lean.

What `"verified"` asserts differs by pipeline:

- **Verus (direct):** the spec (`requires`/`ensures`) lives on the Rust function, and Verus [proves the body satisfies it](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#verification-status-mapping). A spec-less function is `is-disabled: true` and carries no status.
- **Aeneas (indirect):** a Rust function is `"verified"` only if it has a Lean translation whose [primary-spec theorem](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md#rust-specific-fields) is proved. A translated function inherits its translation's status, or `"unverified"` when the translation has no primary spec. A function with no translation `is-disabled: true` and carries no status.

### `specified`

An implementation is `specified` when it has a `primary-spec` reflecting a deliberate spec pairing. Where the `primary-spec` sits — and what counts as deliberate — differs by tool:

- **probe-verus:** on the function itself, as its inline `requires`/`ensures` (non-empty is equivalent to `is-disabled: false`).
- **probe-aeneas:** on the function's Lean translation, reached through `translation-name`, not on the Rust function itself.
- **probe-lean:** the `primary-spec` is derived, not stored ([P18](../kb/engineering/properties.md#p18-lean-specified-is-derived-not-stored)) — from an `@[primary_spec]`/`@[progress]`/`@[pspec]`/`@[step]` attribute or a `<def>_spec` naming convention. A `def` also carries `rust-source` when Aeneas generated it. A Lean `def` is paired with a Rust function — and so an implementation — when it is a translation target, has such a **documented** `primary-spec`, or has `rust-source` (see [Implementations](#implementations-does-an-implementation-verify-against-its-spec)). A generic Lean project has none of these, so every `def` is an unspecified definition.

  probe-lean additionally emits a `primary-spec` by **sole-spec inference**: when a `def` is referenced by exactly one theorem, that theorem is recorded as its primary spec. "One theorem happens to cite this def" is not a pairing claim the project authors made, so consumers must not treat it as one: a `def` whose `primary-spec` comes only from sole-spec inference is **not** an implementation. `count-colors.sh` accepts a `def`'s `primary-spec` only when *documented* — the spec theorem carries one of the attributes above or follows the `<def>_spec` naming convention.

## Colors

Color encodes verification status. Atom roles are a separate axis, derived from `kind` plus the `translation-name`, `primary-spec`, and `rust-source` links; language is `rust`, `verus`, or `lean`.  `"trusted"` is Purple and `"failed"` is Red in every group. The palette is Grey, Yellow, Orange, Blue, Light Green, Dark Green, Purple, Red, and White. Orange flags an incomplete proof (a `sorry`/`assume`); Blue marks a Verus spec.

A pure-Rust project carries no verification information, so every atom is White. Grey is not used there, since it would imply verification intent.

The color tables depend on the project type, described next.

## Verification projects

A verification project pairs each implementation with a spec and asks whether it verifies. Its atoms play the following roles:

| Role | What it is | Provable? | Examples | Color* |
|------|-----------|-----------|----------|-------|
| **Implementation** | executable code that verifies against a spec | it *verifies*, not "proved" | Rust fn, Verus exec, Lean `def` paired with a Rust fn | verify status |
| **Spec-as-property** | a stated condition; a definition with no proof of its own | no | Verus `spec fn` | Blue |
| **Proof** | discharges a proof obligation | yes | Verus `proof fn`, Lean supporting `theorem` | proof status |
| **Theorem-spec** | a Lean `theorem` that fuses the property (its type) with its proof (its term) | yes | Lean primary-spec `theorem` | proof status |
| **Definition / type decl** | supporting constructs with nothing to prove | n/a | Lean `abbrev`/`structure`, plain `def` | White |

\* before the universal overrides (`trusted` → Purple, `failed` → Red); a `def` containing a `sorry` is Orange.

These roles map onto Verus's own `spec`, `proof`, and `exec` [modes](https://verus-lang.github.io/verus/guide/modes.html), and their colors follow the proved-vs-verified split: an implementation is colored by whether it *verifies* against its spec; a spec-as-property is only *stated*, so it is Blue; a proof or theorem-spec is colored by whether it is *proved*.

### Implementations: does an implementation verify against its spec?

Implementations are Rust `exec` atoms, plus any Lean `def` paired with a Rust function — a translation target (some `exec`'s `translation-name` points at it), a `def` with a *documented* `primary-spec` (from an attribute or `<def>_spec` naming; sole-spec inference does not count), or an Aeneas-generated `def` (it has `rust-source`). Membership needs only one of the three. Every implementation is colored by the same ladder — a Rust `exec` by its own `verification-status`, a Lean `def` by its documented primary-spec theorem's status (Yellow when it has no documented spec):

| state | color |
|-------|-------|
| Verus unspecified (`is-disabled: true`), or Aeneas not translated | Grey |
| translated or Aeneas-generated, but unspecified | Yellow |
| specified, proof incomplete (`"unverified"`, or status missing) | Orange |
| specified, `"verified"` | Light Green |
| specified, `"transitively-verified"` | Dark Green |
| `"trusted"` | Purple |
| `"failed"` | Red |

A Lean `def` is graded by its **own** documented primary-spec theorem — the same Lean spec that probe-aeneas propagates onto the Rust `exec`'s status — not by borrowing the exec's color. This is the natural direction: in Aeneas the verdict originates in the Lean proof and flows to the Rust function, so the Lean translation reads it at the source. A `def` with no documented spec is Yellow (translated or generated, but unspecified).

Because the `exec` and its translation each read their status independently, they normally show the same color, but can differ when probe-aeneas's per-node transitive enrichment lands differently on the two atoms — e.g. an `exec` marked `transitively-verified` (Dark Green) whose spec theorem is only locally `verified` (Light Green). Both are "verified"; the shades differ, and the split reflects the underlying data rather than being smoothed over.

Each function is *counted* once: when an exec and its Lean stand-in are both shown, the count lives on the exec and the stand-in is only painted (VeriLib renders both nodes; see [Counting](#counting)). A probe-lean-only extract has no `exec` atoms, so there the stand-ins carry the implementation counts.

Green requires a spec, so a spec-less implementation is never Green: it is Grey (Verus `is-disabled`, or Aeneas not translated) or Yellow (translated or Aeneas-generated but unspecified). An implementation is never Blue: Blue marks a stated spec, while an implementation is colored by proof progress against its spec — an unfinished proof is Orange. `count-colors.sh` never reads `is-disabled`: a function that is in scope (`is-disabled: false`) but untranslated — e.g. `#[test]` functions that Aeneas skips — is Grey exactly like a disabled one, because "not translated" already means "outside the verification pipeline".

### Specifications: Verus `spec fn`

A Verus `spec fn` (`kind: "spec"`) states a condition and carries no proof obligation, so it is Blue regardless of proof progress — never Green — subject only to the universal overrides. Aeneas states its conditions inside a theorem-spec (next group) rather than as standalone atoms, so this group is Verus-only.

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

Constructs with nothing to prove: `def`/`abbrev`/`opaque`/`instance`/`projection`, and `structure`/`inductive`/`class` — except that a `def` that is a translation target, has a *documented* `primary-spec`, or is Aeneas-generated (`rust-source`) is an [implementation](#implementations-does-an-implementation-verify-against-its-spec) instead. A `def` whose `primary-spec` comes only from sole-spec inference stays here, as a definition.

| condition | color |
|-----------|-------|
| `"trusted"` | Purple |
| `"failed"` | Red |
| `"unverified"` (a `def` whose body has a `sorry`) | Orange |
| type decl, sorry-free `def`, or no status | White |

A sorry-free `def` is White, not Green. probe-lean marks it `"verified"` because its body has no `sorry`, but that is not a proved property, and generic Lean cannot tell a spec from an implementation. A `def` with a `sorry` is Orange, so an incomplete obligation stays visible instead of hiding among definitions.

## Generic Lean projects

A generic Lean project has no mechanical rule pairing an implementation with its spec, so it cannot use the implementation or specification roles. The only claim it makes is whether a construct is proved, so only two of the groups above apply:

- A `theorem` is colored by proof status, as in [Proofs and theorem-specs](#proofs-and-theorem-specs-is-it-proved).
- Everything else (`def`, `structure`, and so on) is a definition, as in [Definitions and type declarations](#definitions-and-type-declarations).

Grey, Yellow, and Blue never appear: there are no excluded implementations, no translations, and no standalone specs. So a generic Lean project uses only Light Green, Dark Green, Orange, Red, Purple, and White. In `count-colors.sh` output this is visible directly: the Implementations and Specifications groups are empty by construction, and empty groups print no table.

## Notes

- **Progression:** Grey, Yellow, Orange, Light Green, Dark Green. Purple (intentional trust) and Red (failure) are separate branches. Blue and White sit off the ladder: Blue is a Verus spec; White is a browse-only atom, a type declaration, a sorry-free definition, or a proof never matched to a verification result.
- **Trusted reasons:** `trusted-reason` is Verus `"admit"`/`"external-body"`/`"assume-specification"`, or Lean/Aeneas `"axiom"`/`"externally_verified"`/`"external"` (from `*External.lean`). By convention a Verus `admit()` is intentional trust (Purple), while an `assume()` is an incomplete proof (`"unverified"`, Orange).
- **Separate axis:** Verus can also attach `spec-labels` (functional-correctness, bounds-safety, and so on) describing *what* a spec is about. That is independent of role, status, and color, and is not used here.

## Counting

[`scripts/count-colors.sh`](../scripts/count-colors.sh) counts a Schema 2.0 atoms file, single-tool or merged. It first drops atoms with `code-path: ""` (corresponding to functions from external crates), and atoms flagged `is-hidden`/`is-ignored`/`is-extraction-artifact`, then assigns each remaining atom one color and one role, broken down by language. The groups are as follows:

- **Implementations:** Rust `exec`, plus Lean `def`s that are translation targets, specified (documented `primary-spec` only), or Aeneas-generated.
- **Specifications:** Verus `spec fn`.
- **Proofs and theorem-specs:** Verus `proof fn` and Lean `theorem`.
- **Definitions and type declarations:** everything else.

The tables count each function once: a Lean `def` whose exec is itself shown would repeat the exec's verdict, so it is excluded from the tables and reported in a footnote; when the exec is hidden or absent, the `def` carries the count. A group with nothing counted prints no table, so a generic Lean project shows only Proofs and Definitions.

A browse-only file (no verification framework and no verification information on any shown atom) is reported as all White with no counts.

In tables mode (not `--per-atom`) the script also warns about data problems; it never changes a color because of them:

- a `translation-name` or `primary-spec` naming an atom absent from the file (dangling);
- an unrecognized `verification-status` value;
- a Verus `proof` atom with no status ([probe-verus#33](https://github.com/Beneficial-AI-Foundation/probe-verus/issues/33));
- violations of [P24](../kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope) (disabled yet specified) and [P25](../kb/engineering/properties.md#p25-a-graded-atom-is-in-analysis-scope) (disabled yet graded).

`--per-atom` emits one JSON object per shown atom (`{id, language, group, kind, color}`) — including both atoms of an exec/stand-in pair, since VeriLib paints both nodes; the count-once rule applies to the tables only.
