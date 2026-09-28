# Categorical Framework for Probe Tools

Below, a few notes following the input from Shaowei. Credit goes to Shaowei. Errors are mine. 

---

The categorical structure underlying the probe tools architecture, drawing on two frameworks:

- **DOTS** (Double Operadic Theory of Systems) by Libkind & Myers ([arXiv 2505.18329](https://arxiv.org/abs/2505.18329))
- **SSProve** (State-Separating Proofs) by Spitters et al. ([ePrint 2021/397](https://eprint.iacr.org/2021/397))

Both frameworks describe composable units with typed interfaces governed by algebraic laws. The probe tools instantiate this pattern: `probe merge` is the universal composition operator and each probe tool is a doctrine. The DOTS/SSProve tables below are architectural **analogies**, not implemented mechanisms: the implemented algebra is annotation-preserving composition — cross-language mappings attach correspondence records carried alongside the composition ([ADR-006](../kb/decisions/006-correspondence-records.md)); cross-language *linking* (resolution) is a derived consumer view and a deferred capability, not an achieved one.

## Core Correspondence

### Mapping to DOTS

| DOTS Concept | Probe Architecture |
|---|---|
| **Interface** | Atom schema — the typed signature (code-name, kind, language, dependencies) |
| **Interaction (loose morphism)** | A probe extract output — a dependency graph over atoms with a typed boundary |
| **Tight morphism (interface map)** | Cross-language mapping — relates code-names across languages (e.g. Rust ↔ Lean) |
| **Composition** | `probe merge` — the single operator that composes any atom maps |
| **Composition with functor** | `probe merge --mappings` — the tight morphism is *structural data carried alongside* the composition (correspondence records on atoms), not edges inside the interaction: only the interface participates in composition |
| **Parallel placement** | Merging disjoint atom maps (no overlapping keys, no mappings needed) |
| **Doctrine** | Each probe tool (probe-rust, probe-verus, probe-lean) — defines what atoms look like in its language |

In DOTS, there is one composition operator. When interfaces don't match natively, a tight morphism (interface map) mediates. `--mappings` is exactly this — a tight morphism between the Rust and Lean interface categories.

### Mapping to SSProve

| SSProve Concept | Probe Architecture |
|---|---|
| **Package** | A probe extract output (atoms + dependency edges) |
| **Export interface** | The atoms an extract exposes (its code-names) |
| **Import interface** | The dependencies those atoms reference (potentially unresolved stubs) |
| **Sequential linking (`link`)** | Stub replacement in `probe merge` — an incoming real atom resolves a stub in the base |
| **Parallel composition (`par`)** | Adding new atoms from incoming maps that don't overlap with the base |
| **Translation / simulation** | `--mappings` — maps code-names across language boundaries; resolution is a derived view over `maps-to`/`mapped-from` records, not performed at merge time |
| **State separation** | Merge never interprets source-code *bodies*. Structural metadata is part of the composition interface: `Atom::is_stub` inspects `code-path` and the `code-text` line fields to decide which atom wins, and merge/enrichment interpret verification status, status origin, trust, and correspondence metadata |
| **Identity package** | An empty atom map — merging with it changes nothing |

In SSProve, "interactions" are "packages" — composable units with import/export interfaces and hidden internal state, governed by algebraic composition laws.

## Algebraic Laws

Merge factors into two operations (normative statement:
[P4](../kb/engineering/properties.md#p4-merge-associativity-on-the-carrier)/[P5](../kb/engineering/properties.md#p5-merge-identity-exact-on-the-carrier)):

- `μ(A, B)` — plain merge: normalize each input (per input, before conflict
  resolution), union with the category's conflict rule, then enrichment
  recomputation.
- `F_M(A)` — attachment: for fixed mappings M, attach correspondence records
  to the atoms of A.

`probe merge` without `--mappings` computes `μ`; with `--mappings` it computes
`F_M ∘ μ`. The laws hold on the **carrier**: the set of normalized,
enrichment-consistent atom maps (a precondition established by
normalization + enrichment — `--skip-enrich` output is off-carrier; projected
artifacts are excluded from μ's domain entirely). Laws are stated modulo
envelope meta and over provenance as a deduplicated source inventory.

1. **Associativity**: `μ(μ(A, B), C) = μ(A, μ(B, C))` on the carrier —
   enrichment preserves graph structure, stub classification, and recoverable
   base statuses, so enriching an intermediate merge never changes which atom
   a later conflict selects.

2. **Identity**: `μ(A, ∅) = A` exactly on the carrier; a legacy input is first
   brought onto the carrier (`μ(A, ∅) = enrich(normalize(A))`). Identity does
   not hold for the mapped merge (`F_M(A) ≠ A` when A lacks records) — the law
   F_M satisfies is idempotence.

3. **Commutativity of parallel placement**: when A and B have disjoint keys,
   `μ(A, B) = μ(B, A)`. Stub-vs-real also resolves order-independently; the
   genuine order-dependence is real-vs-real (P6) and spec/proof overlap (P7),
   deliberately — argument order is the user's freshness knob.

4. **Mapping compatibility** (replaces "functoriality holds naturally"):
   `F_M(μ(A, B)) = μ(F_M(A), F_M(B))` and `F_M(F_M(A)) = F_M(A)`. Holds
   because attachment is unconditional, set-like, and key-local. Doing
   cross-language *resolution* at merge time would rewrite `dependencies`
   from co-present atoms and destroy this law — resolution stays a derived
   consumer view.

## Architecture

### `probe merge` — The Universal Composition Operator

`probe merge` is the single composition operator for authoritative, same-category probe outputs satisfying the evidence contract — i.e. the carrier (projections and pre-contract envelopes are rejected, [ADR-006](../kb/decisions/006-correspondence-records.md)). It handles:

- **Homogeneous merging** (rust+rust, lean+lean): no mappings needed, composition via stub replacement and key-based union.
- **Heterogeneous merging** (rust+lean): supply `--mappings` to attach correspondence records linking counterpart atoms across languages; consumers derive cross-language views from the records.

The `SchemaCategory` enum (atoms, specs, proofs) determines which composition law applies:
- **Atoms**: stub-replacement (first-wins for real-vs-real conflicts), followed by enrichment recomputation
- **Specs/Proofs**: last-wins semantics

The `SchemaCategory` + `--mappings` pair is the **doctrine signature**: it tells `probe merge` which composition law to apply and which correspondence annotation (F_M) to attach.

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

probe-aeneas is not a composition operator. It is a **functor factory** — it produces the tight morphism (cross-language mapping) that `probe merge` needs to compose across language boundaries.

- **`probe-aeneas translate`**: construct the functor via three-strategy matching:
  1. `rust-qualified-name` match (Charon-derived)
  2. `file+display-name` match
  3. `file+line-overlap` match
- **`probe-aeneas extract <project_path>`**: orchestrate the full pipeline — resolve Rust/Lean paths from `aeneas-config.yml`, run probe-rust and probe-lean, generate mappings, merge with correspondence-record attachment; consumers derive cross-language linkage from the records.

The domain knowledge about how Aeneas transpilation relates Rust names to Lean names lives in probe-aeneas. The generic composition law lives in probe merge. They don't mix.

## Properties

This categorical structure provides:

### 1. Consistency for New Probes

Adding a new probe (e.g. `probe-haskell`) requires:
- Implement the extractor (the doctrine): produce atoms in the standard envelope format.
- **Evidence obligation**: a producer whose verification evidence is not expressible in the emitted dependency graph must stamp `status-origin` on the affected statuses ([ADR-006](../kb/decisions/006-correspondence-records.md)) — otherwise hub enrichment over its output is unsound.
- `probe merge` handles same-language composition automatically.
- For cross-language support: build a mapping generator that produces `mappings.json`; `probe merge --mappings` attaches the correspondence, and consumers derive linkage from the records.

### 2. Closed Composition

Adding a new cross-language bridge (e.g. Rust↔Haskell) requires only a new mapping generator — no changes to `probe merge`. The composition operator is closed over *authoritative, same-category inputs satisfying the evidence contract* (the carrier): projections are views outside its domain, and pre-contract artifacts are rejected by the version gate.

### 3. Testable Compositionality

The algebraic laws (associativity, identity, commutativity for disjoint keys, mapping compatibility, F_M idempotence) are concrete properties expressible as tests over the carrier. The existing recursive merge test is essentially testing associativity. Law-based tests verify:
- Identity: `μ(A, ∅) == A` exactly on the carrier; up to normalization+enrichment on legacy inputs
- Commutativity: `μ(A, B) == μ(B, A)` for disjoint keys
- Compatibility: `F_M(μ(A, B)) == μ(F_M(A), F_M(B))` across groupings, and `F_M ∘ F_M = F_M`
- Enrichment of intermediates does not change the eventual selected base data

### 4. Provenance as Source Inventory

The `inputs` array in `MergedAtomEnvelope` is a deduplicated **source inventory**: it records *which* sources were composed — not how many times, in what grouping, or which conflict decisions were made. It is the categorical analogue of listing the packages linked into the final game, not a proof trace of the linking process.

## References

- Libkind, S. & Myers, D.J. (2025). *Towards a double operadic theory of systems*. [arXiv:2505.18329](https://arxiv.org/abs/2505.18329)
- Haselwarter, P. et al. (2021). *SSProve: A Foundational Framework for Modular Cryptographic Proofs in Coq*. [ePrint 2021/397](https://eprint.iacr.org/2021/397)
- Brzuska, C. et al. (2018). *State Separation for Code-Based Game-Playing Proofs*. [ePrint 2018/306](https://eprint.iacr.org/2018/306)
