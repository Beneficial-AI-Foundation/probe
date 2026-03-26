---
title: "ADR-001: Keep probe tools in separate directories"
last-updated: 2026-03-19
status: accepted
---

# ADR-001: Keep probe tools in separate directories

## Context

The probe ecosystem has five tools that share a common schema and interchange format. The question is whether to consolidate them into a single Cargo workspace / monorepo or keep them as separate directories under `baif/`.

## Decision

Keep five separate directories: `probe/`, `probe-rust/`, `probe-verus/`, `probe-lean/`, `probe-aeneas/`.

## Rationale

### probe-lean requires Lean toolchain

probe-lean is written in Lean 4 and built with `lake`. It cannot be a Cargo workspace member. This alone forces at least one separation.

### Different external dependencies

Each tool depends on a different language analyzer:
- probe-rust: rust-analyzer + scip
- probe-verus: verus-analyzer + scip + cargo verus
- probe-lean: Lean 4 + lake + elan
- probe-aeneas: probe-rust + probe-lean + lake

A developer working on one tool shouldn't need all four toolchains installed.

### Different versioning cadence

probe-verus is at v5.0 with a formal versioning policy (semver, CHANGELOG.md). Other tools are at earlier versions. Separate directories allow independent version bumps.

### probe-aeneas has a different role

probe-aeneas is an orchestrator/functor factory, not a peer extractor. It imports probe as a Rust crate dependency and delegates merging to it. Its architectural role is fundamentally different from the extractors.

### Shared types via crate dependency

The consolidation opportunity is in shared types, not repo merging. probe-aeneas already imports `probe` as a local path dependency. Other tools share the schema by convention (matching JSON format) rather than by code dependency.

## Consequences

- Each tool has its own CLAUDE.md, test suite, and CI
- Schema evolution requires coordinated changes across directories
- probe-aeneas depends on probe crate via `../probe` path — moving directories breaks this
- New tools (e.g. `probe-haskell`) can be added as new directories without touching existing ones
- Developers can work on one tool without understanding the others

## Alternatives considered

### Single Cargo workspace

Would simplify dependency management for Rust tools but still can't include probe-lean. Would force all Rust tools to share the same nightly toolchain version. Rejected.

### Shared types as published crate

Publishing `probe-types` to crates.io would allow tools to depend on it without path references. Premature — the schema is still evolving and the only consumer of path dependencies is probe-aeneas.
