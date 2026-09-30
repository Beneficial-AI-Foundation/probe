---
title: "Tool: probe (merge operator)"
last-updated: 2026-09-30
status: draft
---

# probe (merge operator)

**Directory**: `baif/probe/`
**Role**: Central hub — defines [Schema 3.x](../engineering/schema.md) types and the universal merge operator.
**Subcommands**: `merge`, `project`, `enrich`, `summary`

## What this tool does

`probe merge` takes two or more Schema 3.x JSON files and produces a single merged output. It is the only composition operator in the ecosystem — all tools that need to combine data go through it.

See [architecture.md](../engineering/architecture.md) for how this fits into the data flow.

## Key source files

| File | Purpose |
|------|---------|
| `src/types.rs` | `Atom`, `AtomEnvelope`, `MergedEnvelope<D>`, `SchemaCategory`, `load_envelope()`, `load_mappings()` |
| `src/commands/merge.rs` | `merge_atom_maps()` / `merge_atom_files()` (re-enriching), `merge_atom_maps_raw()` / `merge_atom_files_raw()` (staging, stale statuses), `merge_generic_maps()`, `normalize_atoms()`, `cmd_merge()` |
| `src/main.rs` | CLI: `probe merge <file1> <file2> [--output] [--mappings]` |

## Merge algorithm detail

### Phase 1: Load and validate authority

