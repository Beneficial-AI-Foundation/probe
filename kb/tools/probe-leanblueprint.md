---
title: "Tool: probe-leanblueprint"
last-updated: 2026-07-21
status: draft
---

# probe-leanblueprint

**Directory**: `baif/probe-leanblueprint/`
**Role**: Enrich `probe-lean/extract` atoms with Lean **blueprint** progress metadata (a human-authored roadmap plus a two-axis statement/proof status), so Lean projects get meaningful verification-progress stats rather than a bare theorem count.
**Subcommands**: `extract`

## What this tool is (and isn't)

probe-leanblueprint is an **enricher**, analogous to [probe-aeneas](probe-aeneas.md): it consumes another probe's output (probe-lean atoms) as its atom "spine" and re-emits a Schema 2.0 envelope with extra fields. It:

- **Consumes** a `probe-lean/extract` atom base (the code call graph + machine `verification-status`)
- **Reads** the blueprint via one of two adapters (Verso manifest or Massot LaTeX)
- **Joins** blueprint nodes to atoms by Lean declaration name
- **Enriches** matched atoms with `blueprint-*` extension fields, and **synthesizes** planned atoms for blueprint nodes with no Lean binding
- **Re-emits** a `probe-leanblueprint/extract` envelope plus a first-class `probe-leanblueprint/summary` sidecar

