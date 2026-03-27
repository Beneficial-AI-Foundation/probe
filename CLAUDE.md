# CLAUDE.md — probe (central hub)

## Project Overview

Central hub of the probe ecosystem: defines Schema 2.0 types and the universal `merge` operator for composing call graph data across tools and languages.

## Build and Test

```bash
cargo build
cargo test
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Knowledge Base

- The `kb/` directory is the **source of truth** for the probe ecosystem. It defines what the system should be. The code is an implementation of that definition.
- Read `kb/index.md` before starting any task to orient yourself.
- If your implementation contradicts `kb/engineering/schema.md`, `kb/engineering/architecture.md`, or `kb/engineering/properties.md`, your implementation is wrong. Fix the code, not the KB.
- KB specification files (engineering/, product/, decisions/) are only updated when the human explicitly refines the intent — never to accommodate implementation shortcuts.
- When creating new features that EXTEND the spec without contradicting it, add a corresponding KB entry and run the kb-update skill.
- After significant changes, run the kb-update skill to verify KB consistency.

## Working With the Knowledge Base

- Before implementing any task, read `kb/index.md` and load the relevant KB files.
- The KB is the specification. Your code must conform to it.
- Use `kb/engineering/properties.md` as your correctness checklist — every change must preserve listed invariants. If you cannot satisfy a property, stop and ask — do not silently drop or weaken it.
- Use `kb/engineering/glossary.md` for terminology — use the exact terms defined there.
- When the task is ambiguous, check `kb/product/spec.md` and `kb/engineering/architecture.md` before asking the user. The answer is often already documented.
- If you find a contradiction between your implementation and the KB, the implementation is wrong. Fix the code to match the spec.
- Reference KB files in commit messages when a change is driven by a KB property or design decision.

## Development Loop (Ralph Loop)

For every implementation task:
1. Implement the change
2. Run all relevant auditor skills (`/ambiguity-auditor`, `/code-quality-auditor`, `/test-quality-auditor`)
3. Read the audit reports in `kb/reports/`
4. Fix every issue found
5. Repeat steps 2-4 until all auditors pass clean
6. Run the validation suite (`cargo test`) before considering the task done

Never skip the audit step. Never mark a task complete with unresolved audit findings.

### When to run the full loop

Run it when touching:
- Merge algorithm or schema types (`types.rs`, `merge.rs`)
- Subcommand behavior (adding/changing CLI commands)
- Cross-tool data flow (envelope format, translation generation)
- Anything that could violate a property in `kb/engineering/properties.md`

For trivial changes (typo fixes, comment updates, dependency bumps), the full loop is overkill — just run `cargo test`.

### Chaining in a single prompt

The user may ask you to run the full loop in one shot:
```
Implement [feature]. Then run the Ralph Loop: run all three auditor
skills, fix every issue, repeat until clean, then run cargo test.
```
When asked this way, loop autonomously through implement → audit → fix cycles until convergence.

### After spec changes

When the user deliberately changes a design decision or adds a capability, run `/kb-update` to sync the KB. The kb-update skill checks whether code changes contradict the KB and adds entries for new concepts.

## Key Properties

The following invariants (from `kb/engineering/properties.md`) are most commonly relevant:
- **P3**: Stub = empty code-path AND lines 0,0 (structural, not heuristic)
- **P6**: Atom merge is first-wins with stub replacement
- **P7**: Specs/proofs merge is last-wins
- **P9**: Provenance is preserved and flattened through recursive merges
- **P10**: Extensions are preserved through merge
- **P14**: All output is deterministic (BTreeMap, sorted keys)

## Project Structure

```
src/
  main.rs          # CLI: `probe merge`
  lib.rs           # Module exports
  types.rs         # Atom, AtomEnvelope, MergedEnvelope, SchemaCategory, loading
  commands/
    merge.rs       # Merge algorithm, normalization, translation application
probe-extract-check/  # Validator for extract output vs source code
kb/                   # Knowledge base (source of truth)
docs/                 # Design documents (reference, not normative)
schemas/              # JSON schema files
tests/                # Integration tests
```

## Cross-Tool KB Access

The KB covers the entire probe ecosystem, not just the probe hub. When working in other tool directories (probe-rust, probe-verus, probe-lean, probe-aeneas), add this to their CLAUDE.md:

```markdown
## Knowledge Base
- The ecosystem KB lives at `../probe/kb/`. Read `../probe/kb/index.md` for orientation.
- Your implementation must conform to `../probe/kb/engineering/properties.md`.
- Use terminology from `../probe/kb/engineering/glossary.md`.
```

## Versioning and Changelog

When modifying code in any of the 5 probes (`probe`, `probe-rust`, `probe-verus`, `probe-lean`, `probe-aeneas`), always check whether the change warrants:

1. **A version bump** — update `version` in `Cargo.toml` (or the equivalent manifest). Use semver: patch for bug fixes, minor for new features/backward-compatible changes, major for breaking changes.
2. **A `CHANGELOG.md` entry** — add a concise entry under an `## [Unreleased]` section (create one if it doesn't exist). Group entries by `Added`, `Changed`, `Fixed`, or `Removed`.

Skip both for purely internal changes that don't affect behavior (comment edits, formatting, CI config tweaks).

## Commit Message Style

Conventional commits. Reference KB files when applicable:
```
fix: preserve extensions through atom merge (P10)
feat: add translations support to merge (ADR-003)
```
