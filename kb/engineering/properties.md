---
title: Properties and Invariants
last-updated: 2026-07-21
status: draft
---

# Properties and Invariants

Correctness constraints that all probe tool implementations must preserve. Every change must be checked against these properties. If you cannot satisfy a property, stop and ask — do not silently weaken it.

## P1. Envelope completeness

Every probe output file MUST be wrapped in a valid [Schema 2.0 envelope](schema.md#envelope). No bare JSON dictionaries as output from any tool's primary commands.

**Validation**: `schema-version` starts with `"2."`. All required envelope fields present and non-empty.

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

## P4. Merge associativity

`merge(merge(A, B), C) = merge(A, merge(B, C))`

Merging is independent of grouping. This enables recursive merging of previously merged files. Provenance flattening makes this transparent.

## P5. Merge identity

`merge(A, empty) = A`

Merging with an empty atom map is a no-op.

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

Before any merge operation, all code-name keys and dependency references are normalized: trailing `.` characters are stripped. This handles a legacy verus-analyzer artifact.

Normalization is applied to:
- Dictionary keys
- All entries in `dependencies` arrays
- `code-name` fields in `dependencies-with-locations` extension arrays

## P9. Provenance is preserved

Every merged output records the provenance of its inputs in the `inputs` array. When a previously merged file is used as input, its `inputs` entries are flattened into the new output — provenance is never lost across recursive merges.

## P10. Extensions are preserved through merge

Tool-specific extension fields (any JSON key/value not part of the core atom schema) MUST be preserved through merge operations. The `extensions` BTreeMap in the Rust `Atom` struct captures these via `#[serde(flatten)]`.

## P11. Mapping generation is 1-to-1 (probe-aeneas)

When generating cross-language mappings (probe-aeneas):
- Each Rust atom maps to at most one Lean atom
- Each Lean atom is claimed by at most one Rust atom
- Once matched, neither side can be matched again

Enforced by `matched_rust` and `matched_lean` HashSets in `probe-aeneas/src/translate.rs`.

Note: `probe merge` accepts 1-to-many mappings (a single `from` key can map to multiple `to` targets). The 1-to-1 constraint is specific to probe-aeneas's generation logic.

## P12. Mapping strategy priority

The matching strategies run in strict priority order:
0. **charon-`def_id`** (confidence: `exact`, method `charon-def-id`) — integer join on the charon `FunDeclId`: probe-rust's `charon-def-id` atom field equals Aeneas's `translation.json` `def_id`, binding to the family's primary (non-loop) Lean def. Only the manifest's `functions` array feeds the join: `globals`/`trait_impls` carry ids from charon's separate `GlobalDeclId`/`TraitImplId` spaces, which could otherwise collide with a `FunDeclId` integer. **Provenance-gated**: runs only when the atom's `charon-version` matches the manifest's `charon_version`, else it is skipped — mismatched ids point at different functions and would corrupt the mapping. Version equality is best-effort provenance, not proof of an identical run (same version + different cargo flags/sources can still diverge); a charon commit hash or LLBC digest would be the durable fix. No-op when probe-rust does not emit `charon-def-id`.
1. **Rust-qualified-name** (confidence: `exact` or `exact-disambiguated`) — Charon-derived names
2. **File + display-name** (confidence: `file-and-name`) — same source file + matching base name, unambiguous matches only
3. **File + line-overlap** (confidence: `file-and-lines`) — same source file + overlapping line ranges, best overlap wins

Higher-priority strategies run first and claim atoms. Lower-priority strategies only see unclaimed atoms.

## P13. Cross-language edges require existence

When applying mappings during merge:
- A mapped dependency is added only if the target code-name exists in the merged key set
- A mapped dependency is not added if it's already present in the atom's dependencies
- Both mapping directions (from→to and to→from) are checked
- When a single source maps to multiple targets (1-to-many), each target is checked independently

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

## P16. Verification status mapping

The `verification-status` field has five possible values: `"transitively-verified"`, `"verified"`, `"failed"`, `"unverified"`, `"trusted"` (or absent for untracked atoms).

For probe-verus, Verus verification output maps to `verification-status` as:

| Verus status | `verification-status` |
|---|---|
| `success` | `"verified"` (upgraded to `"transitively-verified"` after enrichment if all transitive deps are clean) |
| `failure` | `"failed"` |
| `sorries` | `"unverified"` |
| `warning` | `"unverified"` |

This mapping applies only to **spec-bearing** functions. A spec-less in-scope function receives **no** `verification-status`.

For probe-lean, verification status is determined by sorry detection and trust-base classification:

| Condition | `verification-status` |
|---|---|
| `kind == "axiom"` or `code-path` ends with `External.lean` | `"trusted"` |
| No sorry warnings | `"verified"` |
| Has sorry warnings | `"unverified"` |
| Build failure | `"failed"` |

**Precedence**: `"trusted"` overrides sorry-based status — an axiom or `*External.lean` declaration is always `"trusted"` regardless of build output.

**Enrichment** (P23): After initial status assignment, the enrichment step (reverse-BFS contamination) upgrades `"verified"` → `"transitively-verified"` for atoms whose entire transitive closure is verified or trusted. This is run automatically as the last step of `probe-verus extract` and `probe-aeneas extract` (or manually via `probe enrich`).

## P17. Schema category consistency

All inputs to a single `probe merge` invocation MUST belong to the same [schema category](schema.md#schema-categories) (atoms, specs, or proofs). Mixing categories is an error.

## P18. Lean `specified` is derived, not stored

Lean atoms do not have a `specified` field. Whether an atom has specs is inferred from `specs` being non-empty. This aligns with probe-verus v5.0.0 and avoids data redundancy.

## P19. No cross-repo path dependencies

All `Cargo.toml` dependencies referencing crates in a **different** git repository MUST use `git = "https://..."` URLs, never `path = "../..."`.

- **Within the same repo/workspace**: `path = "..."` is correct (e.g., `probe-extract-check` depending on `probe` via `path = ".."`)
- **Across repos**: Use `git = "https://github.com/Beneficial-AI-Foundation/<repo>"`. Cargo auto-discovers workspace member crates within the git repo.
- **Local development**: Use `[patch]` sections or `.cargo/config.toml` overrides (not committed) to redirect git deps to local paths

**Why**: Path deps pointing outside the repo root break `cargo install --git`, CI builds, and any standalone consumer. Cargo validates all path deps during manifest parsing, even for dev-dependencies it won't build.

**Validation**: No `Cargo.toml` in any probe-* repo contains a `path = "..."` dependency where the resolved path exits the repository root.

## P20. Language is derived from kind, not lexical scope

For probe-verus atoms, the `language` field is determined by the atom's `kind`, not by whether the function appears inside a `verus!{}` block:

- `kind == "exec"` → `language: "rust"` — exec functions are Rust code, even when annotated with Verus specifications
- `kind == "proof"` → `language: "verus"` — proof functions are Verus-only constructs, erased at compilation
- `kind == "spec"` → `language: "verus"` — spec functions are Verus-only constructs, erased at compilation

**Why**: Verus exec functions (e.g. `compress`, `decompress`, `mul`) are real Rust code that compiles to machine instructions. They happen to sit inside `verus!{}` blocks because that's where their specs live, but they are not "Verus constructs" — they are Rust functions with formal contracts. Tagging them `language: "verus"` would exclude them from any Rust-specific analysis (e.g. entrypoint detection, call graph filtering).

**Implemented in**: `probe-verus/src/lib.rs` (language assignment in `convert_to_atoms_with_lines_internal`).

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

Tools must NOT rename their `trusted-reason` values to match another tool — the values are part of each tool's public contract. Normalization happens in consumers (e.g., `scripts/summarize_extract.py`).

**Implemented in**: `probe/scripts/summarize_extract.py` (`TRUST_LABELS` mapping and `TOOL_CONFIG` per-tool configuration).

## P23. Transitive verification is computed by reverse-BFS contamination

The `probe enrich` command (and the `enrich_verification_status` library function) upgrades `verification-status` from `"verified"` to `"transitively-verified"` on atoms whose entire transitive dependency closure is verified or trusted:

- **`"transitively-verified"`**: the atom is verified AND every transitively reachable dependency is also verified or trusted.
- **`"verified"`** (after enrichment): the atom is verified but at least one transitively reachable dependency is not verified and not trusted (locally verified only).

The algorithm uses **reverse-BFS contamination**: build a reverse dependency index, seed contamination from atoms with explicit `"unverified"` or `"failed"` status, and propagate backwards through callers. This correctly handles cycles (all cycle members receive the same scope) without requiring SCC computation.

Key rules:
- **Only explicit `"unverified"` / `"failed"` contaminates** — atoms with missing `verification-status` are transparent and do not affect transitive scope. This covers both out-of-scope atoms (`is-disabled: true`, [P25](#p25-atoms-not-in-the-verification-build-are-out-of-scope)) and the in-scope backlog (spec-less, `is-disabled: false`) — e.g. plain Rust functions or Verus spec functions.
- **`trusted` does not block transitive** — trusted atoms are intentional axioms, not incomplete work.
- **Missing deps are treated as trusted** — dependencies not present in the atom map (e.g., external stdlib functions) do not block transitive status. A warning is logged for each.
- **Non-verified atoms are untouched** — only atoms with `verification-status: "verified"` are candidates for upgrade.
- **Deterministic** — uses `BTreeMap`/`BTreeSet` throughout (P14).
- **Idempotent** — running enrichment on already-enriched output produces the same result.

**Integrated into extractors**: probe-verus and probe-aeneas call `enrich_verification_status` as the final step of their `extract` command (skippable via `--skip-enrich`). The `probe enrich` CLI command remains available for re-processing or standalone use.

**Implemented in**: `probe/src/commands/propagate.rs`

## P24. A status-bearing atom is in analysis scope

If an atom carries a `verification-status`, it is in verification scope: `has-verification-status ⟹ ¬is-disabled`. **`is-disabled: true` means out of verification scope** — the atom is not compiled/checked by Verus in this build (see [P25](#p25-atoms-not-in-the-verification-build-are-out-of-scope)) — *not* "unspecified backlog". Verus only assigns a status to a function it actually processes, so a status implies in-scope.

Statuses and what each requires:

- `verified` / `transitively-verified` — proved against a spec. **Never** appears without a spec: verification is *against a spec*, and a spec-less exec function has none. (Verus discharges only body-safety obligations — no overflow, in-bounds indexing, callee `requires` — against a defaulted `ensures true`; that is a vacuous claim, not a `verified` status.)
- `unverified` / `failed` — spec-bearing, not yet proved (sorries/warnings) or errored.
- `trusted` — a trusted axiom: `#[verifier::external_body]` or `admit()` in Verus. Axioms in Lean. In scope.

Scope, spec, and status align as:

| atom | `is-disabled` | `verification-status` |
|---|---|---|
| specified + proved | false | `verified` / `transitively-verified` |
| specified, not proved | false | `unverified` / `failed` |
| `#[verifier::external_body]` / `admit()` | false | `trusted` |
| **backlog** — compiled, non-external, unspecified | false | *(none)* |
| out of scope — Verus: cfg-inactive / `#[verifier::external]` / external-crate stub / bodiless declaration / non-library target; Aeneas: untranslated / `@[out_of_scope]` translation | true | *(none)* |

The **backlog** a Verus project still owes specs for is exactly the in-scope/tracked, compiled, non-external, spec-less functions — `is-disabled: false`, no status.

- **probe-verus** — `is-disabled` is derived from scope (P25); a status is attached only to in-scope atoms, so `has-verification-status ⟹ ¬is-disabled` holds by construction.

**Why it matters**: consumers must not read `is-disabled: true` as "unverified work to do" — it marks code deliberately outside the verification effort. The backlog is `is-disabled: false` with no status.

## P25. Atoms not in the verification build are out of scope

For Verus projects, an atom is **out of verification scope** — `is-disabled: true`, no `verification-status` — exactly when Verus does not compile and check it in this build. Formally: `is-disabled: true ⟺ cfg-inactive ∨ #[verifier::external] ∨ external-crate stub ∨ bodiless-declaration ∨ non-library-target`:

1. **cfg-inactive** — the governing `#[cfg(...)]` predicate is false under the active configuration, so the item is not compiled.
2. **`#[verifier::external]`** — Verus ignores the item entirely (no body check, no spec).
3. **external-crate stub** — referenced from another crate, not part of this crate's source (empty `code-path`).
4. **bodiless declaration** — a function with no body (`has-body: false`), e.g. a trait-method signature. There is no implementation to verify; the implementations carry the proof.
5. **non-library target** — code outside the verified library/binary target: a build script (`build.rs`), integration tests (`tests/`), `examples/`, or `benches/`. Verus verifies the crate's `src/` tree, not these. (`#[cfg(test)]` code *inside* `src/` is covered by cfg-inactivity, not this case.)

`#[verifier::external_body]` is **not** out of scope: it declares a spec Verus trusts without checking the body, so it is `trusted` / `is-disabled: false` (P24). External-*ness* alone does not decide scope — whether the function carries a trusted spec does.

- The **active configuration** = the analyzer/verifier cfg (`verus_keep_ghost = true` for Verus) + the package's **resolved default features** (transitive closure of `[features] default` in `Cargo.toml`) + target defaults. **Inclusion gates do not make an atom out of scope**: `verus_keep_ghost` and active features (e.g. `alloc`, `precomputed-tables`, `zeroize`, `digest`) gate code that *is* compiled and *must* be verified.
- Only **item-gating** `#[cfg(...)]` counts. `#[cfg_attr(..., doc = …)]`, `cfg_attr(..., derive(…))`, `cfg_attr(..., allow(…))` conditionally add an attribute but still compile the item, so they are not scope gates.
- **Conservative**: if a predicate references a flag/feature the tool cannot resolve, the atom is kept in scope (backlog) rather than marked disabled. The tool MUST NEVER silently drop a real backlog item by guessing a predicate is false.

**Why it matters**: cfg-gatedness alone is *not* a scope signal — many cfg-gated `exec` functions are in scope and verified (compiled behind active gates like `verus_keep_ghost` and default features). Scope is decided by whether the predicate holds in the verification build, not by the mere presence of a gate. Marking out-of-build code (inactive features, non-selected backends, `not(verus_keep_ghost)` fallbacks, `#[cfg(test)]`) `is-disabled: true` keeps it out of the backlog, which is reserved for in-scope, compiled, unspecified functions.

For Aeneas projects, a Rust function is **out of verification scope** — `is-disabled: true`, no `verification-status` — exactly when it is not compiled into the verified library in the Aeneas build, its Lean translation is explicitly annotated out of scope, or it is a function Aeneas structurally cannot translate that the project has curated out. Formally: `is-disabled: true ⟺ cfg-inactive ∨ non-library-target ∨ translation carries @[out_of_scope] ∨ config out-of-scope`:

1. **cfg-inactive** — the function's combined item-gating `#[cfg(...)]` predicate (own gate plus enclosing `impl`/`mod`/`trait` gates, emitted by probe-rust as the `cfg` field) is false under the Aeneas build configuration, so the item is not compiled and cannot be translated or verified.
2. **non-library target** — code outside the verified library/binary target: a build script (`build.rs`), integration tests (`tests/`), `examples/`, or `benches/`. Aeneas translates the crate's library tree, not these separate compilation targets. Detected on `code-path` components: a path with no `src` component whose components include `build.rs`/`tests`/`examples`/`benches` (the `src` guard keeps in-`src` modules merely named `tests` in scope). This mirrors the Verus non-library-target case above.
3. **`@[out_of_scope]`** — the generated Lean translation carries an out-of-scope attribute, declaring "this translation will not be verified". This attribute is the explicit opt-out for functions that *are* translated.
4. **config out-of-scope** — a curated glob list (`out-of-scope` in the project's `.verilib/aeneas.json`) matched against the Rust atom's `rust-qualified-name` / `display-name`. This is the opt-out for functions Aeneas *structurally does not translate* — e.g. `Debug`/`Display` `fmt`, `Zeroize` — which therefore never appear in `functions.json` and have no Lean def to carry `@[out_of_scope]`. It is a manual, reviewable editorial decision (like `is-hidden`/`is-ignored`), not an automatic heuristic: it must never be used to bulk-exclude genuine spec backlog.

**Every extracted (compiled) Rust function is tracked backlog by default** (`is-disabled: false`, no `verification-status`), whether or not Aeneas produced a Lean translation for it. Absence from `functions.json` alone does **not** imply out-of-scope: a compiled function that Aeneas has not yet translated is unverified backlog, not out of scope. `functions.json` is the translation-matching bridge (which Lean def a Rust function maps to), not the scope oracle.

- The **active configuration** for the Aeneas build = the package's **resolved default features** (transitive closure of `[features] default` in `Cargo.toml`), overlaid by any `--features` / `--no-default-features` / `--all-features` in the project's `charon.cargo_args`. cfg evaluation mirrors the Verus rules above: only item-gating `#[cfg(...)]` counts (not cosmetic `#[cfg_attr(...)]`), and evaluation is **conservative** — a predicate referencing a flag/feature the tool cannot resolve keeps the atom in scope (backlog), never silently dropping a real backlog item.
- As with Verus, a status-bearing atom is never disabled (P24): the cfg/`@[out_of_scope]` reclassification applies only to atoms that would otherwise be backlog.

## P26. Blueprint status is additive; machine `verification-status` stays authoritative

For blueprint-enriched atoms (probe-leanblueprint), the blueprint's two-axis status is **additive metadata** and never overrides probe-lean's machine `verification-status`.

- The **statement axis** (`blueprint-statement-status`) is blueprint-exclusive — no machine signal contradicts it.
- The **proof axis** carries two independent fields: `verification-status` remains probe-lean's machine sorry-truth (a `sorry` can never render green, consistent with every other probe), and `blueprint-proof-status` records the blueprint's declared/derived claim.
- When the blueprint claims a proof is complete (`proved`/`fully-proved`) but the machine status is `unverified`/`failed`, `blueprint-status-mismatch` is set. probe-leanblueprint MUST NOT silently rewrite `verification-status` to match a blueprint claim.
- Synthetic **planned** atoms (blueprint nodes with no Lean binding) carry **no** `verification-status` — they are roadmap items, not verified/unverified code. They are non-stubs (P3) via a non-empty `code-path` marker.

**Why it matters**: the blueprint is doc-authoritative for *intent* (what should be formalized) but its proof claims — especially Massot's human-authored `\leanok` — are not machine-checked for sorry-freeness. Keeping the machine status authoritative preserves the ecosystem invariant that `verified` means checked, while the mismatch flag surfaces over-claims for review.

## Known bugs and edge cases

### Resolved

- **C6** *(fixed)*: `strategy_rust_qualified_name` in probe-aeneas now uses `HashMap<String, Vec<String>>` for RQN→Rust-atom lookup with disambiguation when multiple candidates share a normalized RQN.
- **C7** *(fixed)*: `enrich_with_aeneas_metadata` in probe-aeneas skips `translation-text` when `start == 0 || end == 0`.
- **C8** *(fixed)*: `load_mappings()` uses `HashMap<String, Vec<String>>` with `or_default().push()` — duplicate `from` keys collect all targets (1-to-many). Covered by `test_duplicate_from_keys_preserved` and `test_one_to_many_mapping_produces_multiple_edges`.