1. Parse each input file's envelope
2. Validate `schema-version` starts with `"3."`
3. **Reject projections** — inputs carrying `probe/projected-atoms`, or the legacy form (`probe/merged-atoms` plus a `projection` envelope field), are errors: projections are views, and deleting edges must not improve assurance ([ADR-006](../decisions/006-correspondence-records.md))
4. **Reject pre-contract envelopes** — the per-producer version gate ([schema.md § Authority validation](../engineering/schema.md#authority-validation-and-re-enrichment)); the `probe` entry is an interval plus a `tool.command: "merge-atoms"` rejection
5. Detect [schema category](../engineering/glossary.md#schema-category) from `schema` field
6. Validate all inputs belong to the same category
7. Flatten provenance from all inputs — composed shape detected structurally (`inputs` vs `source`), entries deduplicated ([P9](../engineering/properties.md#p9-provenance-is-preserved))

Authority validation (steps 3–4) is one shared validator invoked at every envelope boundary: `cmd_merge`, `merge_atom_files`, `cmd_enrich`, and the raw staging entry point `merge_atom_files_raw` (raw means skip recomputation, never skip validation); `probe summary`/`probe project` run the version-gate component. All file-level merge paths also fail closed on out-of-enum `status-origin` markers (ADR-006 Decision 2). The bare-map library API (`merge_atom_maps`) cannot check authority — callers own the envelope boundary.

### Phase 2: Normalize (per input, before conflict resolution)

Strip trailing `.` from all code-name keys, dependency references, code-name-bearing extension arrays, and mapping endpoints ([P8](../engineering/properties.md#p8-code-name-normalization)). Per-input ordering is semantic: aliases collapse within their own input before cross-input conflicts are resolved, so Phase 3 pairs already-normalized atoms.

Correspondence-record fields on input atoms are validated fail-closed and put in canonical form in the same pass: a malformed `maps-to`/`mapped-from` shape (anything other than an array of well-typed `{target, confidence, method?}` objects — the normative list is [P27](../engineering/properties.md#p27-correspondence-records-are-unioned-and-inert)'s) **errors** the offending input — merge re-emits these fields into schema-constrained output, and a malformed field silently swallowing a record union would make the result grouping-dependent (P4). An empty `method` is canonicalized to absent, and record arrays are sorted and deduped by the identity triple. Mapping-file confidences are validated against the same vocabulary (at load, and again at the merge boundary for in-memory `Mapping` values).

If normalization makes two keys within the same file collide, stub-vs-real resolution applies (Phase 3 rules) and identical-modulo-records duplicates collapse; if both are real atoms and differ beyond their correspondence records, the merge **errors** — a single input offering two distinct atoms for one code-name is producer error, and merge re-enriches (Phase 5), so silently selecting one atom's evidence would launder contamination; the same rule the unary `probe enrich` boundary applies ([P8](../engineering/properties.md#p8-code-name-normalization)). Correspondence records are unioned across benign collisions ([P27](../engineering/properties.md#p27-correspondence-records-are-unioned-and-inert)). Per-input rejections are prefixed with the 1-based input position (`input #2 of 3: …`) so the offending producer file is identifiable from the argument order.

On the specs/proofs path an intra-input collision keeps last-wins ([P7](../engineering/properties.md#p7-specsproofs-merge-is-last-wins)) — no verification evidence is at stake — but is warned about and counted in `stats.conflicts` ([P8](../engineering/properties.md#p8-code-name-normalization)).

### Phase 3: Merge

- **Atoms**: `merge_atom_maps()` — first-wins with [stub](../engineering/glossary.md#stub) replacement. See [P6](../engineering/properties.md#p6-atom-merge-is-first-wins-with-stub-replacement). On every equal-key resolution, `maps-to`/`mapped-from` records are unioned ([P27](../engineering/properties.md#p27-correspondence-records-are-unioned-and-inert)).
- **Specs/Proofs**: `merge_generic_maps()` — last-wins. See [P7](../engineering/properties.md#p7-specsproofs-merge-is-last-wins).

Matching is purely by code-name string equality, regardless of language. Atoms with different `language` values coexist in the merged `data` dictionary, and a stub in one language is never resolved against an atom in another — the two stay distinct atoms, linked (if at all) by a correspondence record.

### Phase 4: Attach correspondence records (optional)

When `--mappings <file>` is provided, each mapping entry attaches a `maps-to` record to its `from` atom and a `mapped-from` record to its `to` atom — `dependencies` is never modified. Attachment is unconditional (dangling target ⇒ warning, not skip), key-local, and set-like. See [P13](../engineering/properties.md#p13-correspondence-records-attach-unconditionally) and [schema.md § Correspondence records](../engineering/schema.md#correspondence-records-maps-to-mapped-from).

### Phase 5: Re-enrich

After all inputs are combined, the atoms category runs enrichment recomputation ([P23](../engineering/properties.md#p23-transitive-verification)) — stub resolution can invalidate labels computed at extract time, so merged labels are recomputed, never inherited. The raw staging primitives (`merge_atom_maps_raw`/`merge_atom_files_raw`) for multi-step pipelines (probe-aeneas) defer this single enrichment pass; the file-level `merge_atom_files_raw` still validates authority, while the bare-map `merge_atom_maps_raw` — like `merge_atom_maps` — cannot (callers own the envelope boundary). Their output carries potentially stale derived statuses.

### Phase 6: Write output

Construct merged envelope with `inputs` array (not `source`), serialize with sorted keys for [determinism](../engineering/properties.md#p14-deterministic-output).

## Statistics reported

After merging, the tool prints:

| Metric | Atoms | Specs/Proofs |
|--------|-------|-------------|
| Total entries | yes | yes |
| Stubs replaced | yes | — |
| Stubs remaining | yes | — |
| New entries added | yes | yes |
| Keys normalized | yes | yes |
| Conflicts | yes (cross-input real-vs-real, base kept; intra-input distinct-real collisions are errors, not counts) | yes (overrides, incoming kept) |
| Records attached | yes (if `--mappings`: `maps-to`/`mapped-from` counts, dangling-target warnings) | — |
| Enrichment | yes (transitively-verified / locally-scoped verified counts from the Phase 5 recomputation) | — |

## Categorical framework

`probe merge` is described algebraically in [categorical-framework.md](../engineering/categorical-framework.md). Merge factors as `F_M ∘ μ` (plain merge, then correspondence-record attachment for fixed mappings M). On the **carrier** — normalized, enrichment-consistent atom maps — μ satisfies [associativity](../engineering/properties.md#p4-merge-associativity-on-the-carrier), [identity](../engineering/properties.md#p5-merge-identity-exact-on-the-carrier) (exact on the carrier; up to normalization+enrichment on legacy inputs), and commutativity for disjoint keys; F_M is idempotent and compatible with μ (`F_M(μ(A, B)) = μ(F_M(A), F_M(B))`). Projected artifacts are outside μ's domain (rejected). Laws are stated modulo envelope meta and over provenance as a deduplicated source inventory. Each probe tool is a [doctrine](../engineering/glossary.md#doctrine); probe-aeneas is a [functor](../engineering/glossary.md#functor) factory.

## probe-extract-check

Subdirectory `probe/probe-extract-check/` (~2.4K LOC). Validates extract JSON against actual source code:
- Checks that `code-path` files exist
- Checks that line ranges are valid
- Verifies atom metadata consistency with source

Used in probe-rust and probe-verus test suites.

## Relationship to probe-verus merge-atoms

probe-verus's `merge-atoms` is a legacy independent merge implementation, slated for retirement or reimplementation as a caller of the hub's authority-validating merge ([ADR-006](../decisions/006-correspondence-records.md)). Its envelope writer masquerades as the hub (`tool.name: "probe"` at probe-verus's own version) and its atom type drops verification statuses, `status-origin` markers, and correspondence records. The hub's version gate rejects its outputs at any version: the `probe` gate entry is an interval (`< 1.0.0`) and additionally rejects `tool.command: "merge-atoms"`, a command the hub never shipped.
