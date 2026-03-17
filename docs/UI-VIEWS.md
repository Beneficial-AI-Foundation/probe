# UI Views for Probe Data

Version: draft
Date: 2026-03-17
Parent document: [CONSUMER_GUIDE.md](CONSUMER_GUIDE.md)

This document describes the views a UI should implement to let users
explore probe atom data. It covers language toggles, the three
visualization layouts (call graph, file map, crate map), and
filtering by atom metadata. A reference implementation of these views
exists in the [scip-callgraph](https://github.com/Beneficial-AI-Foundation/scip-callgraph)
interactive viewer, deployed at
[beneficial-ai-foundation.github.io/scip-callgraph](https://beneficial-ai-foundation.github.io/scip-callgraph/).

## Language toggles

Every atom carries a `language` field (`"rust"`, `"lean"`, or `"latex"`).
A merged atom file (schema `probe/merged-atoms`) contains atoms from
multiple languages in the same `data` dictionary. This makes language
filtering trivial: partition nodes by `language` and expose a toggle.

### Rust / Lean toggle

The primary toggle is **Rust** vs **Lean**. When both languages are
present (e.g. after merging probe-verus and probe-lean outputs, or from
a probe-aeneas extraction), the UI should offer:

| Mode | What is shown |
|------|---------------|
| **Rust view** | Only atoms where `language == "rust"`. Dependency edges are restricted to Rust-to-Rust. |
| **Lean view** | Only atoms where `language == "lean"`. Dependency edges are restricted to Lean-to-Lean. |
| **Combined view** | All atoms. Cross-language edges (created by `probe merge --translations`) are visible. |

Switching between these modes is a client-side filter on the loaded
JSON -- no re-fetching is needed. The toggle should be prominent (e.g.
a segmented control in the toolbar) because the two views represent
fundamentally different perspectives on the same codebase.

### What changes between views

The Rust and Lean views are not just subsets -- they surface different
atom metadata:

| Aspect | Rust view | Lean view |
|--------|-----------|-----------|
| **Kind values** | `exec`, `proof`, `spec` (Verus) | `def`, `theorem`, `abbrev`, `class`, `structure`, `inductive`, `instance`, `axiom`, `opaque` |
| **Code-name style** | `probe:crate/version/module/Type#method()` | `probe:Namespace.Name` |
| **Spec display** | `primary-spec` is inline text (requires/ensures) | `primary-spec` is a code-name pointing to a theorem |
| **File paths** | `src/scalar.rs` | `Mathlib/Data/Nat.lean` |
| **Module grouping** | Rust module paths (`backend/serial/u64/field`) | Lean namespaces (`Curve25519Dalek.Backend.Serial`) |

A UI that understands these differences can adapt its rendering:
display `requires`/`ensures` blocks inline for Rust atoms, but show a
link to the specification theorem for Lean atoms.

### Cross-language links

In combined view, atoms from different languages may be connected by
translation edges (created via a
[translations file](translations-spec.md)). These edges deserve
distinct styling (e.g. dashed lines, a different color) to distinguish
them from intra-language dependency edges.

When a Rust atom has a `translation-name` field (from probe-aeneas),
the UI can show a "View Lean counterpart" action that jumps to the
corresponding Lean atom.

## Visualization views

The three views below are ordered from finest to coarsest granularity.
All three operate on the same underlying graph (nodes = atoms,
edges = dependencies). They differ in layout and grouping.

### Call graph view

A force-directed (or hierarchical) graph where each node is a single
atom and each edge is a dependency.

**Purpose:** Function-level exploration. Answer questions like "what
does `batch_invert` call?" or "who calls `reduce`?"

**Key features:**
- **Source / sink filtering.** Enter a function name as source to see
  its callees, as sink to see its callers, or both to see paths between
  two functions.
- **Depth control.** Limit the graph to N hops from the selected
  node(s) to avoid overwhelming the display.
- **Verification coloring.** Color nodes by `verification-status`:
  green = verified, red = failed, grey = unverified, blue = unknown.
- **Kind badges.** Show the `kind` value (exec/proof/spec or
  def/theorem/etc.) as a badge or icon on each node.

**Example (scip-callgraph):**

[`scalar::batch_invert` at depth 1](https://beneficial-ai-foundation.github.io/scip-callgraph/?source=scalar%3A%3Abatch_invert&depth=1)
shows the immediate callees of the `batch_invert` function in
curve25519-dalek. Clicking a callee expands it to show *its* callees.

**Mapping from probe atoms:**

| Graph element | Probe field |
|---------------|-------------|
| Node ID | dictionary key (code-name) |
| Node label | `display-name` |
| Directed edge A → B | B's code-name appears in A's `dependencies` |
| Node color | `verification-status` (if present) |
| Node shape/badge | `kind` |
| Tooltip: file | `code-path` : `code-text.lines-start` |

### File map view

A hierarchical layout that groups atoms by their source file.
Each file is a compound box containing its atoms as inner nodes.
Edges between atoms in different files are drawn between the
file groups.

**Purpose:** Understand module-level structure. Answer questions like
"what functions are in `src/scalar.rs`?" or "which files depend on
`src/field.rs`?"

**Key features:**
- **File grouping.** Group atoms by `code-path`. Atoms with the same
  `code-path` are placed in the same container.
- **File-level edges.** Aggregate atom-level edges into file-level
  edges (with counts) for a higher-level overview.
- **Expandable groups.** Click a file to expand or collapse its atoms.

**Mapping from probe atoms:**

| UI element | Probe field |
|------------|-------------|
| File group label | `code-path` |
| Atom within group | `display-name`, `kind` |
| Line range | `code-text.lines-start` -- `code-text.lines-end` |

**Stub handling:** Atoms with `code-path == ""` are stubs (external
dependencies). Group them separately, e.g. under an "External" box,
or omit them and show them only as edge targets.

### Crate map view

A high-level overview where each node is a crate (Rust) or package
(Lean) and edges represent cross-crate function calls.

**Purpose:** Understand inter-crate dependencies at a glance. Answer
questions like "which crates does `curve25519-dalek` depend on?" or
"how many functions does crate A call in crate B?"

**Key features:**
- **Crate-level nodes.** Aggregate atoms by the crate/package portion
  of their code-name. For Rust, the crate name and version are
  embedded in the code-name URI
  (`probe:<crate>/<version>/...`). For Lean, use `source.package`
  from the envelope (or the top-level namespace from the code-name).
- **Weighted edges.** Edge thickness proportional to the number of
  cross-crate dependency edges between the two crates.
- **Boundary selection.** Select two crates to see the interface
  between them: which functions in crate A are called by crate B.
  The [scip-callgraph crate boundary feature](https://beneficial-ai-foundation.github.io/scip-callgraph/)
  implements this with a source/target crate dropdown.
- **Drill-down.** Double-click a crate to switch to call graph view
  filtered to that crate's atoms.

**Mapping from probe atoms:**

| UI element | Probe field |
|------------|-------------|
| Crate node label | Crate name extracted from code-name URI (Rust) or `source.package` (Lean) |
| Function count per crate | Count of non-stub atoms in that crate |
| File count per crate | Count of distinct `code-path` values in that crate |
| Cross-crate edge weight | Count of dependency edges where source and target are in different crates |

## Filtering dimensions

Beyond the language toggle and view selection, the atom schema supports
several filtering axes that a UI should expose:

### By declaration kind

Filter nodes by `kind`. Useful for isolating executable code from
specifications and proofs:

| Filter | Effect |
|--------|--------|
| Exec only | Show only `exec` (Rust) or `def` (Lean) atoms -- the runnable code |
| Proof only | Show `proof` (Rust) or `theorem` (Lean) atoms |
| Spec only | Show `spec` (Rust) atoms |
| All | No filtering |

### By verification status

When `verification-status` is present, filter by verification outcome:

| Filter | Atoms shown |
|--------|-------------|
| Verified | Green nodes only |
| Failed | Red nodes only |
| Unverified | Grey nodes only |
| All | No filtering |

### By module

Filter or group atoms by `code-module`. Modules are hierarchical
(Rust uses `/`, Lean uses `.`), so a tree-based filter (expandable
module tree with checkboxes) works well.

### Excluding stubs

Stubs (`code-path == ""`) are external dependencies without source
code. The UI should offer an option to hide them (showing only atoms
with local source) or show them dimmed.

### Excluding disabled atoms

When `is-disabled` is `true`, the atom is out of scope. These can be
hidden by default and shown on request.

## URL-driven state

The scip-callgraph viewer demonstrates a useful pattern: encode the
current view state in the URL query string so that views are
shareable and bookmarkable. Key parameters:

| Parameter | Purpose | Example |
|-----------|---------|---------|
| `source` | Source function for callee exploration | `source=scalar%3A%3Abatch_invert` |
| `sink` | Sink function for caller exploration | `sink=reduce` |
| `depth` | Hop limit from source/sink | `depth=1` |
| `json` | URL to a graph JSON file | `json=https://example.com/graph.json` |

For probe-based viewers, additional parameters are useful:

| Parameter | Purpose | Example |
|-----------|---------|---------|
| `lang` | Active language filter | `lang=rust`, `lang=lean`, `lang=all` |
| `view` | Active visualization | `view=callgraph`, `view=filemap`, `view=cratemap` |
| `kind` | Kind filter | `kind=exec`, `kind=theorem` |
| `status` | Verification status filter | `status=verified` |

## Worked example: `scalar::batch_invert`

To illustrate how the views compose, consider the `batch_invert`
function from curve25519-dalek:

### Call graph view (Rust, depth 1)

URL: [`?source=scalar::batch_invert&depth=1`](https://beneficial-ai-foundation.github.io/scip-callgraph/?source=scalar%3A%3Abatch_invert&depth=1)

Shows `batch_invert` at the center with its immediate callees
(functions it calls). The viewer renders each callee as a node
colored by verification status.

### Call graph view (Rust, depth 2)

URL: [`?source=scalar::batch_invert&depth=2`](https://beneficial-ai-foundation.github.io/scip-callgraph/?source=scalar%3A%3Abatch_invert&depth=2)

Expands one more level: callees of callees. The graph grows but
remains navigable with the depth slider.

### File map view

Switch to file map to see which files `batch_invert`'s dependencies
span. The function lives in `src/scalar.rs`; its callees may be in
`src/field.rs`, `src/backend/serial/u64/field.rs`, etc. The file map
makes this cross-file structure visible at a glance.

### Lean view (combined data)

If the loaded JSON is a merged file containing both Rust and Lean
atoms (e.g. from probe-aeneas), toggling to Lean view shows the Lean
counterparts of the same functions -- `batch_invert` becomes
`Curve25519Dalek.Scalar.batch_invert` (or whatever the Lean name is).
Cross-language translation edges connect the two.

## Implementation notes

### Data loading

The viewer should accept probe JSON files directly (either single-tool
or merged envelopes). Read the `schema` field to determine the file
type and adapt the UI accordingly:

- If `schema` is `probe-*/extract` or `probe-*/atoms`: single-language
  file, hide the language toggle.
- If `schema` is `probe/merged-atoms` or `probe-aeneas/extract`:
  multi-language file, show the language toggle.
- The `data` dictionary is the graph. Iterate keys for nodes, follow
  `dependencies` for edges.

### Stub resolution

If the user loads multiple files, the UI can resolve stubs
client-side: when a stub code-name matches a real atom in another
loaded file, replace the stub with the real atom and gain its metadata
(file, lines, kind, etc.).

### Source code linking

When `code-path` and `code-text` are present, the UI can link to the
source. If the envelope's `source.repo` is a GitHub URL, construct a
direct link:

```
{source.repo}/blob/{source.commit}/{code-path}#L{lines-start}-L{lines-end}
```

This gives every node a "View source" action.
