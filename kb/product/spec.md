---
title: Product Specification
last-updated: 2026-03-19
status: draft
---

# Product Specification

## Problem

Formal verification projects span multiple languages (Rust, Lean 4, Verus) and tools (Aeneas transpiler, Verus verifier, Lean prover). Understanding the structure of these projects requires answering:

- What functions exist and what do they call? (call graph)
- Which functions have specifications? What do those specs say? (specification extraction)
- Which functions are verified? Which fail? Which use sorry/assume? (verification status)
- How do Rust definitions correspond to their Lean translations? (cross-language mapping)

No single tool answers all of these. Language-specific analyzers produce siloed data. There is no standard interchange format for combining them.

## Solution

The probe ecosystem extracts structured data from multi-language verification projects and merges it into a unified call graph with verification status, specifications, and cross-language mappings.

### Core capabilities

**1. Call graph extraction** — For each function in a project, identify what it calls and what calls it. Produce a dependency graph with accurate source locations.

- Rust projects: probe-rust (via rust-analyzer + SCIP)
- Verus projects: probe-verus (via verus-analyzer + SCIP)
- Lean 4 projects: probe-lean (via Lean environment introspection)

**2. Specification extraction** — Extract formal specifications (requires/ensures clauses for Verus; theorem statements for Lean) and attach them to their corresponding atoms.

- probe-verus: parses verus_syn AST for requires/ensures text, classifies with TOML taxonomy
- probe-lean: computes specs as reverse dependencies (theorems that reference a definition)

**3. Verification status** — Determine which definitions are verified, failed, or unverified.

- probe-verus: runs `cargo verus`, parses output, maps errors to functions
- probe-lean: detects sorry warnings in build output

**4. Cross-language merging** — Combine data from different languages into a single graph with cross-language dependency edges.

- probe merge: universal composition operator (any language pair)
- probe-aeneas: generates Rust↔Lean translation mappings for Aeneas-transpiled projects

**5. Validation** — Check that extracted data is consistent with source code.

- probe-extract-check: validates atoms against actual source files

### Output

All tools produce JSON conforming to [Schema 2.0](../engineering/schema.md). Every output file is self-describing via its metadata envelope.

Primary consumers:
- **scip-callgraph** — web UI for visualizing call graphs, verification status, and cross-language mappings
- **verilib-cli** — orchestration tool that coordinates probe runs

## Users

1. **Verification engineers** working on formally verified cryptographic implementations (e.g. curve25519-dalek). They need to see which functions are verified, which specs exist, and how Rust implementations correspond to Lean proofs.

2. **Project leads** tracking verification coverage across multi-language projects. They need aggregate verification status and spec coverage metrics.

3. **Tool developers** building on probe data (web UIs, CI pipelines, analysis tools). They consume Schema 2.0 JSON.

## Non-goals

- **IDE integration** — probe tools are batch analysis tools, not language servers
- **Incremental analysis** — full project re-analysis on each run (SCIP caching mitigates cost)
- **Code generation** — probes are read-only; they analyze but don't modify source
- **General-purpose Rust analysis** — focused on verification projects, not arbitrary Rust codebases
