# probe

Cross-tool atom operations for the `probe-*` verification tool family.

This repository contains:

- **Specification documents** defining the interchange format for atom files
- **JSON Schema** for machine-validatable envelope and atom structure
- **`probe` CLI** for cross-tool operations (`merge`, `project`, `enrich`, `summary`)
- **`probe-extract-check`** -- validator that checks extract JSON correctness against source code

## Documentation

The normative specifications live in the knowledge base: start at
[kb/index.md](kb/index.md). In particular
[kb/engineering/schema.md](kb/engineering/schema.md) (Schema 3.x interchange
format, merge algorithm, mappings file format) and
[kb/engineering/properties.md](kb/engineering/properties.md) (invariants).

Reference docs in this repo:

- [docs/consumer-guide.md](docs/consumer-guide.md) -- **Start here**: how to use all probe tools, examples, and working with the data
- [docs/SCHEMA.md](docs/SCHEMA.md) -- Atom interchange format: pointer to the normative spec, per-tool extension docs, design rationale
- [docs/schema-validation.md](docs/schema-validation.md) -- Validating probe output against the JSON Schema (Rust, CLI, CI)
- [docs/ui-views.md](docs/ui-views.md) -- How a UI should implement language toggles, call graph / file map / crate map views
- [docs/testing-guide.md](docs/testing-guide.md) -- Testing that your visualization matches the probe data
- [docs/envelope-rationale.md](docs/envelope-rationale.md) -- Envelope design and rationale
- [kb/engineering/categorical-framework.md](kb/engineering/categorical-framework.md) -- Categorical/algebraic structure of probe merge
- [probe-extract-check/TESTING.md](probe-extract-check/TESTING.md) -- What the extract-check validator checks, and its test guide
- [schemas/atom-envelope.schema.json](schemas/atom-envelope.schema.json) -- JSON Schema

### Per-tool docs across the ecosystem

Each probe repo has `docs/USAGE.md` (command reference) and `docs/SCHEMA.md` (JSON schema):

| Repo | Usage | Schema | Schema scope |
|------|-------|--------|-------------|
| **[probe](https://github.com/Beneficial-AI-Foundation/probe)** | -- | [`kb/engineering/schema.md`](kb/engineering/schema.md) | Interchange spec: core fields, common optional fields, code-name conventions |
| **[probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/SCHEMA.md) | Rust-specific fields |
| **[probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md) | Lean-specific fields |
| **[probe-leanblueprint](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/SCHEMA.md) | Lean blueprint progress fields |
| **[probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md) | Verus-specific fields |
| **[probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md) | Aeneas-specific fields |
| **[probe-vcvio](https://github.com/Beneficial-AI-Foundation/probe-vcvio)** (proof of concept) | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-vcvio/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-vcvio/blob/main/docs/SCHEMA.md) | VCVio security-protocol classification fields |

## Usage

```bash
# Install the CLI (or use `cargo run -- <subcommand> …` from the repo)
cargo install --path .

# Merge data files from different probe tools (atoms, specs, or proofs)
probe merge verus_atoms.json lean_atoms.json -o merged.json

# Project a subgraph around the mapping endpoints (seeds), expanding
# callees (--forward-depth, default 2) and callers (--reverse-depth,
# default 0); add --emit-focus for a probegraph focus-set file
probe project merged.json --mappings mappings.json -o projected.json

# Enrich verification status (upgrade "verified" → "transitively-verified"
# for atoms with no failed/unverified atom reachable along a dependency
# path that doesn't pass through a trusted boundary — see P23)
probe enrich extract_output.json -o enriched.json

# Summarize verified atoms (entrypoints, functions, lemmas)
probe summary merged.json -o summary.json

# Run tests
cargo test
```

## JSON Schema

[`schemas/atom-envelope.schema.json`](schemas/atom-envelope.schema.json) validates
single-tool and merged envelopes; see [docs/schema-validation.md](docs/schema-validation.md)
for what it covers and how to validate (Rust, CLI, CI).

## Acknowledgements

The probe ecosystem's development methodology — knowledge bases, auditor skills, and
Ralph Loops (implement → audit → fix → repeat) — follows the spec-driven agentic
development approach proposed in [kb-sync-demo](https://github.com/yurug/kb-sync-demo).
