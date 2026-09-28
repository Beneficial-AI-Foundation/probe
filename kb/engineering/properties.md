---
title: Properties and Invariants
last-updated: 2026-09-28
status: draft
---

# Properties and Invariants

Correctness constraints that all probe tool implementations must preserve. Every change must be checked against these properties. If you cannot satisfy a property, stop and ask — do not silently weaken it.

## P1. Envelope completeness

Every probe output file MUST be wrapped in a valid [Schema 3.x envelope](schema.md#envelope). No bare JSON dictionaries as output from any tool's primary commands.

**Validation**: `schema-version` starts with `"3."`. All required envelope fields present and non-empty.

## P2. Atom identity via code-name

An [atom's](glossary.md#atom) [code-name](glossary.md#code-name) is its unique identity. Two atoms with the same code-name in the same file represent the same definition.

**Constraint**: Within a single output file, code-names MUST be unique (they are dictionary keys).

**Constraint**: Code-names are deterministic — running the same tool on the same commit produces the same code-names.

## P3. Stub detection is structural

An atom is a [stub](glossary.md#stub) if and only if ALL three conditions hold:
1. `code-path` is `""`
2. `code-text.lines-start` is `0`
3. `code-text.lines-end` is `0`

No other heuristic (e.g. checking `dependencies: []`) determines stub status. This is implemented in `probe/src/types.rs::Atom::is_stub()`.

## P4. Merge associativity (on the carrier)

Merge factors into two operations ([ADR-006](../decisions/006-correspondence-records.md)):

- **`μ(A, B)`** — plain merge: normalize **each input** ([P8](#p8-code-name-normalization), before conflict resolution — the ordering selects evidence), union with the category's conflict rule ([P6](#p6-atom-merge-is-first-wins-with-stub-replacement) atoms, [P7](#p7-specsproofs-merge-is-last-wins) specs/proofs), then enrichment recomputation ([P23](#p23-transitive-verification)) once after all inputs are combined.
- **`F_M(A)`** — attachment: for fixed mappings M, attach correspondence records ([P13](#p13-correspondence-records-attach-unconditionally), [P27](#p27-correspondence-records-are-unioned-and-inert)) to the atoms of A.

`probe merge` without `--mappings` computes `μ`; with `--mappings` it computes `F_M ∘ μ`.

**Carrier**: the laws hold on the set of normalized, enrichment-consistent atom maps. The carrier is a *precondition established by normalization + enrichment*, not an automatic property of tool outputs (P23 permits `--skip-enrich`, so even modern extractor output can be off-carrier); μ's own normalize+enrich brings any authoritative input onto it. Projected artifacts are excluded from μ's domain entirely — rejected, not normalized ([ADR-006](../decisions/006-correspondence-records.md)). Every merge output is a fixed point.

**Laws**:

1. **Associativity**: `μ(μ(A, B), C) = μ(A, μ(B, C))` on the carrier. The argument is not enrichment idempotence alone: enrichment preserves graph structure, stub classification, and the recoverable base statuses (it only rewrites `verified` ↔ `transitively-verified`), so enriching an intermediate merge never changes which atom a later conflict selects.
2. **Commutativity for disjoint keys**: `μ(A, B) = μ(B, A)` when keys are disjoint. Stub-vs-real also resolves order-independently (real wins either way); the genuine order-dependence is real-vs-real (P6) and spec/proof key overlap (P7) — deliberate: argument order is the user's freshness knob.
3. **Mapping compatibility** (replaces "functoriality holds naturally"): `F_M(μ(A, B)) = μ(F_M(A), F_M(B))` and `F_M(F_M(A)) = F_M(A)`. Holds because attachment is unconditional, set-like, and key-local (P13, P27).

Laws are stated modulo envelope meta (`timestamp`, `tool` — merge writes fresh values) and over provenance as a deduplicated source inventory ([P9](#p9-provenance-is-preserved)).

## P5. Merge identity (exact on the carrier)

`μ(A, ∅) = A` exactly on the carrier. A legacy (off-carrier) input is first brought onto the carrier: `μ(A, ∅) = enrich(normalize(A))`.

Identity does **not** hold for the mapped merge: `F_M(μ(A, ∅)) = F_M(A) ≠ A` when A lacks records — correct behavior, not a bug; the law F_M satisfies is idempotence ([P4](#p4-merge-associativity-on-the-carrier), law 3).

## P6. Atom merge is first-wins with stub replacement

When merging atoms:
- [Stubs](glossary.md#stub) in base are replaced by real atoms from incoming files (stub replacement)
- Real-vs-real conflicts: base version is kept (first-wins), warning emitted
- New atoms (not in base) are added

This means input order matters for real-vs-real conflicts. The first file is the base.

## P7. Specs/proofs merge is last-wins

When merging specs or proofs:
- Same code-name in multiple inputs: last one wins
- No stub concept exists for specs/proofs

This is appropriate because re-running `specify` or `verify` should override stale results.

## P8. Code-name normalization

Normalization strips trailing `.` characters (a legacy verus-analyzer artifact). In merge it runs **per input, before conflict resolution** ([P4](#p4-merge-associativity-on-the-carrier) — the ordering selects evidence: it decides which atom wins a post-normalization collision before evidence from other inputs is considered). Unary recomputation boundaries (`probe enrich`, `probe project`) apply the same normalization to their input before enrichment, seed matching, or trimming.

Normalization is applied to:
- Dictionary keys
- All entries in `dependencies` arrays
- All code-name-bearing extension arrays: `requires-dependencies`, `ensures-dependencies`, `body-dependencies`, `type-dependencies`, `term-dependencies`, and `code-name` fields in `dependencies-with-locations`
- `maps-to` / `mapped-from` record targets ([P27](#p27-correspondence-records-are-unioned-and-inert))
- Mapping-file endpoints (`from`/`to`), before any lookup — atom keys and mapping endpoints are normalized by the same rule on both sides of every lookup

## P9. Provenance is preserved

Every merged output records the provenance of its inputs in the `inputs` array. When a previously merged file is used as input, its `inputs` entries are flattened into the new output — provenance is never lost across recursive merges.

Provenance is a **deduplicated source inventory**: the `inputs` array records *which* sources were composed, not how many times or in what order — identical entries are deduplicated, which gives envelope-level merge idempotence up to metadata ([P4](#p4-merge-associativity-on-the-carrier)). It is not a record of the composition process or its conflict decisions.

**Composed detection is structural**: an envelope with an `inputs` array is composed, one with a `source` object is single-tool — never keyed on a schema-string prefix. Producers may emit composed envelopes under their own schema strings (e.g. `probe-aeneas/extract` with two `inputs` and no `source`); their inventories must survive loading and re-merge.

Scope note: the inventory guarantee currently extends to inventory-preserving producers; probe-leanblueprint emits a single-`source` envelope over a composed base until its [ADR-006](../decisions/006-correspondence-records.md) rollout lands.

## P10. Extensions are preserved through merge

Tool-specific extension fields (any JSON key/value not part of the core atom schema) MUST be preserved through merge operations. The `extensions` BTreeMap in the Rust `Atom` struct captures these via `#[serde(flatten)]`.

## P13. Correspondence records attach unconditionally

When applying mappings during merge (`--mappings`), correspondence records ([`maps-to`/`mapped-from`](schema.md#correspondence-records-maps-to-mapped-from)) are attached to atoms; `dependencies` is never modified ([ADR-006](../decisions/006-correspondence-records.md), superseding the edge-injection semantics this property previously described):

- **Unconditional**: an atom gets its record whether or not the record's target is present in this invocation's key set. A dangling target is warned about, never skipped — it is no worse than a dangling dependency (already tolerated: warned, treated as trusted). An existence check would make attachment depend on the surrounding key set and break the mapping-compatibility law ([P4](#p4-merge-associativity-on-the-carrier), law 3).
- **Key-local**: attachment depends only on the atom's own code-name.
- **Set-like**: record identity is the `(target, confidence, method)` triple; re-application is a no-op ([P27](#p27-correspondence-records-are-unioned-and-inert)).
- Records attach only to atoms present in the merged map (`maps-to` on the `from` atom, `mapped-from` on the `to` atom); 1-to-many mappings yield one record per target.
- Endpoints are normalized before lookup ([P8](#p8-code-name-normalization)).

Cross-language *resolution* is a derived consumer view over the records, never computed at merge time.

## P14. Deterministic output

All tools produce deterministic output for the same input:
- Atom dictionaries use `BTreeMap` (sorted keys)
- File traversal is sorted
- Merge output keys are sorted
- Spec extraction uses `BTreeMap` throughout
- Array fields (e.g. `dependencies-with-locations`) MUST be sorted in a stable, deterministic order — typically `(line, code-name)`. Iterating over `HashSet` or `HashMap` and serializing the result without sorting violates this property.

**Known fix**: probe-verus fixed non-deterministic `dependencies-with-locations` ordering in v5.2.0 by sorting by `(line, code-name)` before serialization. probe-rust has the same issue (tracked).

## P15. Dependency completeness

For probe-verus `extract` output, the `dependencies` field is the union of three categorized subsets:
- `requires-dependencies` (from `requires` clauses)
- `ensures-dependencies` (from `ensures` clauses)
- `body-dependencies` (from function body)

Similarly for probe-lean: `dependencies` = deduplicated union of `type-dependencies` + `term-dependencies`.

The `dependencies` field MUST always equal the union of its categorized subsets. It is never a superset or subset.

**Scope**: the decomposition equality applies to atoms that carry categorized subsets — plain Rust atoms (probe-rust) carry none, and the equality is vacuous there.

**Preservation**: every transformation (merge, project) must preserve the decomposition where present. In particular, when `probe project` trims `dependencies` to the projected set, it trims the categorized extension arrays with the same filter.

## P16. Verification status mapping

The `verification-status` field has five possible values: `"transitively-verified"`, `"verified"`, `"failed"`, `"unverified"`, `"trusted"` (or absent for atoms with no verification status — both out-of-scope atoms (`untracked: true`) and the in-scope backlog (`untracked: false`, spec-less)).

For probe-verus, Verus verification output maps to `verification-status` as:

| Verus status | `verification-status` |
|---|---|
| `success` | `"verified"` (relabelled `"transitively-verified"` by enrichment recomputation when clean — [P23](#p23-transitive-verification)) |
| `failure` | `"failed"` |
| `sorries` | `"unverified"` |
| `warning` | `"unverified"` |

This mapping applies only to **spec-bearing** functions. A spec-less in-scope function receives **no** `verification-status`. Note that Verus `spec` functions are structurally excluded from verification reporting (they have no proof obligations of their own), so spec atoms are always status-less.

For probe-lean, statuses are **kernel-based**, not warning-based: trust and taint are computed over the kernel environment (a taint walk over compiler-elaborated terms), not from build warnings or file paths alone. Classification detail is normative in probe-lean's own [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md); the hub's evidence contract is:

- `"trusted"` marks the trust base: axioms (`trusted-reason: "axiom"`), declarations tagged `@[externally_verified]` (`"externally_verified"`), and **non-proof** declarations in `*External` modules (`"external"`) — External-module trust excludes proofs: a theorem in an External file is `unverified`, not trusted.
- `"verified"` = locally proved but kernel-reachable taint exists; `"transitively-verified"` = clean per the kernel walk. The kernel walk can see taint the emitted dependency graph cannot express (e.g. through compiler-generated proof auxiliaries that are not atoms). Whenever the taint pass runs (including under `--skip-enrich`), probe-lean stamps `status-origin: "kernel-taint"` on atoms whose `"verified"` reflects graph-inexpressible taint; such atoms are blocker seeds ([P23](#p23-transitive-verification)). An **unmarked** `"verified"` atom is safely promotable by hub enrichment.

**Enrichment** ([P23](#p23-transitive-verification)): the enrichment step **recomputes** the `"verified"`/`"transitively-verified"` split as a function of the final graph and base statuses — it sets labels fresh, never merely upgrades. It runs automatically as the last step of `probe-verus extract` and `probe-aeneas extract` (or via `probe enrich`), and `probe merge` re-enriches its output.

## P17. Schema category consistency

All inputs to a single `probe merge` invocation MUST belong to the same [schema category](schema.md#schema-categories) (atoms, specs, or proofs). Mixing categories is an error.

## P19. No cross-repo path dependencies

All `Cargo.toml` dependencies referencing crates in a **different** git repository MUST use `git = "https://..."` URLs, never `path = "../..."`.

- **Within the same repo/workspace**: `path = "..."` is correct (e.g., `probe-extract-check` depending on `probe` via `path = ".."`)
- **Across repos**: Use `git = "https://github.com/Beneficial-AI-Foundation/<repo>"`. Cargo auto-discovers workspace member crates within the git repo.
- **Local development**: Use `[patch]` sections or `.cargo/config.toml` overrides (not committed) to redirect git deps to local paths

**Why**: Path deps pointing outside the repo root break `cargo install --git`, CI builds, and any standalone consumer. Cargo validates all path deps during manifest parsing, even for dev-dependencies it won't build.

**Validation**: No `Cargo.toml` in any probe-* repo contains a `path = "..."` dependency where the resolved path exits the repository root.

## P21. Cross-tool RQN alignment

When probe-rust and probe-verus process the same Rust source project, `rust-qualified-name` values must be identical for all functions that both tools discover:

- Both tools use `derive_rust_qualified_name(code_path, display_name)` with the same algorithm
- `display_name` for trait impl methods must be `SelfType::method` (not `TraitName::method`)
- `code_path` must use `crate-name/src/...` format (workspace prefix included)

When both tools run with `--with-public-api`, `is-public-api` values must agree for every function with a matching RQN.

**Implemented in**: `probe-verus/src/lib.rs` (`build_call_graph` re-enriches display names for single-hash trait impl symbols using the self-type from the SCIP pre-pass), `probe-verus/src/public_api.rs` (RQN-based matching against `cargo public-api`).

## P22. Cross-tool trust-reason vocabulary

Each probe tool emits tool-specific `trusted-reason` values that reflect the source language's constructs. Cross-tool consumers (dashboards, summary scripts) must normalize these to a common vocabulary:

| Canonical category | probe-verus value | probe-lean value | Meaning |
|--------------------|-------------------|------------------|---------|
| `axiom` | `"admit"` | `"axiom"` | Property assumed without proof |
| `external` | `"external-body"` | `"external"` | Implementation trusted without checking |
| `assumed spec` | `"assume-specification"` | — | External function whose declared spec is not proved |
| `attested` | — | `"externally_verified"` | Proof discharged outside Lean; a human vouches via the `@[externally_verified]` attribute |

probe-leanblueprint extends the vocabulary additively on its synthetic node atoms (`"declared"`, `"upstream-proved"` — claims, not attestations; see the code-atom scoping in [P23](#p23-transitive-verification) and [ADR-006](../decisions/006-correspondence-records.md)).

Tools must NOT rename their `trusted-reason` values to match another tool — the values are part of each tool's public contract. Normalization happens in consumers (e.g., `scripts/summarize_extract.py`).

**Implemented in**: `probe/scripts/summarize_extract.py` (`TRUST_LABELS` mapping and `TOOL_CONFIG` per-tool configuration).

## P23. Transitive verification

One executable, path-based definition ([ADR-006](../decisions/006-correspondence-records.md); this *replaces* the earlier "every transitively reachable dependency is verified or trusted" wording):

> A locally verified atom is `"transitively-verified"` iff no **seed** — an explicit `"failed"`/`"unverified"` atom, or any atom carrying `status-origin` — is reachable from it along a dependency path that does not pass through a **trusted boundary** (a zero-length path counts: a seed is never promoted). A `"trusted"` atom is a boundary only when it carries no `status-origin`: `trusted_boundary(atom) = status == "trusted" AND status-origin absent`. Seeds keep their own base status, so blocking never downgrades below `"verified"`.

**Enrichment recomputes rather than upgrades.** One reverse BFS from the one seed set; then, for every atom whose status is `"verified"` or `"transitively-verified"`, the label is set fresh: reaches a seed along a non-trusted path, or is itself a seed → `"verified"`; otherwise → `"transitively-verified"`. A contaminated atom arriving with a `transitively-verified` label is downgraded; a `status-origin`-bearing atom arriving as `transitively-verified` is rewritten to `verified` **unconditionally**, not only when contaminated. Recomputation makes the result a function of the final graph and base statuses only — the property that makes merge re-enrichment and the merge algebra ([P4](#p4-merge-associativity-on-the-carrier)) sound.

Key rules:

- **Missing-status atoms are transparent by construction** — contamination flows through them; they are never labelled themselves. This covers out-of-scope atoms (`untracked: true`, [P25](#p25-atoms-not-in-the-verification-build-are-out-of-scope)) and the in-scope backlog (spec-less, `untracked: false`).
- **`trusted` is a trust boundary — whole-atom trust**: trusting an atom trusts its entire dependency closure; what is behind a trusted atom is irrelevant to its consumers. Enrichment traverses the unified `dependencies` set (the body/spec/type split is not available on plain atoms). This is a deliberate semantic decision recorded with the producer-audit findings in [ADR-006](../decisions/006-correspondence-records.md); the body-trust/closure-trust split is the documented fallback if a producer case emerges that markers cannot express.
- **`status-origin`-bearing atoms are blocker seeds, never boundaries** ([ADR-006](../decisions/006-correspondence-records.md)): never promoted, unconditionally demoted from an imported `transitively-verified`, and blocking promotion of every atom that reaches them along a non-trusted path. On a locally verified atom, blocking and contamination are the same operation (the label is set to `"verified"`). A copied `trusted` + `status-origin` seeds like any other marked atom — copied trust must not shield callers.
- **Correspondence records never contaminate** — `maps-to`/`mapped-from` ([P27](#p27-correspondence-records-are-unioned-and-inert)) are not dependencies and do not participate in the BFS.
- **Missing deps are treated as trusted** — dependencies not present in the atom map (e.g., external stdlib functions) do not block transitive status. A warning is logged for each.
- **Non-candidates are untouched** — `"failed"`/`"unverified"`/`"trusted"` base statuses are never rewritten.
- **Deterministic** ([P14](#p14-deterministic-output)) and **idempotent** — enrichment output is a fixed point.

**Merge re-enriches.** Plain merge alone can invalidate labels (stub resolution replaces a transparent status-less stub with a real `unverified` atom), so the shared merge implementation runs enrichment recomputation on the atoms category once after all inputs are combined; every public entry point inherits it. A raw staging primitive for multi-step pipelines defers recomputation but still validates input authority; its output is documented as carrying potentially stale derived statuses.

**Authority boundary.** Merge and enrich reject projected inputs (both formats) and pre-contract envelopes (per-producer version gate) — see [schema.md § Authority validation](schema.md#authority-validation-and-re-enrichment) and [ADR-006](../decisions/006-correspondence-records.md). The carrier ([P4](#p4-merge-associativity-on-the-carrier)) is a precondition: `--skip-enrich` outputs are off-carrier until re-enriched.

**Scope caveats**:

- Recomputed labels assert consistency with the *selected merged graph and the retained base statuses*. They do not re-validate proofs — a `verified` status extracted against one implementation of a dependency is carried over unchanged when merge selects another.
- Labels inside a projection describe the **original** graph: `probe project` recomputes enrichment on the full authoritative input before trimming, never from the trimmed view; already-projected inputs keep their labels untouched.
- The assurance contract — including the trusted-boundary reading — is stated over **code atoms**: synthetic `language: "blueprint"` statuses are presentation aggregates, never trust-boundary assertions ([ADR-006](../decisions/006-correspondence-records.md)).

**Integrated into extractors**: probe-verus and probe-aeneas call `enrich_verification_status` as the final step of their `extract` command (skippable via `--skip-enrich`). The `probe enrich` CLI command remains available for re-processing or standalone use.

**Implemented in**: `probe/src/commands/propagate.rs`

## P24. A status-bearing atom is in analysis scope

If an atom carries a `verification-status`, it is in verification scope: `has-verification-status ⟹ ¬untracked`. **`untracked: true` means out of verification scope** — the atom is not compiled/checked by Verus in this build (see [P25](#p25-atoms-not-in-the-verification-build-are-out-of-scope)) — *not* "unspecified backlog". Verus only assigns a status to a function it actually processes, so a status implies in-scope.

Statuses and what each requires:

- `verified` / `transitively-verified` — proved against a spec. **Never** appears without a spec: verification is *against a spec*, and a spec-less exec function has none. (Verus discharges only body-safety obligations — no overflow, in-bounds indexing, callee `requires` — against a defaulted `ensures true`; that is a vacuous claim, not a `verified` status.)
- `unverified` / `failed` — spec-bearing, not yet proved (sorries/warnings) or errored.
- `trusted` — a trusted axiom: `#[verifier::external_body]` or `admit()` in Verus. Axioms in Lean. In scope.

Scope, spec, and status align as:

| atom | `untracked` | `verification-status` |
|---|---|---|
| specified + proved | false | `verified` / `transitively-verified` |
| specified, not proved | false | `unverified` / `failed` |
| `#[verifier::external_body]` / `admit()` | false | `trusted` |
| **backlog** — compiled, non-external, unspecified | false | *(none)* |
| out of scope — Verus: cfg-inactive / `#[verifier::external]` / external-crate stub / bodiless declaration / non-library target; Aeneas: cfg-inactive / unmounted / bodiless declaration / non-library target / `@[out_of_scope]` translation / config out-of-scope | true | *(none)* |

The **backlog** a Verus project still owes specs for is exactly the in-scope/tracked, compiled, non-external, spec-less functions — `untracked: false`, no status.

- **probe-verus** — `untracked` is derived from scope (P25); a status is attached only to in-scope atoms, so `has-verification-status ⟹ ¬untracked` holds by construction.

**Why it matters**: consumers must not read `untracked: true` as "unverified work to do" — it marks code deliberately outside the verification effort. The backlog is `untracked: false` with no status.

## P25. Atoms not in the verification build are out of scope

For Verus projects, an atom is **out of verification scope** — `untracked: true`, no `verification-status` — exactly when Verus does not compile and check it in this build. Formally: `untracked: true ⟺ cfg-inactive ∨ #[verifier::external] ∨ external-crate stub ∨ bodiless-declaration ∨ non-library-target`:

1. **cfg-inactive** — the governing `#[cfg(...)]` predicate is false under the active configuration, so the item is not compiled.
2. **`#[verifier::external]`** — Verus ignores the item entirely (no body check, no spec).
3. **external-crate stub** — referenced from another crate, not part of this crate's source (empty `code-path`).
4. **bodiless declaration** — a function with no body (`has-body: false`), e.g. a trait-method signature. There is no implementation to verify; the implementations carry the proof.
5. **non-library target** — code outside the verified library/binary target: a build script (`build.rs`), integration tests (`tests/`), `examples/`, or `benches/`. Verus verifies the crate's `src/` tree, not these. (`#[cfg(test)]` code *inside* `src/` is covered by cfg-inactivity, not this case.)

`#[verifier::external_body]` is **not** out of scope: it declares a spec Verus trusts without checking the body, so it is `trusted` / `untracked: false` (P24). External-*ness* alone does not decide scope — whether the function carries a trusted spec does.

- The **active configuration** = the analyzer/verifier cfg (`verus_keep_ghost = true` for Verus) + the package's **resolved default features** (transitive closure of `[features] default` in `Cargo.toml`) + target defaults. **Inclusion gates do not make an atom out of scope**: `verus_keep_ghost` and active features (e.g. `alloc`, `precomputed-tables`, `zeroize`, `digest`) gate code that *is* compiled and *must* be verified.
- Only **item-gating** `#[cfg(...)]` counts. `#[cfg_attr(..., doc = …)]`, `cfg_attr(..., derive(…))`, `cfg_attr(..., allow(…))` conditionally add an attribute but still compile the item, so they are not scope gates.
- **Conservative**: if a predicate references a flag/feature the tool cannot resolve, the atom is kept in scope (backlog) rather than marked untracked. The tool MUST NEVER silently drop a real backlog item by guessing a predicate is false.

**Why it matters**: cfg-gatedness alone is *not* a scope signal — many cfg-gated `exec` functions are in scope and verified (compiled behind active gates like `verus_keep_ghost` and default features). Scope is decided by whether the predicate holds in the verification build, not by the mere presence of a gate. Marking out-of-build code (inactive features, non-selected backends, `not(verus_keep_ghost)` fallbacks, `#[cfg(test)]`) `untracked: true` keeps it out of the backlog, which is reserved for in-scope, compiled, unspecified functions.

For Aeneas projects, a Rust function is **out of verification scope** — `untracked: true`, no `verification-status` — exactly when it is not compiled into the verified library in the Aeneas build, has no body to translate, its Lean translation is explicitly annotated out of scope, or it is a function Aeneas structurally cannot translate that the project has curated out. Formally: `untracked: true ⟺ cfg-inactive ∨ unmounted ∨ bodiless-declaration ∨ non-library-target ∨ translation carries @[out_of_scope] ∨ config out-of-scope`:

1. **cfg-inactive** — the function's combined item-gating `#[cfg(...)]` predicate (own gate, enclosing `impl`/`mod`/`trait` gates, and the gates on the parent-file `mod` declaration chain, emitted by probe-rust as the `cfg` field) is false under the Aeneas build configuration, so the item is not compiled and cannot be translated or verified. probe-rust also emits the mod-chain component alone as `file-cfg`; that field never classifies on its own, it only refines the reported reason from `cfg-inactive` to `file-cfg-inactive`.
2. **unmounted** — no `mod` chain from the package's library or binary target entries reaches the function's file, so rustc compiles it into no lib or bin build (probe-rust's `is-unmounted`). Configuration-independent, so it applies even when the feature set cannot be resolved. probe-rust emits it only from a provably complete module-tree walk, so lib/bin-compiled code is never flagged.
3. **bodiless declaration** — a function with no body: there is no implementation here to verify. Rust has exactly two forms, and probe-rust emits a distinct fact for each, so their disjunction is this clause:
   - **foreign declaration** (`is-foreign`) — declared inside an `extern { … }` block. The implementation lives outside Rust, so nothing in the atom graph will ever discharge the obligation. A function with a non-Rust ABI but a real body (`pub extern "C" fn f() { … }`) is *not* foreign and stays in scope.
   - **trait signature** (`trait-required`) — a trait method declared without a default body. The proof obligations live on the `impl`s, which are tracked as their own atoms. Trait methods *with* a default body are ordinary code and stay in scope. Where Aeneas translates a trait *declaration* as an interface record, that atom carries a status and P24 keeps it tracked, so this fires only on signatures with no matched translation.

   This is the same clause the Verus rule above states as `has-body: false`; the two vocabularies describe one property, approached from the producer that reports it.
4. **non-library target** — code outside the verified library/binary target: a build script (`build.rs`), integration tests (`tests/`), `examples/`, or `benches/`. Aeneas translates the crate's library tree, not these separate compilation targets. Detected on `code-path` components: a path with no `src` component whose components include `build.rs`/`tests`/`examples`/`benches` (the `src` guard keeps in-`src` modules merely named `tests` in scope). This mirrors the Verus non-library-target case above.
5. **`@[out_of_scope]`** — the generated Lean translation carries an out-of-scope attribute, declaring "this translation will not be verified". This attribute is the explicit opt-out for functions that *are* translated.
6. **config out-of-scope** — a curated glob list (`out-of-scope` in the project's `.verilib/aeneas.json`) matched against the Rust atom's `rust-qualified-name` / `display-name`. This is the opt-out for functions Aeneas *structurally does not translate* — e.g. `Debug`/`Display` `fmt`, `Zeroize` — which therefore never appear in `functions.json` and have no Lean def to carry `@[out_of_scope]`. It is a manual, reviewable editorial decision (like `is-hidden`/`is-ignored`), not an automatic heuristic: it must never be used to bulk-exclude genuine spec backlog.

Causes (1) and (2) are about whether rustc compiles the item at all; (3) is about the declaration itself; (4)–(6) are targeting and editorial policy. probe-aeneas reports the applicable cause on each out-of-scope atom in `untracked-reason`, ordered most-intrinsic-first, and its normative field semantics live in that repo's [docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md).

**Every extracted (compiled) Rust function is tracked backlog by default** (`untracked: false`, no `verification-status`), whether or not Aeneas produced a Lean translation for it. Absence from `functions.json` alone does **not** imply out-of-scope: a compiled function that Aeneas has not yet translated is unverified backlog, not out of scope. `functions.json` is the translation-matching bridge (which Lean def a Rust function maps to), not the scope oracle.

- The **active configuration** for the Aeneas build = the package's **resolved default features** (transitive closure of `[features] default` in `Cargo.toml`), overlaid by any `--features` / `--no-default-features` / `--all-features` in the project's `charon.cargo_args`. cfg evaluation mirrors the Verus rules above: only item-gating `#[cfg(...)]` counts (not cosmetic `#[cfg_attr(...)]`), and evaluation is **conservative** — a predicate referencing a flag/feature the tool cannot resolve keeps the atom in scope (backlog), never silently dropping a real backlog item.
- As with Verus, a status-bearing atom is never untracked (P24): every reclassification above applies only to atoms that would otherwise be backlog.
- The **configuration-independent** causes (2) and (3) are judged from source structure alone, so they apply even when the feature set cannot be resolved and cfg classification is skipped.

## P27. Correspondence records are unioned and inert

`maps-to`/`mapped-from` records ([schema.md § Correspondence records](schema.md#correspondence-records-maps-to-mapped-from)):

- **Union through every equal-key resolution**: on stub replacement, real-vs-real first-wins, stub-vs-stub, and post-normalization collisions within a single input, the surviving atom's `maps-to`/`mapped-from` arrays are the set union of both sides'. This is a narrow, decidable carve-out from whole-atom conflict semantics ([P6](#p6-atom-merge-is-first-wins-with-stub-replacement), [P10](#p10-extensions-are-preserved-through-merge)), specific to correspondence records — they cannot be re-derived without the mappings file. General extension union remains undecided.
- **Identity and order**: record identity is `(target, confidence, method)` with absent `method` ordering as the empty string; arrays are sorted by the same triple ([P14](#p14-deterministic-output)); duplicates collapse. Same target with different confidence/method = distinct assertions, both kept.
- **Normalized**: record targets are covered by [P8](#p8-code-name-normalization).
- **Inert**: records never participate in the contamination/promotion BFS ([P23](#p23-transitive-verification)) and are never traversed by projection BFS.
- **Mirrors are best-effort**: the correspondence relation is the union over both fields; consumers index both and derive reverse lookups. A missing mirror (target absent at attachment time) loses no information; regeneration restores it. Mapping corrections and withdrawals take effect only by regenerating from extracts + the corrected mappings file — attachment never removes a record.

## Single-probe invariants (owned by each probe's repo)

Per [ADR-005](../decisions/005-doc-ownership-boundary.md), invariants specific to
one probe live in that probe's own repo, not in this shared file:

- Mapping generation is 1-to-1 and strategy-priority-ordered → **probe-aeneas**
  ([docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md),
  [docs/USAGE.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/USAGE.md)).
  The hub's merge accepts 1-to-many mappings ([P13](#p13-correspondence-records-attach-unconditionally)).
- `language` is derived from `kind`, not lexical scope → **probe-verus**
  ([docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#language-assignment)).
  The resulting `kind → language` value convention is recorded in the shared
  [schema.md#language-assignment-for-verus-atoms](schema.md#language-assignment-for-verus-atoms).
- Lean `specified` is derived from non-empty `specs`, not stored → **probe-lean**
  ([docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md)).
- Blueprint status is additive; machine `verification-status` stays authoritative
  → **probe-leanblueprint**
  ([docs/SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/SCHEMA.md)).
  Its consumer-side obligation — preserve the additive fields through merge — is
  the hub contract [P10](#p10-extensions-are-preserved-through-merge).
