# KB Update

Maintain the KB as implementation progresses. The KB is the source of truth — code must conform to the KB, not the other way around.

## Trigger

Run this after any change to:
- Architecture or component boundaries
- Schema 2.0 fields or envelope structure
- Merge algorithm behavior
- Translation matching strategies
- Cross-tool data flow
- Design decisions or invariants
- Domain terminology

## Process

1. Read `kb/index.md` to understand current KB structure
2. Compare the current changes against the relevant KB files
3. If the code contradicts a KB specification file (`kb/engineering/schema.md`, `kb/engineering/architecture.md`, `kb/engineering/properties.md`, `kb/engineering/glossary.md`, `kb/decisions/`):
   - This is an implementation error. Flag it. Do NOT update the KB to match the code.
   - Fix the code to match the KB, or ask the user if the spec should be refined.
4. If the code introduces new concepts that EXTEND the KB without contradicting it:
   - Add new KB entries (new file or new section in an existing file)
   - Add the entry to the relevant index.md
   - Keep frontmatter `last-updated` current
   - Preserve cross-references; add new ones if needed
5. Run the ambiguity auditor (see below) on modified KB files
6. Verify all links in modified files resolve correctly

## What NOT to do

- Never weaken a property in `kb/engineering/properties.md` because the implementation is hard
- Never change `kb/engineering/schema.md` to match what the code happens to do
- Never change `kb/product/spec.md` to remove a capability because it's not yet implemented
- Never remove a known bug from `properties.md` without fixing it first
