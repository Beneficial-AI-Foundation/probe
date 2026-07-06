# Probes: factual data about (verified) code

---

## What the probes are

The probes use code indexers to extract structured data about a codebase. They read what the indexer already understands and write it down.

Each probe wraps the indexer that fits its language:

| Probe | Indexer it uses | Reads |
|-------|-----------------|-------|
| probe-rust | rust-analyzer | Rust source |
| probe-verus | verus-analyzer | Verus source (Rust plus Verus specs and proofs) |
| probe-lean | Lean metaprogramming | Lean source |

probe-aeneas has no indexer of its own. It uses probe-rust and probe-lean, and joins them (next slide).

---

## probe-aeneas: probe-rust plus probe-lean

An Aeneas project has two sides: the Rust crate, and the Aeneas-generated Lean that models it and the specs proved about it. probe-aeneas runs both probes and links their output.

- **probe-rust** indexes the Rust crate with rust-analyzer, and additionally runs Charon to tag each Rust function with a Charon-derived qualified name.
- **probe-lean** indexes the Lean side, where each Aeneas-generated definition remembers the Rust function it came from.

The Rust atoms carry rust-analyzer ids; the Lean translations speak Charon names. Charon is the shared vocabulary: tagging each rust-analyzer atom with its Charon-derived qualified name is what makes the two comparable. Matching those names links a Rust function to the Lean definition that implements it and the theorem that specifies it.

---

## The probes generate JSON

Every probe emits the same shape of data: one entry per code atom (a function, a spec, a proof, a definition), with its dependencies. What each pipeline can say about an atom depends on what its indexer knows.

| Project | Typical information per atom |
|---------|------------------------------|
| Rust | function calls (the call graph) |
| Verus, Aeneas | function calls plus verification status |
| Lean | dependencies plus proof status |

---

## A Verus atom

From `dalek-verus/.../verus_curve25519-dalek_4.1.3.json`. A Rust function, its calls, and whether it verifies against its spec.

```json
"probe:curve25519-dalek/4.1.3/.../[ProjectivePoint]double()": {
  "kind": "exec",
  "language": "rust",
  "code-path": "src/backend/serial/curve_models/mod.rs",
  "primary-spec": "requires\n  is_valid_projective_point(*self),\n  ensures ...",
  "verification-status": "transitively-verified",
  "dependencies": [
    "probe:.../[FieldElement51]square()",
    "probe:.../[FieldElement51]square2()"
  ]
}
```

---

## An Aeneas atom

From `curve25519-dalek-lean-verify/.../aeneas_curve25519-dalek_4.2.0.json`. The Rust function `EdwardsPoint::mul_base`, transpiled to Lean, paired with the theorem that specifies it, and its proof status.

```json
"probe:curve25519_dalek.edwards.EdwardsPoint.mul_base": {
  "kind": "def",
  "language": "lean",
  "rust-source": "curve25519-dalek/src/edwards.rs",
  "primary-spec": "probe:curve25519_dalek.edwards.EdwardsPoint.mul_base_spec",
  "verification-status": "verified",
  "dependencies": [
    "probe:...constants.ED25519_BASEPOINT_POINT",
    "probe:curve25519_dalek.edwards.EdwardsPoint"
  ]
}
```

---

## Three kinds of projects, three questions

We work with three kinds of projects, and each asks a different question.

1. **Functional verification.** Does this function satisfy its spec?
2. **Mathlib-style formalization.** Is this theorem proved?
3. **Security-protocol formalization in Lean.** Is this construction secure?

The question is not cosmetic. It decides what an atom even means and what a good answer looks like.

---

## One framework for all three can mislead

It is tempting to fold the three into a single framework and grade every project the same way.

We can always do this. Everything reduces to zeros and ones. But in reducing to a common denominator we throw away the meaning that made each question worth asking. "Is this construction secure?" is not "is this theorem proved?" with different labels.

Forcing one framework on every project is the hammer that turns every project into a nail.

**Suggestion: a separate view for each kind of project.** It keeps the meaning, and it is less work than bending three questions into one shape.

---

## Probes provide data; VeriLib interprets it

The probes have one job: provide factual data about the code, as JSON.

VeriLib takes that data and decides how to present it, including colors and statistics.

Colors and stats are a matter of taste. They depend on what a given user wants to see and highlight. That makes them orthogonal to the probes, which only report facts about the code and take no position on how those facts should look.

---

## Next: colors

With that separation in mind, we can talk about colors as a VeriLib concern, on top of the factual data the probes provide.
