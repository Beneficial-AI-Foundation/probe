# Probe Atom Interchange Specification

The interchange format for atom files produced by `probe-*` tools (probe-verus,
probe-lean, probe-leanblueprint, etc.). Any tool that produces or consumes atom
files should conform to it. A shared schema enables:

- Merging atoms from different languages/tools into a single file
- Generic consumers (verilib-cli, specs browser, etc.) that work across languages
- Language-specific extensions without burdening unrelated consumers

## Normative specification

The authoritative specification is
**[`kb/engineering/schema.md`](../kb/engineering/schema.md)** (Schema 3.x). It
defines:

- the envelope (fields, merged variant, provenance) — design rationale in
  [envelope-rationale.md](envelope-rationale.md)
- registered `schema` values and schema categories
- atom core fields, kind values, language assignment, common optional fields
- correspondence records (`maps-to`/`mapped-from`)
- code-name URI conventions and stubs
- merge rules, authority validation, and normalization
- the mappings file format
- projection metadata
- versioning rules and version history

The invariants every implementation must preserve are in
[`kb/engineering/properties.md`](../kb/engineering/properties.md).

## Tool-specific extension fields

Tools may add language-specific fields. Rules:

1. Extension fields must not conflict with core or common optional field names.
2. Extension fields should use kebab-case naming.
3. Consumers that do not recognize an extension field must ignore it.
4. Extension fields should be omitted (not set to null) when not applicable.

Each tool's extension fields are specified in that tool's own `docs/SCHEMA.md`:

- [probe-rust](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/docs/SCHEMA.md)
- [probe-verus](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md)
- [probe-lean](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md)
- [probe-aeneas](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md)
- [probe-leanblueprint](https://github.com/Beneficial-AI-Foundation/probe-leanblueprint/blob/main/docs/SCHEMA.md)
  (also carries the probe-lean fields, since it enriches a `probe-lean/extract` base)

## Design Rationale

### Why code-names are fully qualified URIs

Code-names like `probe:curve25519-dalek/4.1.3/scalar/Scalar#add()` embed crate name and
version even though the envelope already identifies the producing tool:

**Crate/version is a per-atom fact.** A single file can hold atoms from multiple crates --
after merging, or when indexing a multi-package Cargo workspace -- so crate/version
belongs on the atom, not the envelope.

**References are self-contained.** An atom's `dependencies` may point across crates (e.g.
`probe:crate-b/1.0/helpers/compute()`); fully qualified URIs stay interpretable without
envelope context.

**Copy-paste resilience.** A code-name pasted into a bug report or database identifies its
atom uniquely; a short ID like `scalar/Scalar#add()` would not.

This matches HTTP URLs, SCIP symbols, DOIs, and RDF IRIs, which are all fully qualified.

### Why `code-module` and `display-name` duplicate parts of the code-name

`code-module` (e.g., `"scalar"`) is extractable from the code-name URI, and `display-name`
(e.g., `"Scalar::add"`) is a human-friendly version of the function component. Intentional
denormalization: grouping atoms by module or displaying a name should not require URI
parsing.

### What the envelope contains vs. what it does not

The envelope describes the **file**: who produced it (`schema`, `tool`), what format it
uses (`schema-version`), what project was analyzed (`source`), and when (`timestamp`).
The full field reference is
[`kb/engineering/schema.md` § Envelope fields](../kb/engineering/schema.md#envelope-fields);
[envelope-rationale.md](envelope-rationale.md) records the rationale.

The envelope does **not** duplicate per-atom facts:

- **Language.** Implied by `schema` for single-tool files (`probe-verus` = Rust —
  though per-atom `language` still follows the
  [kind→language rule](../kb/engineering/schema.md#language-assignment-for-verus-atoms),
  so `proof`/`spec` atoms carry `"verus"`; `probe-lean` = Lean). For merged files,
  atoms come from multiple languages, so a
  per-file language field would not apply. The per-atom `language` field handles both
  cases.
- **Crate name or version.** These are per-atom facts embedded in the code-name URI.
  A file may contain atoms from multiple crates (workspace indexing, merging). The
  `source` object captures the primary package for provenance but is not authoritative
  for per-atom identity.

### Summary of information layers

```
Envelope    "about the file"    schema, schema-version, tool, source, timestamp
Code-name   "about the atom"    crate, version, module, type, function (canonical ID)
Atom fields "convenient access" display-name, code-module, code-path, code-text, kind, language
```

Atom fields denormalize parts of the code-name (and add source location) for convenience;
no information is duplicated across the envelope and per-atom layers.

## JSON Schema

A machine-readable JSON Schema for the envelope and core atom fields is maintained at
[`schemas/atom-envelope.schema.json`](../schemas/atom-envelope.schema.json). All
`probe-*` codebases should validate their output against this schema in tests. See the
project [README](../README.md#json-schema) for usage examples in Rust, Lean, and CI.
