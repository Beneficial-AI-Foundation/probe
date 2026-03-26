# Categorical Framework for Probe Tools

This document describes the categorical structure underlying the probe tools architecture, drawing on two frameworks:

- **DOTS** (Double Operadic Theory of Systems) by Libkind & Myers ([arXiv 2505.18329](https://arxiv.org/abs/2505.18329))
- **SSProve** (State-Separating Proofs) by Spitters et al. ([ePrint 2021/397](https://eprint.iacr.org/2021/397))

Both frameworks describe composable units with typed interfaces governed by algebraic laws. The probe tools instantiate this pattern: `probe merge` is the universal composition operator, each probe tool is a doctrine, and translation mappings are functors between language categories.

## Core Correspondence

### Mapping to DOTS

| DOTS Concept | Probe Architecture |
|---|---|
| **Interface** | Atom schema — the typed signature (code-name, kind, language, dependencies) |
| **Interaction (loose morphism)** | A probe extract output — a dependency graph over atoms with a typed boundary |
| **Tight morphism (interface map)** | Translation mapping — relates code-names across languages (e.g. Rust ↔ Lean) |
| **Composition** | `probe merge` — the single operator that composes any atom maps |
| **Composition with functor** | `probe merge --translations` — composition mediated by a cross-language functor |
| **Parallel placement** | Merging disjoint atom maps (no overlapping keys, no translations needed) |
| **Doctrine** | Each probe tool (probe-rust, probe-verus, probe-lean) — defines what atoms look like in its language |

In DOTS, there is one composition operator. When interfaces don't match natively, a tight morphism (interface map) mediates. `--translations` is exactly this — a tight morphism between the Rust and Lean interface categories.

### Mapping to SSProve

| SSProve Concept | Probe Architecture |
|---|---|
| **Package** | A probe extract output (atoms + dependency edges) |
| **Export interface** | The atoms an extract exposes (its code-names) |
| **Import interface** | The dependencies those atoms reference (potentially unresolved stubs) |
| **Sequential linking (`link`)** | Stub replacement in `probe merge` — an incoming real atom resolves a stub in the base |
| **Parallel composition (`par`)** | Adding new atoms from incoming maps that don't overlap with the base |
| **Translation / simulation** | `--translations` — maps code-names across language boundaries so linking can resolve cross-language dependencies |
| **State separation** | Each atom's internal code (`code-text`, `code-path`) is opaque to the merge — only the interface (code-name, dependencies, kind) participates in composition |
| **Identity package** | An empty atom map — merging with it changes nothing |

In SSProve, "interactions" are "packages" — composable units with import/export interfaces and hidden internal state, governed by algebraic composition laws.

## Algebraic Laws

`probe merge` satisfies (or should satisfy) the following laws:

1. **Associativity**: `merge(merge(A, B), C) = merge(A, merge(B, C))` — merging is independent of grouping. The recursive merge test (`test_recursive_merge_flattens_provenance`) exercises this.

2. **Identity**: `merge(A, ∅) = A` — merging with an empty atom map is a no-op.

3. **Commutativity of parallel placement**: when A and B have disjoint keys and no stubs to resolve, `merge(A, B) = merge(B, A)` (modulo provenance ordering).

4. **Translation functoriality**: if T is a translation, applying T then merging equals merging then applying T. Translations are applied after the base merge (in `merge_atom_maps`), which makes this hold naturally.

## Architecture

### `probe merge` — The Universal Composition Operator

`probe merge` is the single composition operator for all probe outputs. It handles:

- **Homogeneous merging** (rust+rust, lean+lean): no translations needed, composition via stub replacement and key-based union.
- **Heterogeneous merging** (rust+lean): supply `--translations` to mediate cross-language dependencies.

The `SchemaCategory` enum (atoms, specs, proofs) determines which composition law applies:
- **Atoms**: stub-replacement (first-wins for real-vs-real conflicts)
- **Specs/Proofs**: last-wins semantics

The `SchemaCategory` + `--translations` pair is the **doctrine signature**: it tells `probe merge` which composition law to apply and which functor to use for cross-language mediation.

### Each Probe Tool — A Doctrine

Each probe tool defines what atoms look like in its language:

- **probe-rust**: Rust atoms via rust-analyzer + SCIP. Kind is always `exec`, language is always `rust`.
- **probe-verus**: Verus atoms with specs and verification status. Kinds include `exec`, `proof`, `spec`.
- **probe-lean**: Lean 4 atoms with typed/term dependencies, sorry detection. Language is always `lean`.

A doctrine specifies:
- The **interface type** (what does an atom look like in this language?)
- The **internal structure** (language-specific extensions via the `extensions` field)
- The **extraction method** (how to produce atoms from source)

### `probe-aeneas` — A Functor Factory

probe-aeneas is not a composition operator. It is a **functor factory** — it produces the tight morphism (translation mapping) that `probe merge` needs to compose across language boundaries.

- **`probe-aeneas translate`**: construct the functor via three-strategy matching:
  1. `rust-qualified-name` match (Charon-derived)
  2. `file+display-name` match
  3. `file+line-overlap` match
- **`probe-aeneas extract <project_path>`**: orchestrate the full pipeline — resolve Rust/Lean paths from `aeneas-config.yml`, run probe-rust and probe-lean, generate translations, merge with cross-language edges.

The domain knowledge about how Aeneas transpilation relates Rust names to Lean names lives in probe-aeneas. The generic composition law lives in probe merge. They don't mix.

## Properties

This categorical structure provides:

### 1. Consistency for New Probes

Adding a new probe (e.g. `probe-haskell`) requires only:
- Implement the extractor (the doctrine): produce atoms in the standard envelope format.
- `probe merge` handles same-language composition automatically.
- For cross-language support: build a translation generator that produces `translations.json`, then `probe merge --translations` handles the rest.

### 2. Closed Composition

Adding a new cross-language bridge (e.g. Rust↔Haskell) requires only a new translation generator — no changes to `probe merge`. The composition operator is closed over the space of all probe outputs.

### 3. Testable Compositionality

The algebraic laws (associativity, identity, functoriality) are concrete properties expressible as tests. The existing recursive merge test is essentially testing associativity. Additional law-based tests could verify:
- Identity: `merge([atoms, empty]) == atoms`
- Commutativity: `merge([A, B]) == merge([B, A])` for disjoint keys
- Functoriality: translations commute with merge order

### 4. Provenance as Composition Trace

The `inputs` array in `MergedAtomEnvelope` records which packages were composed — the categorical analogue of a proof trace in SSProve, recording which packages were linked to produce the final game.

## References

- Libkind, S. & Myers, D.J. (2025). *Towards a double operadic theory of systems*. [arXiv:2505.18329](https://arxiv.org/abs/2505.18329)
- Haselwarter, P. et al. (2021). *SSProve: A Foundational Framework for Modular Cryptographic Proofs in Coq*. [ePrint 2021/397](https://eprint.iacr.org/2021/397)
- Brzuska, C. et al. (2018). *State Separation for Code-Based Game-Playing Proofs*. [ePrint 2018/306](https://eprint.iacr.org/2018/306)
