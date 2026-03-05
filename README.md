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

## JSON Schema

The file [`schemas/atom-envelope.schema.json`](schemas/atom-envelope.schema.json) is a
[JSON Schema (draft 2020-12)](https://json-schema.org/draft/2020-12/schema) that validates
both single-tool and merged-atoms envelopes, including the core atom fields. It is the
machine-readable contract that all `probe-*` codebases should validate against.

### Validating in Rust (probe, probe-verus)

Add `jsonschema` as a dev-dependency and validate in tests:

```rust
use jsonschema::Validator;

#[test]
fn output_conforms_to_schema() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/atom-envelope.schema.json")).unwrap();
    let validator = Validator::new(&schema).unwrap();

    let output: serde_json::Value = /* your tool's JSON output */;
    assert!(validator.validate(&output).is_ok());
}
```

### Validating in Lean (probe-lean)

Use a JSON library to parse the output and check required fields, or shell out to a
schema validator:

```bash
# Using jsonschema-rs CLI (install via cargo install jsonschema-cli)
jsonschema validate --schema schemas/atom-envelope.schema.json --instance output.json

# Using Python jsonschema (pip install jsonschema)
python -m jsonschema -i output.json schemas/atom-envelope.schema.json
```

### Validating in CI

Any CI pipeline can validate probe output against the schema. Download it from the repo:

```bash
curl -sL https://raw.githubusercontent.com/Beneficial-AI-Foundation/probe/main/schemas/atom-envelope.schema.json \
  -o atom-envelope.schema.json
```

### What the schema covers

- **Envelope structure**: `schema`, `schema-version`, `tool`, `source`/`inputs`, `timestamp`, `data`
- **Single-tool vs merged**: discriminated by the `schema` field (`probe-*/atoms` vs `probe/merged-atoms`)
- **Core atom fields**: `display-name`, `dependencies`, `code-module`, `code-path`, `code-text`, `kind`, `language`
- **Extensions**: `additionalProperties: true` on atoms allows language-specific fields to pass through

## Related projects

- [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus) -- Rust/Verus call graph atoms and verification
- [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean) -- Lean call graph atoms and verification
