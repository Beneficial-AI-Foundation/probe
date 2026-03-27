---
auditor: code-quality-auditor
date: 2026-03-27
status: 0 critical, 2 warnings, 9 info
scope: is-public visibility feature (probe-rust + probe-aeneas)
---

## Critical

None. No envelope breakage, no forbidden cross-repo path dependencies, no conflict with stub identity (P3).

## Warnings

### [W1] "Unknown" vs `false` for `is-public` semantics
- **Location**: `probe-aeneas/src/extract.rs` enrich_with_aeneas_metadata
- **Issue**: Rust atoms without `is-public` get defaulted to `false`. This does not distinguish truly-private-per-Charon from never-enriched/Charon-absent/match-failed. Consumers may interpret `false` as ground truth.
- **Recommendation**: Document in SCHEMA.md that `false` means "not declared `pub` OR visibility data unavailable." Already done in probe-aeneas SCHEMA.md semantic section.

### [W2] Multi-candidate non-resolution leaves `is_public` as `None`
- **Location**: `probe-rust/src/charon_names.rs` enrich_atoms_with_charon_names
- **Issue**: When multiple candidates exist and both span disambiguation and heuristic RQN match fail, code leaves prior heuristic `rust_qualified_name` unchanged but does not set `is_public`. Low frequency but slightly inconsistent.
- **Recommendation**: Document or accept as intentional (no reliable candidate → no visibility claim).

## Info

### [I1] P1 (Envelope completeness) — OK
`is-public` is an extension field, not part of envelope structure.

### [I2] P2 (Atom identity) — OK
Identity remains code-name / map keys. `is-public` is metadata.

### [I3] P3 (Stubs) — OK
Stubs structural. `is-public` orthogonal.

### [I4] P10 (Extensions preserved) — OK
`#[serde(flatten)]` on `Atom.extensions` captures `is-public` through merge. probe-aeneas conditional insert preserves Charon-derived values.

### [I5] P14 (Deterministic output) — OK
LLBC is deterministic; `Option<bool>` serialization skips `None`; `BTreeMap` used.

### [I6] P19 (No cross-repo path deps) — OK
Both probe-rust and probe-aeneas use `git = "https://..."` for cross-repo deps.

### [I7] probe-rust implementation — OK
`FunDeclMeta` / `CharonFunInfo::is_public` / `build_fun_span_map` correctly implemented. All three enrichment paths propagate visibility.

### [I8] probe-rust `AtomWithLines` — OK
Field properly configured: rename, skip_serializing_if, default.

### [I9] Architecture fit — OK
Both tools operating within documented roles.
