# Test Quality Auditor

Verify test coverage against KB properties and identify testing gaps.

## Process

1. Read `kb/engineering/properties.md` to load all invariants
2. For each property (P1-P18), check if there are tests that exercise it:

### Coverage matrix

For each property, find tests in:
- `probe/tests/` and `probe/src/commands/merge.rs` (inline tests)
- `probe-rust/tests/` and source-level tests
- `probe-verus/tests/` and source-level tests
- `probe-lean/Tests/`
- `probe-aeneas/tests/` and source-level tests

### What to check

- **Every invariant has a corresponding test** — if P4 (associativity) is a property, there should be a test that merges `merge(A,B)` with C and compares to merging A with `merge(B,C)`
- **Known bugs (C6, C7, C8) have regression tests** — or are flagged as untested
- **Edge cases from schema.md are tested** — stubs, empty files, merged-of-merged provenance
- **Property-based testing opportunities** — properties like associativity and identity are ideal for property-based tests (proptest/quickcheck)

### Impact analysis

For recent changes (check `git log --oneline -20` in each tool directory):
- Identify which properties are affected by recent changes
- Check if those properties have test coverage for the new behavior
- Flag any property-affecting changes without corresponding test additions

## Output

Write findings to `kb/reports/test-report.md` using the standard auditor format.

### Coverage summary table

Include a table:

```markdown
| Property | Tests | Coverage | Notes |
|----------|-------|----------|-------|
| P1 | probe/tests/envelope_test.rs | Full | |
| P4 | probe/src/commands/merge.rs::test_recursive_merge | Partial | Only tests 3 files |
| P11 | probe-aeneas/src/translate.rs::tests | Full | |
```

## Severity guide

- **Critical**: Property with no test coverage at all
- **Warning**: Property with partial coverage, or known bug without regression test
- **Info**: Property-based testing opportunity, or test that could be more precise
