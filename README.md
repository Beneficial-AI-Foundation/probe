# probe

Cross-tool atom operations for the `probe-*` verification tool family.

This repository contains:

- **Specification documents** defining the interchange format for atom files
- **JSON Schema** for machine-validatable envelope and atom structure
- **`probe` CLI** for cross-tool operations (`merge`, `enrich`, `summary`)
- **`probe-extract-check`** -- validator that checks extract JSON correctness against source code

## Documentation

- [docs/consumer-guide.md](docs/consumer-guide.md) -- **Start here**: how to use all four probe tools, examples, and working with the data
- [docs/SCHEMA.md](docs/SCHEMA.md) -- Atom interchange format (Schema 2.0)
- [docs/schema-validation.md](docs/schema-validation.md) -- Validating probe output against the JSON Schema (Rust, Lean, CI)
- [docs/ui-views.md](docs/ui-views.md) -- How a UI should implement language toggles, call graph / file map / crate map views
- [docs/verification-statuses.md](docs/verification-statuses.md) -- Per-atom status fields and the color scheme derived from them
- [docs/testing-guide.md](docs/testing-guide.md) -- Testing that your visualization matches the probe data
- [docs/envelope-rationale.md](docs/envelope-rationale.md) -- Envelope design and rationale
- [docs/merge-algorithm.md](docs/merge-algorithm.md) -- Merge algorithm specification
- [docs/mappings-spec.md](docs/mappings-spec.md) -- Cross-language mapping file format
- [docs/categorical-framework.md](docs/categorical-framework.md) -- Categorical/algebraic structure of probe merge
- [docs/extract-check-design.md](docs/extract-check-design.md) -- Design of the extract-check validation tool
- [probe-extract-check/TESTING.md](probe-extract-check/TESTING.md) -- Test guide for probe-extract-check
- [schemas/atom-envelope.schema.json](schemas/atom-envelope.schema.json) -- JSON Schema

### Per-tool docs across the ecosystem

Each probe repo has `docs/USAGE.md` (command reference) and `docs/SCHEMA.md` (JSON schema):

| Repo | Usage | Schema | Schema scope |
|------|-------|--------|-------------|
| **[probe](https://github.com/Beneficial-AI-Foundation/probe)** | -- | [`docs/SCHEMA.md`](docs/SCHEMA.md) | Interchange spec: core fields, common optional fields, code-name conventions |
| **[probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/SCHEMA.md) | Rust-specific fields |
| **[probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md) | Lean-specific fields |
| **[probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md) | Verus-specific fields |
| **[probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas)** | [`docs/USAGE.md`](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/USAGE.md) | [`docs/SCHEMA.md`](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md) | Aeneas-specific fields |

## Usage

```bash
# Build
cargo build

# Merge data files from different probe tools (atoms, specs, or proofs)
probe merge verus_atoms.json lean_atoms.json -o merged.json

# Enrich verification status (upgrade "verified" → "transitively-verified"
# for atoms whose entire transitive closure is verified or trusted)
probe enrich extract_output.json -o enriched.json

# Summarize verified atoms (entrypoints, functions, lemmas)
probe summary merged.json -o summary.json

# Run tests
cargo test
```

## JSON Schema

[`schemas/atom-envelope.schema.json`](schemas/atom-envelope.schema.json) is a
[JSON Schema (draft 2020-12)](https://json-schema.org/draft/2020-12/schema) validating both
single-tool and merged-atoms envelopes -- the machine-readable contract all `probe-*`
codebases should validate against. See [docs/schema-validation.md](docs/schema-validation.md)
for validation examples (Rust, Lean, CI) and what the schema covers.

## Acknowledgements

The probe ecosystem's development methodology — knowledge bases, auditor skills, and
Ralph Loops (implement → audit → fix → repeat) — follows the spec-driven agentic
development approach proposed in [kb-sync-demo](https://github.com/yurug/kb-sync-demo).

## Related projects

- [probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust) -- Rust call graph atoms from SCIP index
- [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus) -- Rust/Verus call graph atoms and verification
- [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean) -- Lean call graph atoms and verification
- [probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas) -- Cross-language Rust+Lean merged atoms (Aeneas projects)
- [scip-callgraph](https://github.com/Beneficial-AI-Foundation/scip-callgraph) -- Interactive web viewer for probe data ([live demo](https://beneficial-ai-foundation.github.io/scip-callgraph/))
