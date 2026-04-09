---
auditor: ambiguity-auditor
date: 2026-04-07
pass: trusted-reason (probe-lean KB + cross-file consistency)
status: 0 critical, 3 warnings, 8 info
---

## Scope

Audit focused on **`trusted-reason`** and **`verification-status: "trusted"`** after KB updates to [schema.md](../engineering/schema.md), [probe-lean.md](../tools/probe-lean.md), and [glossary.md](../engineering/glossary.md). Requested files were read in full; [probe-verus.md](../tools/probe-verus.md) was additionally scanned because it is the paired Verus tool doc and [schema.md](../engineering/schema.md) documents `trusted-reason` for both tools.

## Critical

None.

## Warnings

| ID | Topic | Notes |
|----|--------|--------|
| **W1** | [probe-verus.md](../tools/probe-verus.md) vs [schema.md](../engineering/schema.md) | **Stale / contradictory.** Output fields list gives `verification-status` as only `"verified"`, `"failed"`, `"unverified"` and does not mention `"trusted"` or `trusted-reason`. [schema.md](../engineering/schema.md) and [glossary.md](../engineering/glossary.md) state probe-verus emits `trusted-reason` when status is `"trusted"` (values `"admit"`, `"external-body"`, `"assume-specification"`). Readers using the tool page alone will mis-implement consumers or think the schema table is wrong. |
| **W2** | [P16](../engineering/properties.md#p16-verification-status-mapping) incomplete for probe-verus | **Normative gap.** P16 documents Verus run output mapping (`success` / `failure` / `sorries` / `warning`) but never states when atoms are `"trusted"` or how that relates to `trusted-reason`. Trust-base behavior for Verus is only implied elsewhere ([glossary](../engineering/glossary.md#trusted-verification-status), [schema optional fields](../engineering/schema.md#core-fields-required-for-all-languages)). P16 should either add a Verus `"trusted"` row/table or explicitly defer to glossary/schema for trust-base + `trusted-reason`. |
| **W3** | [glossary.md](../engineering/glossary.md#trusted-verification-status) “trusted” lede | **Vague / easy to misread.** The opening sentence describes only Lean (`axiom`, `*External.lean`) before the list that covers Verus. A quick read suggests `"trusted"` is Lean-only; the full entry is cross-tool. Consider leading with “Cross-tool value” or splitting into probe-lean vs probe-verus sentences up front. |

## Info

1. **P16 vs probe-lean detail:** [probe-lean.md](../tools/probe-lean.md) documents `trusted-reason` per row and **axiom vs external precedence**; [P16](../engineering/properties.md#p16-verification-status-mapping) gives Lean `verification-status` only. Precedence is tool-doc-specific, which is acceptable, but P16 does not cross-link `trusted-reason` or [glossary](../engineering/glossary.md#trusted-verification-status) for the Lean trust base.

2. **Schema version history:** [schema.md § Version history](../engineering/schema.md#version-history) still only records probe-rust 2.1 optional fields. Introduction of `trusted-reason` (and normative pairing with `"trusted"`) for probe-verus / probe-lean is not reflected there — optional for traceability, useful for consumers asking “when did this field appear?”.

3. **product/spec.md:** [§ Core capabilities — Verification status](../product/spec.md#core-capabilities) mentions Lean axioms / `*External.lean` as trusted but does not mention **`trusted-reason`** as the machine-readable classifier or Verus trust-base categories. Low priority for product-level doc.

4. **Glossary cross-links:** Under [trusted (verification-status)](../engineering/glossary.md#trusted-verification-status), the see-also points at P16 and probe-lean; adding [probe-verus.md](../tools/probe-verus.md) (once fixed) would balance cross-navigation.

5. **index.md metadata:** [index.md](../index.md) `last-updated: 2026-04-03` lags pages touched for `trusted-reason` (`2026-04-07`). Cosmetic only.

6. **viewify and `trusted-reason`:** [probe-lean.md](../tools/probe-lean.md) documents `trusted-reason` on extract output but does not state whether **`viewify` preserves** the field on molecules. If molecules are a strict subset/projection, stating pass-through or omission would remove ambiguity for UI authors.

7. **Prior P14 warning (unchanged):** [P14](../engineering/properties.md#p14-deterministic-output) headline still reads as universal deterministic output while probe-rust caveat remains below; unrelated to `trusted-reason` but still a clarity hazard (same as prior ambiguity-auditor finding W1).

8. **Code-name pattern `*External.lean`:** Used consistently; [glossary](../engineering/glossary.md#trusted-verification-status) ties it to Aeneas convention. No contradiction found with [probe-lean.md](../tools/probe-lean.md) table (`code-path` ends with `External.lean`).

## Consistency checks (clean)

- [schema.md](../engineering/schema.md) optional-field description of `trusted-reason` matches [glossary.md](../engineering/glossary.md) enumerations for both tools.
- [probe-lean.md](../tools/probe-lean.md) verification table and Lean-specific fields table align with [schema.md](../engineering/schema.md) and with [trust base](../engineering/glossary.md#trust-base) wording.
- [trust base](../engineering/glossary.md#trust-base) correctly references both tools and `trusted-reason`.
- No conflict found between “`trusted-reason` present only when `verification-status` is `"trusted"`” across schema, glossary, and probe-lean tool doc.

## Files read (this pass)

`kb/index.md`, `kb/engineering/properties.md`, `kb/engineering/schema.md`, `kb/engineering/glossary.md`, `kb/tools/probe-lean.md`, `kb/product/spec.md`

## Additional KB file scanned

`kb/tools/probe-verus.md` (for `trusted-reason` / `"trusted"` alignment with schema)
