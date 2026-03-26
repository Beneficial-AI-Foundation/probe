---
auditor: ambiguity-auditor
date: 2026-03-19
status: 5 critical, 12 warnings, 8 info
---

## Critical

### [C1] Undefined term: "molecule"
- **Location**: kb/tools/probe-lean.md, line 103-106
- **Issue**: The `viewify` subcommand description references "molecules" ("produces molecules", "molecule generation") and architecture.md line 148 references `molecule_all.json` in the `.verilib/views/` directory. The term "molecule" is never defined in kb/engineering/glossary.md. It appears to be a core domain concept (a filtered/projected view of atoms) but has no definition.
- **Recommendation**: Add a `## molecule` entry to glossary.md defining what a molecule is, how it relates to atoms, and what filtering criteria produce one.

### [C2] Undefined term: "scip-callgraph"
- **Location**: kb/engineering/architecture.md, line 123; kb/product/spec.md, line 56
- **Issue**: `scip-callgraph` is listed as a "web UI consumer" of all probe JSON output and as a "primary consumer" in the product spec. It is never defined in the glossary or described in any KB file. Its relationship to the probe ecosystem (is it part of probe? a separate project? what repo?) is unclear.
- **Recommendation**: Add a glossary entry for `scip-callgraph` and/or describe it in the product spec's consumer section.

### [C3] Undefined term: "verilib-cli"
- **Location**: kb/product/spec.md, line 57
- **Issue**: `verilib-cli` is listed as a primary consumer ("orchestration tool that coordinates probe runs") but is never defined in the glossary or described anywhere else in the KB. Its role, location, and relationship to the probe tools is completely opaque.
- **Recommendation**: Add a glossary entry for `verilib-cli` or at minimum add a description in the product spec.

### [C4] Contradiction: schema-version "2.0" vs "2.1" for probe-rust
- **Location**: kb/tools/probe-rust.md, line 106; kb/engineering/schema.md, line 41-42; kb/engineering/properties.md, line 15
- **Issue**: probe-rust.md states that probe-rust outputs `schema-version: "2.1"` (minor bump). However, properties.md P1 states validation checks that `schema-version` starts with `"2."`, and schema.md defines the current version as `"2.0"`. The schema.md versioning section (line 250-252) defines what minor bumps mean but does not document that 2.1 exists or what fields it adds. There is no mention of 2.1 anywhere except probe-rust.md. This creates ambiguity: is 2.1 a real version in use, or a hypothetical?
- **Recommendation**: Either (a) document schema version 2.1 in schema.md with its changelog, or (b) clarify in probe-rust.md that it currently outputs 2.0 and remove the 2.1 reference.

### [C5] Contradiction: probe-aeneas imports `merge_atom_maps` from different module paths
- **Location**: kb/tools/probe-aeneas.md, line 159; kb/tools/probe-merge.md, line 24
- **Issue**: probe-aeneas.md line 159 says it imports `probe::types::merge_atom_maps`. probe-merge.md line 24 says `merge_atom_maps()` lives in `src/commands/merge.rs`. If `merge_atom_maps` is in `commands/merge.rs`, the import path should be `probe::commands::merge::merge_atom_maps`, not `probe::types::merge_atom_maps`. One of these is wrong.
- **Recommendation**: Verify the actual code and correct the inaccurate reference.

## Warnings

### [W1] Vague requirement: "~95% accuracy"
- **Location**: kb/tools/probe-verus.md, line 41
- **Issue**: The verus_syn AST visitor is described as having "~95% accuracy for Verus-specific syntax". This is vague -- 95% of what? Files? Functions? Syntax constructs? What happens with the other ~5%? Is this a measured number or an estimate?
- **Recommendation**: Either quantify precisely (e.g., "correctly parses N of M test corpus functions") or remove the percentage and describe known limitations instead.

### [W2] Vague requirement: "tolerance: 10 lines"
- **Location**: kb/tools/probe-aeneas.md, line 81
- **Issue**: Strategy 3 (file + line-overlap) uses a "tolerance: 10 lines" for overlap checking. This magic number is stated but not justified. Why 10? What happens at the boundary? Is this configurable?
- **Recommendation**: Document the rationale for the 10-line tolerance and whether it is configurable.

