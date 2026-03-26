# Code Quality Auditor

Check the implementation against KB-defined properties and architectural constraints.

## Process

1. Read `kb/engineering/properties.md` to load all invariants (P1-P19)
2. Read `kb/engineering/architecture.md` to understand component boundaries
3. Read `kb/engineering/glossary.md` for precise terminology
4. For each property, verify the implementation satisfies it:

### Property checks

- **P1 (Envelope completeness)**: Verify all primary commands produce Schema 2.0 envelopes
- **P2 (Atom identity)**: Check code-name uniqueness within output files
- **P3 (Stub detection)**: Verify `is_stub()` matches the structural definition
- **P4-P5 (Merge laws)**: Check merge tests cover associativity and identity
- **P6 (First-wins)**: Verify atom merge keeps base on real-vs-real conflict
- **P7 (Last-wins)**: Verify specs/proofs merge uses last-wins
- **P8 (Normalization)**: Check trailing-dot stripping is applied consistently
- **P9 (Provenance)**: Verify `inputs` flattening in recursive merges
- **P10 (Extensions preserved)**: Check `#[serde(flatten)]` on Atom struct
- **P11-P12 (Translation 1-to-1 and priority)**: Verify matched sets enforce constraint
- **P13 (Cross-language edges)**: Check existence requirement before adding translated deps
- **P14 (Deterministic output)**: Verify BTreeMap usage throughout
- **P15 (Dependency completeness)**: Check union property for categorized dependencies
- **P16 (Verification status mapping)**: Verify mapping tables match implementation
- **P17 (Category consistency)**: Check merge validates same-category inputs
- **P18 (Lean specified derived)**: Verify no `specified` field stored on Lean atoms
- **P19 (No cross-repo path deps)**: Scan all `Cargo.toml` files for `path = "..."` dependencies where the path resolves outside the repository root (any path starting with `../` that leaves the git repo). Flag as Critical.

### Architecture checks

- Component boundaries: each tool only does what `architecture.md` says it does
- Data flow: outputs go where the diagram says
- Shared patterns: SCIP caching, auto-install, envelope construction follow documented patterns
- Naming: code uses terms as defined in `glossary.md`

### Documentation staleness checks

For each tool repo (probe-verus, probe-lean, etc.), verify docs reflect the current state:

- **Version numbers in example JSON**: `"version"` in illustrative JSON blocks should match `Cargo.toml`
- **Command names**: no references to renamed/removed commands (e.g., old `verify`/`run` when `extract` is current)
- **CLI flags**: option lists and examples don't reference deprecated flags
- **Output filenames**: documented filenames match what the code actually produces
- **Schema names**: envelope `"schema"` values match current implementation
- **Schema doc version header**: `docs/SCHEMA.md` version tracks the package version
- **Docker/Action docs**: entrypoint command, flags, and output format examples match current CLI

Common staleness pattern: a breaking rename (command, flag, schema) gets updated in the main README and SCHEMA.md but missed in Docker README, Action README, HOW_IT_WORKS, or format.md.

### Known bugs

- Check if C6, C7, C8 from `properties.md` are still present or have been fixed

## Output

Write findings to `kb/reports/quality-report.md` using the same format as the ambiguity auditor.

## Severity guide

- **Critical**: Property violation (code contradicts a KB invariant)
- **Warning**: Architectural boundary violation, missing test for a property
- **Info**: Naming inconsistency with glossary, documentation drift
