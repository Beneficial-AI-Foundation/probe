---
auditor: ambiguity-auditor
date: 2026-10-06
repo: probe (hub)
kb: kb/ (own KB, resolved via common §1.1)
scope: branch la/kb-probe-aeneas-0.21.0 (issue #87), working-tree edits to
  kb/tools/probe-aeneas.md and the probe-aeneas section of
  kb/engineering/architecture.md. Per §2b the branch is not the default
  branch. The diff touches no property, schema or glossary text, so the
  audit used origin/main's properties.md unchanged. Code claims were checked
  against probe-aeneas branch feat/hub-0.5.0-adr-006 at f592887 (0.21.0,
  draft PR Beneficial-AI-Foundation/probe-aeneas#72).
status: 0 critical, 0 warnings, 1 info
---

## Critical

None.

## Warnings

None remaining. Two found during the audit and fixed in the same changeset:

- architecture.md step 5 used "raw merge" and "correspondence records"
  without links. Both now link to glossary.md#raw-staging-primitive and
  schema.md#correspondence-records-maps-to-mapped-from.
- architecture.md "External tools" named probe-lean's install mechanics
  (prebuilt download, else source build). ADR-005 assigns build/install
  orchestration to the probe repo. The line now says only that probe-lean is
  auto-installed and matched to the Lean project's toolchain.

## Info

- I1. The imported-symbol list in kb/tools/probe-aeneas.md is pinned to
  0.21.0 and will drift as probe-aeneas changes its imports. It matches the
  code today: `merge_atom_files_raw` (src/extract.rs:20),
  `endpoint_lookup_maps`, `Atom`, `InputProvenance`, `Mapping`,
  `MergedAtomEnvelope`, `Tool` (src/extract.rs:21-23), `load_atom_file`
  (src/translate.rs:674), `enrich_verification_status` (src/extract.rs:855).

## Checked, no finding

- Step 5 claims (authority validation on the raw path, no enrichment) agree
  with kb/tools/probe-merge.md lines 39 and 66 and the glossary entry
  "raw staging primitive".
- Step 6 `status-origin: "translation"` and step 7 single enrichment with
  `--skip-enrich` agree with kb/tools/probe-aeneas.md "Hub contracts" and
  with src/extract.rs:850-857.
- The probe-lean >= 0.16.0 threshold matches the ADR-006 gate and the
  probe-aeneas test at src/extract.rs:3282.
- probe-verus, probe-leanblueprint and probe-vcvio each declare a `probe`
  git dependency in Cargo.toml, so the removed "only probe" claim was false.
- No remaining KB reference to `merge_atom_maps` as the probe-aeneas entry
  point, and no remaining "auto-cloned" wording.
