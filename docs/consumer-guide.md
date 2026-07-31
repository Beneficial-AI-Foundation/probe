# Probe Consumer Guide

A reference for consuming the JSON outputs produced by the probe tool
family.

## The probe tools

| Tool | Language | What it extracts | Repo |
|------|----------|-----------------|------|
| **probe-rust** | Rust | Call graph atoms from SCIP index | [probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust) |
| **probe-lean** | Lean 4 | Call graph atoms + sorry detection + specs | [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean) |
| **probe-leanblueprint** | Lean 4 | probe-lean atoms enriched with blueprint progress (statement/proof status) | [probe-leanblueprint](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint) |
| **probe-verus** | Rust/Verus | Call graph + specs + verification status | [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus) |
| **probe-aeneas** | Rust + Lean | Cross-language merged graph (Aeneas projects) | [probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas) |

They all produce JSON files conforming to the Schema 3.0 envelope format
defined in [`probe/docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe/blob/main/docs/SCHEMA.md).

## Running extract

The typical command for each tool is:

```bash
probe-rust          extract <project_path>
probe-lean          extract <project_path>
probe-leanblueprint extract <project_path>
probe-verus         extract <project_path>
probe-aeneas        extract <project_path>
```

For probe-leanblueprint, the Lean project must ship a blueprint — a Verso
`versoBlueprint` dependency or a Massot `blueprint/src/web.tex` tree; the Massot
path additionally needs the Python emitter dependencies. It runs `probe-lean
extract` for the atom base and enriches it with blueprint progress.

For probe-aeneas, the project path must be an Aeneas project directory
containing `aeneas-config.yml`. The tool reads `crate.dir` from the
config to locate the Rust crate and uses the project root as the Lean
project. If you already have extracted JSON files, you can use the
advanced flags instead:

```bash
probe-aeneas extract --rust <rust_json> --lean <lean_json> --lean-project <lean_project_path>
```

Output lands in `.verilib/probes/` by default. Each tool's repo README
documents additional flags for caching, auto-install, output paths, and
other options.

## Output format

Every output file is a JSON object with this envelope:

```json
{
  "schema": "probe-<tool>/extract",
  "schema-version": "3.0",
  "tool": {
    "name": "probe-<tool>",
    "version": "0.1.0",
    "command": "extract"
  },
  "source": {
    "repo": "https://github.com/...",
    "commit": "abc123...",
    "language": "rust",
    "package": "my-crate",
    "package-version": "1.0.0"
  },
  "timestamp": "2026-03-17T12:00:00Z",
  "data": { ... }
}
```

For merged files (probe-aeneas), `source` is replaced by
`inputs` — an array of provenance entries, one per input file.

### The `data` object

A dictionary keyed by **code-name** (a URI like
`probe:curve25519-dalek/4.1.3/scalar/Scalar#add()`). Each value is an
atom:

```json
{
  "display-name": "add",
  "dependencies": ["probe:curve25519-dalek/4.1.3/field/reduce()"],
  "code-module": "scalar",
  "code-path": "src/scalar.rs",
  "code-text": { "lines-start": 42, "lines-end": 67 },
  "kind": "exec",
  "language": "rust"
}
```

**Core fields** (present on every atom from every tool):

| Field | Type | Description |
|-------|------|-------------|
| `display-name` | string | Human-readable name |
| `dependencies` | array | Code-names of referenced atoms |
| `code-module` | string | Module/namespace path |
| `code-path` | string | Relative file path (empty for stubs); can be used to reconstruct the project's folder structure |
| `code-text` | object | `{ "lines-start": N, "lines-end": N }` (both 0 for stubs) |
| `kind` | string | Declaration kind (language-specific) |
| `language` | string | Source language of the atom; see [SCHEMA.md](https://github.com/Beneficial-AI-Foundation/probe/blob/main/docs/SCHEMA.md#language-string) for the value set |

**Common optional fields.** Beyond the core fields, atoms may carry
`primary-spec`, `verification-status`, `trusted-reason`, `untracked`, `specs`,
and tool-specific extension fields (e.g. probe-leanblueprint's `blueprint-*`
progress fields). The authoritative list — field names, types, value sets, and
which tool populates each — is
[SCHEMA.md § Common Optional Fields](https://github.com/Beneficial-AI-Foundation/probe/blob/main/docs/SCHEMA.md#common-optional-fields),
with per-tool detail in each tool's own `SCHEMA.md`.

### Kind values

`kind` values are language-specific. See
[SCHEMA.md § Kind Values](https://github.com/Beneficial-AI-Foundation/probe/blob/main/docs/SCHEMA.md#kind-values)
for the full set per language.

### Stubs

An atom with `code-path: ""` and `code-text: { "lines-start": 0, "lines-end": 0 }`
is a **stub** — a dependency reference without local source code. Stubs
represent external crate functions or library calls.

## Example files

All examples use the **curve25519-dalek** ecosystem as the reference
project.

| Repo | File | Schema |
|------|------|--------|
| [probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust) | [`examples/rust_curve25519-dalek_4.1.3.json`](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/examples/rust_curve25519-dalek_4.1.3.json) | `probe-rust/extract` |
| [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean) | [`examples/lean_Curve25519Dalek_0.1.0.json`](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/examples/lean_Curve25519Dalek_0.1.0.json) | `probe-lean/extract` |
| [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus) | [`examples/verus_curve25519-dalek_4.1.3.json`](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/examples/verus_curve25519-dalek_4.1.3.json) | `probe-verus/extract` |
| [probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas) | [`examples/aeneas_curve25519-dalek_4.1.3.json`](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/examples/aeneas_curve25519-dalek_4.1.3.json) | `probe-aeneas/extract` |

probe-leanblueprint has no standalone example JSON — it enriches a `probe-lean`
extract. Its [`examples/`](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/tree/main/examples)
directory holds runnable blueprint projects instead.

## Documentation

For the full doc index -- per-tool `USAGE.md`/`SCHEMA.md` across the ecosystem and the
other reference docs in this repo -- see the
[probe README](https://github.com/Beneficial-AI-Foundation/probe#documentation).

## Working with the data

### Reading the call graph

The `data` dictionary is a directed graph where:
- **Nodes** = dictionary keys (code-names)
- **Edges** = each atom's `dependencies` array

To traverse: iterate the dictionary, and for each atom follow its
`dependencies` to other keys in the same dictionary.

```python
import json

with open("rust_curve25519-dalek_4.1.3.json") as f:
    envelope = json.load(f)

atoms = envelope["data"]

for code_name, atom in atoms.items():
    print(f"{atom['display-name']} ({atom['kind']}, {atom['language']})")
    print(f"  file: {atom['code-path']}:{atom['code-text']['lines-start']}")
    print(f"  deps: {len(atom['dependencies'])}")
```

### Filtering stubs

```python
real_atoms = {
    k: v for k, v in atoms.items()
    if v["code-path"] != ""
}
```

## Progress and summaries

Extract files hold the full call graph. For a roll-up of verification progress,
read the summary sidecars. These are analysis outputs and are never merged back
into atom files:

- `probe summary <atoms>` partitions the verified atoms of a merged file into
  entrypoints, functions, and lemmas (`probe/summary`).
- `probe-leanblueprint extract` writes a two-axis blueprint progress sidecar
  next to its enriched atoms, counting statement and proof status per blueprint
  node (`probe-leanblueprint/summary`).

## Validating extract output

The [`probe-extract-check`](https://github.com/Beneficial-AI-Foundation/probe/tree/main/probe-extract-check)
tool (included in the probe repo) validates extract JSON against the
source code it was generated from. It checks file existence, line
ranges, display-name presence, kind correctness, and dependency
consistency.

```bash
# Structural checks only (no source needed)
probe-extract-check output.json

# Full validation against the source project
probe-extract-check output.json --project /path/to/project
```

See [extract-check-design.md](extract-check-design.md) for the full
list of checks and [probe-extract-check/TESTING.md](../probe-extract-check/TESTING.md)
for the test guide.

## Installation

For installation instructions, see the README in each probe's
repository listed in [The probe tools](#the-probe-tools) above.
