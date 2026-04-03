---
title: probe query
last-updated: 2026-04-03
status: draft
---

# probe query

Read-only analysis subcommand of the probe hub. Partitions atoms with `verification-status: "verified"` into two disjoint lists.

## Entrypoints

Verified, non-stub, non-test, Rust `exec` atoms whose code-name never appears in any atom's `dependencies` array. These represent the API surface of the project — functions that are verified but not called by other verified functions in the graph.

Criteria (all must hold):

| Criterion | Check |
|-----------|-------|
| Verified | `extensions["verification-status"] == "verified"` |
| Non-stub | `is_stub() == false` ([P3](../engineering/properties.md#p3-stub-detection-is-structural)) |
| Non-test | `code_module` and `display_name` do not contain `"test"` |
| Rust exec | `language == "rust"` and `kind == "exec"` ([P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope)) |
| Not depended upon | code-name does not appear in any atom's `dependencies` |

## Verified dependencies

All verified atoms that are **not** entrypoints. This includes stubs, specs, proofs, test functions, depended-upon exec functions, and non-Rust atoms.

## Partition property

`entrypoints ∪ verified_dependencies = { a ∈ atoms | verified(a) }` and `entrypoints ∩ verified_dependencies = ∅`.

## Output format

Schema 2.0 envelope ([P1](../engineering/properties.md#p1-envelope-completeness)) with `schema: "probe/query"`. The `data` field contains:

```json
{
  "entrypoints": ["code-name-1", "code-name-2"],
  "verified_dependencies": ["code-name-3", "code-name-4"]
}
```

Both arrays are sorted by code-name ([P14](../engineering/properties.md#p14-deterministic-output)).

## CLI

```
probe query <INPUT> [-o <OUTPUT>]
```

- `INPUT` — Schema 2.0 atom file (required)
- `-o OUTPUT` — Write envelope to file (defaults to stdout)

Summary statistics are always printed to stderr.

## Implementation

`src/commands/query.rs` — annotated with `@kb` references to [P1](../engineering/properties.md#p1-envelope-completeness), [P3](../engineering/properties.md#p3-stub-detection-is-structural), [P14](../engineering/properties.md#p14-deterministic-output), [P20](../engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope).
