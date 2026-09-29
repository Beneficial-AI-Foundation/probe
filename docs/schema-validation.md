# Validating probe output against the JSON Schema

[`schemas/atom-envelope.schema.json`](../schemas/atom-envelope.schema.json) is a
[JSON Schema (draft 2020-12)](https://json-schema.org/draft/2020-12/schema) that validates
both single-tool and merged envelopes, including the core atom fields.

Today this repo is the only codebase that validates against it (in
`tests/schema_validation.rs`); the sibling `probe-*` repos do not yet.
The recipes below show how any consumer or sibling can adopt it.

## Validating in Rust

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

## Validating from the command line

Use a JSON library to parse the output and check required fields, or shell out to a
schema validator:

```bash
# Using jsonschema-rs CLI (install via cargo install jsonschema-cli)
jsonschema validate --schema schemas/atom-envelope.schema.json --instance output.json

# Using Python jsonschema (pip install jsonschema)
python -m jsonschema -i output.json schemas/atom-envelope.schema.json
```

## Validating in CI

Any CI pipeline can validate probe output against the schema. Download it from the repo:

```bash
curl -sL https://raw.githubusercontent.com/Beneficial-AI-Foundation/probe/main/schemas/atom-envelope.schema.json \
  -o atom-envelope.schema.json
```

## What the schema covers

- **Envelope structure**: `schema`, `schema-version`, `tool`, `source`/`inputs`, `timestamp`, `data`
- **Single-tool vs composed**: the atoms branches are discriminated by provenance shape — a `source` object marks single-tool, an `inputs` array marks composed ([ADR-006](../kb/decisions/006-correspondence-records.md), [P9](../kb/engineering/properties.md#p9-provenance-is-preserved)); an envelope carrying both matches neither branch
- **Core atom fields**: `display-name`, `dependencies`, `code-module`, `code-path`, `code-text`, `kind`, `language`
- **Extensions**: `additionalProperties: true` on atoms allows language-specific fields to pass through

### Registered schema strings

The `oneOf` accepts exactly:

- `probe-(rust|lean|verus|aeneas|leanblueprint|vcvio)/(atoms|enriched-atoms|extract)` with `source` provenance — single-tool atom envelopes
- the same atom schema strings plus `probe/merged-atoms` and `probe/projected-atoms` with `inputs` provenance — composed atom envelopes (e.g. `probe-aeneas/extract` with `inputs: [Rust, Lean]`); a `probe/projected-atoms` envelope must also carry its `projection` block
- `probe-(rust|lean|verus|aeneas|leanblueprint)/(specs|proofs|stubs|verification-report)` — single-tool generic envelopes
- `probe/merged-(specs|proofs)` — merged generic envelopes

Not yet registered (files with these `schema` values fail validation):
`probe/summary`, `probe/mappings`,
`probe-lean/viewify`, `probe-leanblueprint/summary`.
