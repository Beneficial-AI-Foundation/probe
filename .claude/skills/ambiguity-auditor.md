# Ambiguity Auditor

Examine the KB for gaps, contradictions, undefined terms, and vague language. Ambiguity in the KB becomes bugs in the code.

## Process

Examine all files in `kb/` and identify:

1. **Undefined or inconsistently used terms** — compare against `kb/engineering/glossary.md`. Every domain term used in the KB must be defined there.

2. **Vague requirements** — flag phrases like "should be fast", "handle errors gracefully", "as appropriate". These need quantification or concrete criteria.

3. **Contradictions between files** — e.g., `schema.md` says X but `properties.md` says Y, or a tool file contradicts `architecture.md`.

4. **Missing cross-references** — concepts mentioned but not linked to their KB file.

5. **Stale content** — `last-updated` older than 30 days, or references to removed components/fields.

6. **Incomplete sections** — files marked as "planned" in `kb/index.md` that block understanding of a related file.

7. **Property coverage gaps** — invariants in `properties.md` not referenced by any tool file, or tool behaviors not covered by any property.

## Output

Write findings to `kb/reports/ambiguity-report.md` using this format:

```markdown
---
auditor: ambiguity-auditor
date: YYYY-MM-DD
status: N critical, N warnings, N info
---

## Critical

### [C1] Title
- **Location**: kb/file.md, line N
- **Issue**: description
- **Recommendation**: what to do

## Warnings

### [W1] Title
...

## Info

### [I1] Title
...
```

## Severity guide

- **Critical**: Contradictions, undefined terms used in properties, vague requirements in spec
- **Warning**: Missing cross-references, stale dates, incomplete sections
- **Info**: Style inconsistencies, minor phrasing improvements
