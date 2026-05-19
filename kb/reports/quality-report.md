---
auditor: code-quality-auditor
date: 2026-05-12
scope: propagate-verification-status feature (propagate command, tests, KB/docs updates)
status: 0 critical, 2 warnings, 5 info
---

## Critical

None. Implementation in `src/commands/propagate.rs` matches **P23** (explicit `unverified` / `failed` only as contamination sources; `trusted` and missing status transparent; missing deps treated as trusted with sorted/deduped warnings; only `verification-status: "verified"` atoms receive `transitive-verification-status`; graph walks use `BTreeMap` / `BTreeSet` / deterministic iteration). **P14** is satisfied for the atom map and dependency edges (`BTreeSet` → sorted JSON arrays). **P10** is satisfied structurally: the command only inserts `extensions["transitive-verification-status"]` and does not strip other flattened fields through serde.

## Warnings

- **P23 test gap — `failed` as contaminant**: `is_contamination_source` correctly treats `"failed"` like `"unverified"`, but neither unit tests in `propagate.rs` nor integration tests in `tests/propagate.rs` exercise a verified atom whose dependency is `"failed"`. Add at least one regression test so the `"failed"` branch cannot drift.

- **P10 test gap — CLI round-trip**: Integration tests assert envelope top-level fields (`test_envelope_structure_preserved`) and transitive scope on selected atoms, but no test asserts that unrelated extension-style fields (e.g. a synthetic extra JSON property on an atom) survive `probe propagate-verification-status` end-to-end. A single fixture field would lock **P10** for this command path.

## Info

- **Architecture doc staleness**: `kb/engineering/architecture.md` still describes the probe hub as exposing subcommands **`merge`, `summary` only** and does not list `src/commands/propagate.rs` or `propagate-verification-status`. Align with the current CLI surface.

- **Human-facing verification doc**: `docs/verification-statuses.md` describes transitive vs locally-scoped verified at the UX/color level but does not mention the concrete atom field `transitive-verification-status` or the `probe propagate-verification-status` command. Cross-linking would reduce drift between KB/schema and product language.

- **Glossary**: `kb/engineering/glossary.md` defines `verification-status` semantics indirectly (e.g. trusted) but has no short entry for transitive scope / `transitive-verification-status`, despite that term appearing in schema and P23.

- **Code–KB traceability**: `src/main.rs` file-level `// @kb:` points at merge tooling; the new subcommand has good Clap docstrings but no `@kb` pointer to P23 or `schema.md` for auditors tracing CLI → spec.

- **Schema narrative symmetry**: `kb/engineering/schema.md` documents `transitive-verification-status` in the common optional-fields table; the following “Tool-specific extension fields” section lists probe-verus / probe-lean / probe-aeneas / probe-rust bullets but not a **probe (hub)** bullet for hub-computed extensions. Optional polish for readers scanning by tool.
