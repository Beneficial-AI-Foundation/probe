---
title: probe summary
last-updated: 2026-04-03
status: draft
---

# probe summary

Read-only analysis subcommand of the probe hub. Partitions atoms with `verification-status: "verified"` into three disjoint lists.

## Entrypoints

Verified, non-stub, non-test, Rust `exec` atoms whose code-name never appears in any non-test atom's `dependencies` array. These represent the API surface of the project — functions that are verified but not called by other verified functions in the graph.

Criteria (all must hold):

| Criterion | Check |
|-----------|-------|
| Verified | `extensions["verification-status"] == "verified"` |
| Non-stub | `is_stub() == false` ([P3](../engineering/properties.md#p3-stub-detection-is-structural)) |
| Non-test | `code_module` and `display_name` do not contain `"test"` |
| Rust exec | `language == "rust"` and `kind == "exec"` ([P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope)) |
| Not depended upon | code-name does not appear in any non-test atom's `dependencies` |

## Verified functions

All verified Rust `exec` atoms that are **not** entrypoints. This includes depended-upon helper functions, stubs, and test functions.

## Verified lemmas

All verified Verus `proof`/`spec` atoms.

## Partition property

`verified_entrypoints ∪ verified_functions ∪ verified_lemmas = { a ∈ atoms | verified(a) }` and the three sets are pairwise disjoint.

## Output format

Schema 2.0 envelope ([P1](../engineering/properties.md#p1-envelope-completeness)) with `schema: "probe/summary"`. The `data` field contains:

```json
{
  "verified_entrypoints": ["code-name-1", "code-name-2"],
  "verified_functions": ["code-name-3", "code-name-4"],
  "verified_lemmas": ["code-name-5", "code-name-6"]
}
```

All arrays are sorted by code-name ([P14](../engineering/properties.md#p14-deterministic-output)).

## CLI

```
probe summary <INPUT> [-o <OUTPUT>]
```

- `INPUT` — Schema 2.0 atom file (required)
- `-o OUTPUT` — Write envelope to file (defaults to `summary_<package>_<version>.json`)

Summary statistics are always printed to stderr.

## Implementation

`src/commands/summary.rs` — annotated with `@kb` references to [P1](../engineering/properties.md#p1-envelope-completeness), [P3](../engineering/properties.md#p3-stub-detection-is-structural), [P14](../engineering/properties.md#p14-deterministic-output), [P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope).