### [W3] Undefined term: "Aeneas"
- **Location**: Multiple files (architecture.md, glossary.md, probe-aeneas.md, spec.md, ADR-003)
- **Issue**: "Aeneas" is used extensively (transpiler, transpilation, Aeneas-transpiled projects) but has no glossary entry. The glossary defines `functions.json` as "Aeneas-generated" but never defines Aeneas itself. A reader unfamiliar with the project would not know what Aeneas is.
- **Recommendation**: Add a glossary entry for Aeneas describing it as a Rust-to-Lean transpiler and linking to its upstream project.

### [W4] Undefined term: "Charon"
- **Location**: kb/tools/probe-rust.md, line 66 and 102; kb/tools/probe-aeneas.md, line 58-60
- **Issue**: Charon is referenced as providing "Charon-derived fully qualified names" and `rust-qualified-name`, and probe-rust lists it as an external dependency. But Charon has no glossary entry. Its relationship to Aeneas is not explained.
- **Recommendation**: Add a glossary entry for Charon.

### [W5] Undefined term: "RQN" / "rust-qualified-name"
- **Location**: kb/tools/probe-aeneas.md, line 60; kb/engineering/properties.md, line 149
- **Issue**: The abbreviation "RQN" is used in properties.md C6 ("normalized RQN") without expansion. `rust-qualified-name` is documented as an extension field but the abbreviation RQN is never formally defined.
- **Recommendation**: Expand "RQN" on first use or add it to the glossary as an abbreviation of "rust-qualified-name".

### [W6] Stale "planned" markers on files that now exist
- **Location**: kb/index.md, lines 36-47
- **Issue**: The root index.md marks all tool files, all decision files, and the product spec as `*(planned)*`. But all of these files now exist with substantive content. The `*(planned)*` markers are stale and misleading -- a reader would think these files don't exist yet.
- **Recommendation**: Remove all `*(planned)*` markers from index.md for files that exist.

### [W7] Undefined term: "views" / "viewify"
- **Location**: kb/engineering/architecture.md, line 148; kb/tools/probe-lean.md, lines 103-106
- **Issue**: The `.verilib/views/` directory and the `viewify` subcommand are mentioned but the concept of a "view" is never defined in the glossary. What distinguishes a view from raw probe output? What filtering is applied?
- **Recommendation**: Add a glossary entry for "view" explaining the concept and its relationship to atoms/molecules.

### [W8] Missing cross-reference: architecture.md references categorical-framework.md outside KB
- **Location**: kb/engineering/architecture.md, line 100
- **Issue**: The link `../../probe/docs/categorical-framework.md` points outside the KB directory to `probe/docs/`. While the file exists, this is a non-KB document referenced as if it were authoritative. The KB root index.md states "This KB is the source of truth." If the categorical framework is important enough to reference, key concepts should be captured in the KB.
- **Recommendation**: Either summarize the categorical framework in the KB (e.g., in architecture.md or a new file) or clearly mark this as an external reference to a non-normative document.

### [W9] Property P11 and P12 not referenced by any tool file for the claiming mechanism
- **Location**: kb/engineering/properties.md, lines 81-96
- **Issue**: Properties P11 (translation mapping is 1-to-1) and P12 (translation strategy priority) are referenced by probe-aeneas.md, which is good. However, the enforcement mechanism (`matched_rust` and `matched_lean` HashSets in `translate.rs`) is described identically in both properties.md and probe-aeneas.md. If either changes, the other becomes stale.
- **Recommendation**: Have properties.md state the invariant only, and probe-aeneas.md describe the implementation. Currently both describe both.

### [W10] Undefined term: "latex" as a language
- **Location**: kb/engineering/schema.md, lines 47, 113, 260
- **Issue**: `"latex"` appears as a valid `source.language` value and has a package versioning strategy entry. But no probe tool handles LaTeX, no tool file mentions it, and it has no glossary entry. This creates confusion about whether LaTeX support exists, is planned, or is vestigial.
- **Recommendation**: Either document LaTeX support plans or remove it from schema.md to avoid confusion.

### [W11] Vague phrase: "lower priority" for known inefficiencies
- **Location**: kb/tools/probe-verus.md, lines 143-150
- **Issue**: Known inefficiencies are listed with the note "Remaining optimizations are lower priority." This is vague -- lower than what? Is there a prioritized backlog? Are these acceptable permanent states or planned work?
- **Recommendation**: Either link to tracked issues or state explicitly whether these are accepted trade-offs or planned improvements.

