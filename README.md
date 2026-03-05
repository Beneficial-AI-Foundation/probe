# probe

Cross-tool atom operations for the `probe-*` verification tool family.

This repository contains:

- **Specification documents** defining the interchange format for atom files
- **JSON Schema** for machine-validatable envelope and atom structure
- **`probe` CLI** for cross-tool operations (currently: `merge-atoms`)

## Specification

- [docs/interchange-spec.md](docs/interchange-spec.md) -- Atom interchange format (Schema 2.0)
- [docs/envelope-rationale.md](docs/envelope-rationale.md) -- Envelope design and rationale
- [docs/merge-algorithm.md](docs/merge-algorithm.md) -- Merge algorithm specification
- [schemas/atom-envelope.schema.json](schemas/atom-envelope.schema.json) -- JSON Schema

## Usage

```bash
# Build
cargo build

# Merge atom files from different probe tools
probe merge-atoms verus_atoms.json lean_atoms.json -o merged.json

# Run tests
cargo test
```

## Related projects

- [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus) -- Rust/Verus call graph atoms and verification
- [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean) -- Lean call graph atoms and verification
