---
title: probe summary
last-updated: 2026-09-28
status: draft
---

# probe summary

Read-only analysis subcommand of the probe hub. Partitions verified atoms into four disjoint lists: three local-evidence partitions plus `imported-verified`.

Input authority: summary runs the **version-gate component** of the shared validator ([ADR-006](../decisions/006-correspondence-records.md)) at its load boundary — an unmarked pre-contract `verified` is indistinguishable from local evidence, so the consumer contract below is enforceable only behind the gate. Projections remain readable (views with inherited labels); summary reports the artifact's own labels and never recomputes enrichment.

## Scope: code atoms only

All partitions are computed over **code atoms**: atoms with `language: "blueprint"` (probe-leanblueprint synthetic nodes) appear in no list and are excluded from the `depended_upon` set used for entrypoint classification — presentation-layer binding edges must not alter code entrypoints, and one checked theorem must report as one verified lemma, not one per bound node.

## Imported evidence (`imported-verified`)

Membership: `status-origin == "translation"` **AND** status ∈ {`verified`, `transitively-verified`}. The conjunction is required because the marker rides on every copied status (probe-aeneas also copies `trusted`/`failed`); a translated non-verified status appears in **no** verified list. Names only, deterministic order. Atoms in this list appear in none of the three local partitions — summary must not present imported evidence as a locally established result.

`status-origin: "kernel-taint"` atoms stay in the ordinary partitions: that evidence is local (a real, locally checked proof); the marker only makes it non-promotable ([P23](../engineering/properties.md#p23-transitive-verification)).

## Entrypoints

Verified, non-stub, non-test, Rust `exec` atoms whose code-name never appears in any non-test atom's `dependencies` array. These represent the API surface of the project — functions that are verified but not called by other verified functions in the graph.

Criteria (all must hold):

| Criterion | Check |
|-----------|-------|
| Verified | `verification-status` ∈ {`"verified"`, `"transitively-verified"`}, and no `status-origin: "translation"` |
| Non-stub | `is_stub() == false` ([P3](../engineering/properties.md#p3-stub-detection-is-structural)) |
| Non-test | `code_module` and `display_name` do not contain `"test"` |
| Rust exec | `language == "rust"` and `kind == "exec"` ([schema.md § Language assignment for Verus atoms](../engineering/schema.md#language-assignment-for-verus-atoms)) |
| Not depended upon | code-name does not appear in any non-test atom's `dependencies` |

## Verified functions

All verified Rust `exec` atoms that are **not** entrypoints. This includes depended-upon helper functions, stubs, and test functions.

## Verified lemmas

All verified Verus `proof`/`spec` atoms.

## Partition property

`verified_entrypoints ∪ verified_functions ∪ verified_lemmas ∪ imported_verified = { a ∈ atoms | verified(a) ∧ language(a) ≠ "blueprint" }` and the four sets are pairwise disjoint. The three local partitions contain only atoms without `status-origin: "translation"`.

## Output format

Schema 3.x envelope ([P1](../engineering/properties.md#p1-envelope-completeness)) with `schema: "probe/summary"`. The `data` field contains:

```json
{
  "verified_entrypoints": ["code-name-1", "code-name-2"],
  "verified_functions": ["code-name-3", "code-name-4"],
  "verified_lemmas": ["code-name-5", "code-name-6"],
  "imported_verified": ["code-name-7"]
}
```

All arrays are sorted by code-name ([P14](../engineering/properties.md#p14-deterministic-output)). The summary output has no JSON schema in `schemas/` — it is human/CI-facing output, not a cross-tool interchange contract; the lists are covered by unit tests on `SummaryResult`.

## CLI

```
probe summary <INPUT> [-o <OUTPUT>]
```

- `INPUT` — Schema 3.0 atom file (required)
- `-o OUTPUT` — Write envelope to file (defaults to `summary_<package>_<version>.json`)

Summary statistics are always printed to stderr.

## Implementation

`src/commands/summary.rs` — annotated with `@kb` references to [P1](../engineering/properties.md#p1-envelope-completeness), [P3](../engineering/properties.md#p3-stub-detection-is-structural), [P14](../engineering/properties.md#p14-deterministic-output), and [schema.md § Language assignment for Verus atoms](../engineering/schema.md#language-assignment-for-verus-atoms).