### [W12] Undefined term: "extract-check validator"
- **Location**: kb/engineering/architecture.md, line 16
- **Issue**: Architecture.md line 16 calls it "extract-check validator" in the component listing, but the glossary defines it as "probe-extract-check". The hyphenation and naming is inconsistent across files.
- **Recommendation**: Standardize on one name (recommend `probe-extract-check` per glossary) and use it consistently.

## Info

### [I1] Missing cross-reference: glossary "doctrine" links to docs outside KB
- **Location**: kb/engineering/glossary.md, line 73
- **Issue**: The glossary entry for "doctrine" references `probe/docs/categorical-framework.md` but uses the text "see `probe/docs/categorical-framework.md`" rather than a clickable relative link. Minor formatting issue.
- **Recommendation**: Either convert to a proper markdown link or note it as an external document.

### [I2] Redundant description of merge algorithm across three files
- **Location**: kb/engineering/schema.md (lines 197-224), kb/engineering/properties.md (P6, P7), kb/tools/probe-merge.md (lines 27-55)
- **Issue**: The merge algorithm (first-wins with stub replacement, last-wins for specs/proofs) is described in three places. While cross-references exist, each file restates the rules in slightly different formats. This increases maintenance burden.
- **Recommendation**: Consider designating one file as the canonical description and having others reference it. schema.md could define the behavior, properties.md the invariants, and probe-merge.md the implementation details.

### [I3] Product spec lists "General-purpose Rust analysis" as non-goal
- **Location**: kb/product/spec.md, line 72
- **Issue**: The non-goal "focused on verification projects, not arbitrary Rust codebases" seems to conflict with probe-rust's role of extracting call graphs from "standard Rust projects" (architecture.md line 37, probe-rust.md line 10). Probe-rust works on any Rust project, not just verification projects.
- **Recommendation**: Clarify whether probe-rust is intended for arbitrary Rust projects or only those involved in verification workflows.

### [I4] No KB file for `config.json` format
- **Location**: kb/engineering/architecture.md, line 151
- **Issue**: `.verilib/config.json` is mentioned as "User/project configuration" and probe-lean.md references config-based filtering (`is-hidden`, `is-ignored`, `relevant-crate`, `extraction-artifact-suffixes`). But the config.json format is never specified in any KB file.
- **Recommendation**: Document the config.json schema in either schema.md or a new file when the config stabilizes.

### [I5] `probe-lean/atoms` schema value is not registered
- **Location**: kb/engineering/schema.md, lines 75-83
- **Issue**: The registered schema values list includes `probe-lean/extract` and `probe-lean/viewify` but not `probe-lean/atoms`. The merged envelope example on line 65 shows `"schema": "probe-lean/atoms"` as an input source. If `probe-lean/atoms` is a valid schema value, it should be registered. If probe-lean only produces `probe-lean/extract`, the example should be updated.
- **Recommendation**: Clarify whether `probe-lean/atoms` is a valid schema value and update accordingly.

### [I6] `callee-crates` and `list-functions` output format undocumented
- **Location**: kb/tools/probe-rust.md, lines 69 and 78; kb/tools/probe-verus.md, lines 100-101
- **Issue**: Both `callee-crates` and `list-functions` subcommands are documented as producing "No envelope (raw JSON)". Their output format is not specified anywhere in the KB. These are secondary commands, so this is low priority, but it creates a coverage gap.
- **Recommendation**: Note these as non-Schema-2.0 outputs that are outside the interchange specification scope.

### [I7] "Ralph Loop" glossary entry references kb/index.md anchor that doesn't exist
- **Location**: kb/engineering/glossary.md, line 93
- **Issue**: The Ralph Loop entry links to `../index.md#reports` but the anchor `#reports` in index.md resolves to the "Reports" section heading. The link text says "[KB auditors](../index.md#reports)" but that section just lists report files, not auditors. The connection is tenuous.
- **Recommendation**: Either link to a more specific description of auditor skills or drop the link.

### [I8] `stubify` subcommand in probe-verus is minimally documented
- **Location**: kb/tools/probe-verus.md, lines 109-110
- **Issue**: The `stubify` subcommand is described only as "Convert `.md` files with YAML frontmatter to JSON stubs." This is the least documented subcommand -- no flags, no use case, no relationship to the rest of the ecosystem. It's unclear how this relates to the stub concept in the glossary (which is about atoms with empty code-path).
- **Recommendation**: Either expand the description or note it as a utility command outside the core extraction pipeline.
