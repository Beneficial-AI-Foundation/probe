---
title: Properties and Invariants
last-updated: 2026-03-19
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

## P11. Translation mapping is 1-to-1

When generating cross-language translations (probe-aeneas):
- Each Rust atom maps to at most one Lean atom
- Each Lean atom is claimed by at most one Rust atom
- Once matched, neither side can be matched again

Enforced by `matched_rust` and `matched_lean` HashSets in `probe-aeneas/src/translate.rs`.

## P12. Translation strategy priority

The three matching strategies run in strict priority order:
1. **Rust-qualified-name** (confidence: `exact` or `exact-disambiguated`) — Charon-derived names
2. **File + display-name** (confidence: `file-and-name`) — same source file + matching base name, unambiguous matches only
3. **File + line-overlap** (confidence: `file-and-lines`) — same source file + overlapping line ranges, best overlap wins

Higher-priority strategies run first and claim atoms. Lower-priority strategies only see unclaimed atoms.

## P13. Cross-language edges require existence

When applying translations during merge:
- A translated dependency is added only if the target code-name exists in the merged key set
- A translated dependency is not added if it's already present in the atom's dependencies
- Both mapping directions (from→to and to→from) are checked

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

For probe-verus, Verus verification output maps to `verification-status` as:

| Verus status | `verification-status` |
|---|---|
| `success` | `"verified"` |
| `failure` | `"failed"` |
| `sorries` | `"unverified"` |
| `warning` | `"unverified"` |

For probe-lean, sorry detection in build output determines status: definitions with sorry warnings are `"unverified"`, otherwise `"verified"`.

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

## Known bugs and edge cases

These are documented defects that should be fixed, not acceptable behavior:

- **C6**: When two Rust atoms share the same normalized RQN in probe-aeneas translation, `rqn_to_rust.insert()` overwrites (last-wins in HashMap.insert), silently dropping the first. Should collect into a Vec.
- **C7**: Lean atoms without source location (lines 0,0) get misleading `translation-text` in probe-aeneas enrichment. Should check and skip.
- **C8**: Duplicate translation `from` keys silently overwrite in `load_translations()`. Last-wins in HashMap.insert.