It does **not** re-implement blueprint parsing, does not touch probe-lean (which stays blueprint-unaware), and does **not** override the machine `verification-status` — the blueprint proof axis is additive (see [P26](../engineering/properties.md#p26-blueprint-status-is-additive-machine-verification-status-stays-authoritative)).

Why not extend probe-lean? probe-lean is written in Lean and is deliberately generic across all Lean projects; blueprint is a complementary, doc-authoritative layer that only some projects have. Keeping it separate avoids growing probe-lean into a project-type-specific monster and keeps downstream schema consumers working for free via extension preservation ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).

## Two blueprint ecosystems

| Ecosystem | Source of truth | How we read it | Status authority |
|-----------|-----------------|----------------|------------------|
| **Verso Blueprint** (`versoBlueprint`, Lean-native; used by baif projects) | `blueprint-manifest.json` (rendered by the Verso docs build) | Parse the JSON directly (Rust) | code-derived (`blueprint-status-source: code-derived`) |
| **Patrick Massot `leanblueprint`** (LaTeX/plasTeX; the Mathlib-community standard) | `blueprint/src/web.tex` | Bundled headless plasTeX emitter reusing leanblueprint's own parser | human-declared (`blueprint-status-source: declared`) |

## Two-axis status vocabulary (canonical)

Both ecosystems track progress on two independent axes. probe-leanblueprint normalizes every source status into a single canonical vocabulary (`src/model.rs`):

- **statement axis** — is the *statement* formalized in Lean? `none` (informal only) < `blocked` (prerequisites not ready) < `ready` (ready to formalize) < `formalized`.
- **proof axis** — is the *proof* complete (sorry-free)? `none` < `ready` < `proved` (local, sorry-free) < `fully-proved` (proved + all ancestors).

### Mapping table

| Source | statement axis | proof axis |
|--------|----------------|------------|
| Verso `statementStatus` | `formalized`/`ready`/`blocked`/`none` → direct | — |
| Verso `proofStatus` | — | `formalizedWithAncestors`→`fully-proved`, `formalized`→`proved`, `ready`→`ready`, `none`→`none` |
| Massot (`\leanok`/`\mathlibok`/`\notready`/computed `can_state`) | `leanok`→`formalized`, `can_state`→`ready`, `notready`→`blocked`, else `none` | `proved`+`fully_proved`→`fully-proved`, `proved`→`proved`, `can_prove`→`ready`, else `none` |

Note: leanblueprint's `fully_proved` counts definitions as vacuously done, so the strongest proof state is gated on `proved` to avoid over-claiming on `definition` nodes.

## Extract pipeline

The `extract` command (`src/main.rs` → `src/enrich.rs`):

```
project → (probe-lean extract | --lean) → atom base
        → adapter (Verso manifest | Massot plasTeX) → BlueprintModel
        → join by probe:<canonical> → enrich atoms + synthesize planned atoms
        → propagate::enrich_verification_status (idempotent)
        → probe-leanblueprint/extract envelope + probe-leanblueprint/summary sidecar
```

1. **Resolve adapter** — explicit `--adapter`, else auto-detect: `--verso-manifest`/`versoBlueprint` in the lakefile → Verso; `--blueprint-src`/`blueprint/src/web.tex` → Massot.
2. **Load atom base** — `--lean <probe-lean.json>` if given, else run `probe-lean extract <project>` (a single incremental compile).
3. **Build the blueprint model** — Verso adapter parses `blueprint-manifest.json`; Massot adapter shells out to the bundled `scripts/blueprint_emit.py`. That script is **embedded into the binary** (`include_str!`) and materialized to a temp file at runtime, so a `cargo install`ed executable is self-contained; an explicit `--emitter` or a copy shipped next to the executable takes precedence.
4. **Join + enrich** — match blueprint nodes to atoms by `probe:` + Lean declaration name; attach `blueprint-*` fields; synthesize planned atoms; compute `blueprint-status-mismatch`.
5. **Propagate** — reuse `probe::commands::propagate::enrich_verification_status` (idempotent; machine status stays authoritative).
6. **Emit** — the enriched atom envelope and the summary sidecar (an aggregate over the blueprint nodes).

### Single-build guarantee

Lake builds are incremental and the code libraries are shared between the code target and the Verso docs/blueprint target. Total cost is **one full compile**: rendering the Verso docs (which writes `blueprint-manifest.json`) compiles the libs; the subsequent `probe-lean extract` is an incremental no-op on the already-compiled libs. The Massot/LaTeX path needs no Lean docs build at all — plasTeX only parses LaTeX.

## The join

Both ecosystems bind a blueprint node to Lean declarations by **user-facing fully-qualified name**: Massot via `\lean{Foo.bar}`, Verso via `codeData.external.decls[].canonical`. probe-lean keys atoms as `probe:` + that same user-facing name (`probeRef`), so the join is `probe:<canonical>`.

Edge-case rules (`src/enrich.rs`):

- **Node binds multiple decls** — attach the node to every present atom.
- **Same-decl collision** — if two blueprint nodes bind the same present atom, the later node wins (keep-last); a warning is logged and the case is counted in the summary `collisions` total.
- **Decl-missing authority** — probe-lean atom membership is the **sole** authority on whether a bound declaration is present. (Verso also emits its own per-decl `present` / node `missingExternalDecl` hints; these are intentionally **not** consumed — they coincide with atom membership on real data, and atom membership is the tool's premise that probe-lean is the code spine.)
  - **All bound decls absent** — emit a synthetic planned node flagged `blueprint-decl-missing: true` (counted in `decl-missing`) rather than fabricating a code atom.
  - **Some bound decls absent (partial miss)** — the node stays bound (attached to its present atoms); the absent names are recorded on the present atom(s) as `blueprint-missing-decls: [...]` and counted in the `partial-missing` total. This keeps the bound / planned-only / decl-missing partition clean.
- **Node has no Lean binding** (planned-only) — synthesize a `probe:blueprint:<label>` atom with `language: "blueprint"`, `kind: "blueprint-<def|theorem>"`, and a non-empty `code-path` marker (`"blueprint"`) so [P3](../engineering/properties.md#p3-stub-detection-is-structural) stub detection does not misclassify it.
- **Blueprint `uses` edges stay extension-only** (`blueprint-statement-uses`/`blueprint-proof-uses`); they are the informal roadmap graph and are never merged into an atom's `dependencies` (the code call graph).

## Status reconciliation

The statement axis is blueprint-exclusive (no conflict with any machine signal). Only the proof axis can disagree with probe-lean's machine sorry-truth. Resolution (see [P26](../engineering/properties.md#p26-blueprint-status-is-additive-machine-verification-status-stays-authoritative)):

- `verification-status` stays probe-lean's machine value — a `sorry` can never render green, consistent with every other probe.
- The blueprint's declared proof status is kept in the separate `blueprint-proof-status` field.
- When the blueprint claims a proof is done (`proved`/`fully-proved`) but the machine status is `unverified`/`failed`, `blueprint-status-mismatch` is set (e.g. `"claims-proved-but-unverified"`).

This flag is most valuable for the Massot path, where `\leanok` is a human claim (leanblueprint only `checkdecls` that a declaration *exists*, not that it is sorry-free). Verso's `proofStatus` is code-derived and usually agrees.

## Outputs

### `probe-leanblueprint/extract` (atoms category)

A Schema 2.0 atom envelope. Detected as the **Atoms** category by the hub (via the `*/extract` suffix in `detect_category()`), so `probe merge`/`project` accept it and preserve the blueprint extensions ([P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)).

### `probe-leanblueprint/summary` (sidecar)

A first-class, two-axis progress report that **aggregates over** the blueprint nodes (it is not keyed per node) — this is where the meaningful blueprint stats live (the hub's `probe summary` is Rust/Verus-centric). Contains statement/proof histograms overall and by kind (definition vs theorem), totals (nodes, with-lean-decl, planned-only, decl-missing, partial-missing, collisions, mismatches), a headline "theorems fully proved" fraction, and a `by-chapter` breakdown (per-chapter node count, two-axis histograms, and theorems-fully-proved / theorems-total). Not an atoms-category schema, so it is never merged.

### Displaying stats

`scripts/blueprint_stats.py <extract.json>` renders a readable report (headline, statement/proof tables, per-chapter breakdown, and any mismatches / missing decls) directly from a `probe-leanblueprint/extract` file. It recomputes everything from the `blueprint-*` extension fields, so it doubles as an independent cross-check of the summary sidecar and needs no Python blueprint dependencies. Pass `--json` for a machine-readable form.

## Blueprint extension fields

Attached (flattened per [P10](../engineering/properties.md#p10-extensions-are-preserved-through-merge)) to enriched and synthetic atoms:

| Field | Description |
|-------|-------------|
| `blueprint-label` | Blueprint node label |
| `blueprint-kind` | Blueprint node kind (`definition`/`theorem`); lets consumers classify bound atoms whose atom `kind` is the Lean kind |
| `blueprint-statement-status` | Canonical statement axis (`none`/`blocked`/`ready`/`formalized`) |
| `blueprint-proof-status` | Canonical proof axis (`none`/`ready`/`proved`/`fully-proved`) |
| `blueprint-status-source` | `code-derived` (Verso) or `declared` (Massot) |
| `blueprint-group` | Sub-construction grouping label, Verso `parent` (optional) |
| `blueprint-chapter` | Chapter the node belongs to; one Verso manifest = one chapter (optional) |
| `blueprint-title` | Display title, e.g. "Theorem 2.3" (optional) |
| `blueprint-discussion` | GitHub discussion issue number (optional) |
| `blueprint-statement-uses` | Code-names used by the statement (resolved from blueprint labels) |
| `blueprint-proof-uses` | Code-names used by the proof |
| `blueprint-status-mismatch` | Set when the blueprint over-claims vs the machine status (optional) |
| `blueprint-decl-missing` | `true` when **all** bound Lean decls are absent from the atom set (synthetic planned node; optional) |
| `blueprint-missing-decls` | For a bound node, the subset of `\lean{...}` decls absent from the atom set (partial miss); recorded on the present atom(s) (optional) |

## CLI

```
probe-leanblueprint extract <PROJECT>
    [--lean <probe-lean.json>]
    [--adapter auto|verso|massot]
    [--verso-manifest <file|dir>]
    [--blueprint-src <web.tex|dir>]
    [--python <interp>] [--emitter <blueprint_emit.py>]
    [-o <extract.json>] [--summary-output <summary.json>]
```

Defaults write to `<project>/.verilib/probes/leanblueprint_<package>[_<version>].json` and `..._summary.json`.

## Key source files

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI, adapter auto-detection, orchestration, output |
| `src/model.rs` | Normalized `BlueprintModel`/`BlueprintNode`, canonical status enums, extension field set |
| `src/adapters/verso.rs` | Verso `blueprint-manifest.json` → `BlueprintModel` |
| `src/adapters/massot.rs` | Shell out to the plasTeX emitter → `BlueprintModel` |
| `src/enrich.rs` | Join, synthesis, mismatch, summary computation |
| `src/emit.rs` | Envelope + summary sidecar construction |
| `src/emitter.rs` | Embeds `blueprint_emit.py` (`include_str!`) and resolves/materializes it at runtime |
| `scripts/blueprint_emit.py` | Bundled headless plasTeX emitter (reuses leanblueprint's parser); embedded into the binary |
| `scripts/blueprint_stats.py` | Display two-axis + per-chapter stats from an `extract.json` |

## External tool dependencies

| Tool | Required | Notes |
|------|----------|-------|
| probe-lean | yes (unless `--lean` given) | Produces the atom base; single incremental build |
| Verso docs build (`lake`) | yes for Verso (unless a fresh manifest / `--verso-manifest` exists) | Writes `blueprint-manifest.json` |
| python3 + plasTeX + leanblueprint | yes for Massot | `pip install leanblueprint`; needs graphviz/libgraphviz-dev for pygraphviz. No Lean build needed. |

## Dependency on the probe crate

probe-leanblueprint depends on the `probe` hub crate. Uses:
- `probe::types::{Atom, AtomEnvelope, Source, Tool, CodeText, load_atom_file}`
- `probe::commands::propagate::enrich_verification_status`

Rationale and alternatives considered: [ADR-004](../decisions/004-probe-leanblueprint.md). Broader Lean stats context: `probe/docs/lean-stats-brainstorm.md` (non-normative).
