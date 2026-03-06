# Probe Atom Interchange Specification

Version: 2.0-draft

## Purpose

This document defines the interchange format for atom files produced by `probe-*` tools
(probe-verus, probe-lean, probe-latex, etc.). Any tool that produces or consumes atom files
should conform to this specification.

The goal is a shared schema that enables:

- Merging atoms from different languages/tools into a single file
- Generic consumers (verilib-cli, specs browser, etc.) that work across languages
- Language-specific extensions without burdening unrelated consumers

## Envelope

Every output file is wrapped in a metadata envelope. The envelope structure, field
definitions, and rationale are defined in
[envelope-rationale.md](envelope-rationale.md). This section covers only the
interchange-specific aspects: registered `schema` values and how the envelope relates
to multi-language merging.

### Registered `schema` Values

The `schema` field identifies the producing tool and data type. Format: `<tool>/<type>`.

| schema | Description |
|--------|-------------|
| `probe-verus/atoms` | Rust/Verus call graph atoms |
| `probe-verus/specs` | Rust/Verus function specifications |
| `probe-verus/proofs` | Rust/Verus verification results |
| `probe-lean/atoms` | Lean call graph atoms |
| `probe-lean/specs` | Lean specification status |
| `probe-lean/proofs` | Lean verification results (sorry detection) |
| `probe-lean/enriched-atoms` | Lean atoms + specs + proofs combined |
| `probe-latex/atoms` | LaTeX atoms (reserved, not yet defined) |
| `probe/merged-atoms` | Merged atoms from multiple tools |

New tools register their `schema` values by adding them to this table.

The `schema` field implicitly identifies the source language (`probe-verus` produces
Rust/Verus atoms, `probe-lean` produces Lean atoms, etc.). There is no separate `language`
field in the envelope because merged files contain atoms from multiple languages; the
per-atom `language` field handles that case. See [Design Rationale](#design-rationale).

### `data` (object, required)

The payload. For atom files, this is a dictionary keyed by `code-name`.
The structure of `data` depends on the `schema` value and is defined below.

## Atom Schema

When `schema` ends in `/atoms` or `/merged-atoms`, the `data` object is a dictionary
keyed by `code-name` strings. Each value is an atom object with the following fields.

### Core Fields (required for all languages)

#### Dictionary key: `code-name` (string)

A URI that uniquely identifies the atom. Not serialized inside the value object.

The URI scheme is language-specific (see [Code-Name URI Conventions](#code-name-uri-conventions)),
but all code-names must:

- Be valid UTF-8 strings
- Be unique within a single file
- Contain a scheme prefix followed by `:` (e.g., `probe:`, `latex:`)

#### `display-name` (string)

Human-readable name for the atom.

Examples: `"Scalar::add"`, `"Nat.add"`, `"Definition 3.1"`

#### `dependencies` (array of strings)

Code-name URIs of atoms referenced by this atom. For code atoms, these are
called functions. For mathematical atoms, these are referenced definitions/lemmas.

May be empty. Must be sorted for deterministic output.

#### `code-module` (string)

The module, namespace, or section path where this atom is defined.
Interpretation is language-specific:

- Rust: module path (e.g., `"scalar"`, `"backend/serial/u64/field"`)
- Lean: namespace (e.g., `"Mathlib.Data.Nat"`)
- LaTeX: section path (e.g., `"chapter3/elliptic_curves"`)

May be empty for top-level definitions.

#### `code-path` (string)

Relative path to the source file from the project root.

Examples: `"src/scalar.rs"`, `"Mathlib/Data/Nat.lean"`, `"chapters/ch3.tex"`

Empty string for external stubs (atoms without a local source definition).

#### `code-text` (object)

Line range of the atom's definition in the source file.

- `lines-start` (number): First line, 1-based.
- `lines-end` (number): Last line, 1-based.

Both are `0` for external stubs.

#### `kind` (string)

Classification of the atom's role. The allowed values are language-specific
(see [Kind Values](#kind-values)), but the field name and semantics are universal:
it answers "what kind of unit is this?"

#### `language` (string)

The source language of the atom. This field allows consumers to distinguish
atoms in a merged file without parsing the code-name URI.

Known values: `"rust"`, `"lean"`, `"latex"`

### Kind Values

Kind values are scoped by language. A consumer that does not recognize a kind value
should treat it as opaque (display it, but do not assign special semantics).

#### Rust/Verus

| Value | Meaning |
|-------|---------|
| `exec` | Executable code (compiled and verified) |
| `proof` | Proof code (verified but erased at runtime) |
| `spec` | Specification (defines logical properties, erased at runtime) |

Default for external stubs: `exec`.

#### Lean

| Value | Lean construct | Notes |
|-------|---------------|-------|
| `def` | `def` | Computable definition |
| `theorem` | `theorem` | Proven proposition (erased at runtime) |
| `abbrev` | `abbrev` | Abbreviation (reducible definition) |
| `class` | `class` | Type class |
| `structure` | `structure` | Record type |
| `inductive` | `inductive` | Inductive type |
| `instance` | `instance` | Type class instance |
| `axiom` | `axiom` | Axiom (trusted, no proof) |
| `opaque` | `opaque` | Opaque definition (no unfolding) |
| `quot` | `Quot` | Quotient type (built-in) |

Lean does not have a separate "proof" kind because proofs are the bodies of `theorem`
declarations, not standalone units.

#### LaTeX (reserved, not yet finalized)

Anticipated values: `definition`, `theorem`, `lemma`, `proof`, `remark`, `corollary`.
These will be defined when probe-latex is implemented.

### Optional Extension Fields

Tools may add language-specific optional fields to atom objects. Rules:

1. Optional fields must not conflict with core field names.
2. Optional fields should use kebab-case naming.
3. Consumers that do not recognize an optional field must ignore it.
4. Optional fields should be omitted (not set to null) when not applicable.

Current extensions defined by probe-verus:

| Field | Type | Description |
|-------|------|-------------|
| `dependencies-with-locations` | array of objects | Per-call location data (only with `--with-locations`) |

Each entry in `dependencies-with-locations` has:

- `code-name` (string): the dependency's code-name
- `location` (string): `"precondition"`, `"postcondition"`, or `"inner"`
- `line` (number): source line of the call

Current extensions defined by probe-lean:

| Field | Type | Description |
|-------|------|-------------|
| `is-hidden` | bool | From `.verilib/config.json` `user.is-hidden` list |
| `is-extraction-artifact` | bool | Name ends with configured extraction artifact suffix |
| `is-ignored` | bool | From `.verilib/config.json` `user.is-ignored` list |
| `is-relevant` | bool | Rust source is from the target crate (Aeneas projects only) |
| `rust-source` | string or null | Rust source path from Aeneas docstring |

## Code-Name URI Conventions

Each language defines its own code-name format. Languages may share a scheme prefix
(both Rust and Lean use `probe:`) as long as the code-name structure is unambiguous.

### Rust (`probe:`)

Format:

```
probe:<crate>/<version>/<module>/<Type>#<Trait><TypeParam>#<method>()
```

Examples:

- Free function: `probe:curve25519-dalek/4.1.3/field/reduce()`
- Inherent method: `probe:curve25519-dalek/4.1.3/field/FieldElement51#square()`
- Trait impl: `probe:curve25519-dalek/4.1.3/scalar/Scalar#Add<&Scalar>#add()`

### Lean (`probe:`)

Lean atoms use the `probe:` prefix followed by the fully qualified Lean name:

```
probe:<FullyQualifiedName>
```

Examples:

- `probe:ArkLib.SumCheck.Protocol.Prover.prove`
- `probe:Mathlib.Data.Nat.Basic.succ_pos`
- `probe:RegexDeriv.Language.Semantics.derive_correct`

Lean's namespace hierarchy already encodes the package/library prefix (e.g.,
`Mathlib.Data.Nat` is unambiguously from Mathlib), so the package name and version
are not embedded in the code-name. Cross-project disambiguation is handled by the
envelope's `source.package` field and, in merged files, by the per-atom `language`
field.

### LaTeX (`latex:`) -- reserved

Format to be defined when probe-latex is implemented.

## External Function Stubs

An atom that represents a dependency without a local source definition is a stub.
Stubs are identified by:

- `code-path` is `""`
- `code-text` has `lines-start: 0` and `lines-end: 0`
- `dependencies` is empty

Stubs allow the dependency graph to reference external functions without requiring
their source to be indexed. The `merge` operation can resolve stubs: if a stub
in one file matches a real atom in another, the real atom replaces the stub.

## Merged Data

The `probe merge` command combines data files of the same category from multiple
sources. It supports three categories: **atoms**, **specs**, and **proofs**. All
inputs must be the same category.

### Merged Atoms

A merged atom file combines atoms from multiple sources. Its envelope uses
`"schema": "probe/merged-atoms"`. The `data` dictionary may contain atoms
with different `language` values and code-name URI schemes.

Atom merge rules (first-wins with stub replacement):

1. If the same code-name appears as a stub in one file and a real atom in another,
   the real atom wins.
2. If the same code-name appears as a real atom in multiple files, this is a conflict.
   The merge tool should report it and keep the first occurrence.
3. Atoms with distinct code-names are concatenated.

### Merged Specs and Proofs

Merged spec and proof files use `"schema": "probe/merged-specs"` or
`"probe/merged-proofs"` respectively. The `data` dictionary values are
opaque to the merge tool (they differ between tools and languages).

Spec/proof merge rules (last-wins):

1. If the same code-name appears in multiple inputs, the **last** one wins. This is
   appropriate because re-running `specify` or `verify` should override stale results.
2. Entries with distinct code-names are concatenated.

The full merge algorithm, including normalization, envelope handling, and cross-language
considerations, is specified in [merge-algorithm.md](merge-algorithm.md).

## Complete Example

```json
{
  "schema": "probe-verus/atoms",
  "schema-version": "2.0",
  "tool": {
    "name": "probe-verus",
    "version": "2.0.0",
    "command": "atomize"
  },
  "source": {
    "repo": "https://github.com/ArtificialBreeze/curve25519-dalek",
    "commit": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
    "language": "rust",
    "package": "curve25519-dalek",
    "package-version": "4.1.3"
  },
  "timestamp": "2026-03-05T14:30:00Z",
  "data": {
    "probe:curve25519-dalek/4.1.3/scalar/Scalar#Add<&Scalar>#add()": {
      "display-name": "Scalar::add",
      "dependencies": [
        "probe:curve25519-dalek/4.1.3/scalar/UnpackedScalar#add()"
      ],
      "code-module": "scalar",
      "code-path": "src/scalar.rs",
      "code-text": {
        "lines-start": 450,
        "lines-end": 475
      },
      "kind": "exec",
      "language": "rust"
    },
    "probe:curve25519-dalek/4.1.3/scalar/UnpackedScalar#add()": {
      "display-name": "UnpackedScalar::add",
      "dependencies": [],
      "code-module": "",
      "code-path": "",
      "code-text": {
        "lines-start": 0,
        "lines-end": 0
      },
      "kind": "exec",
      "language": "rust"
    }
  }
}
```

## Specs and Proofs

The envelope structure applies to all probe-verus output file types. The `data`
field is always a dictionary keyed by code-name, but the value schema differs:

- **`probe-verus/specs`**: Values contain specification metadata (see probe-verus
  documentation for the full field list: `specified`, `kind`, `language`,
  `code-path`, `spec-text`, `has_requires`, `has_ensures`, etc.)
- **`probe-verus/proofs`**: Values contain verification results (`code-path`,
  `code-line`, `verified`, `status`).

These schemas are currently probe-verus-specific. If probe-lean defines specs or
proofs, it would register `probe-lean/specs`, `probe-lean/proofs` with its own
value schemas.

## Design Rationale

### Why code-names are fully qualified URIs

Code-names like `probe:curve25519-dalek/4.1.3/scalar/Scalar#add()` include the crate
name and version even though the envelope identifies the producing tool. This is intentional:

**Crate/version is a per-atom fact, not a per-file fact.** A single file can contain atoms
from multiple crates -- after `merge-atoms`, or when indexing a Cargo workspace with
multiple packages. Putting crate/version in the envelope would be incorrect for these
cases. The code-name is the only location that correctly captures per-atom origin.

**Self-contained references.** Atom A's `dependencies` list contains code-names of other
atoms, potentially from different crates. The dependency
`probe:crate-b/1.0/helpers/compute()` is a cross-crate reference that must be
interpretable without envelope context. Fully qualified URIs make every reference
self-contained.

**Copy-paste resilience.** A code-name pasted into a bug report, log, or database
identifies its atom uniquely. A short ID like `scalar/Scalar#add()` would be ambiguous
without knowing which crate and version it came from.

This follows established practice: HTTP URLs, SCIP symbols, DOIs, and RDF IRIs are all
fully qualified even when surrounding context could supply parts of the identifier.

### Why `code-module` and `display-name` duplicate parts of the code-name

`code-module` (e.g., `"scalar"`) is extractable from the code-name URI. `display-name`
(e.g., `"Scalar::add"`) is a human-friendly version of the function component. This is
intentional denormalization for consumer convenience: grouping atoms by module or
displaying a name should not require URI parsing.

### What the envelope contains vs. what it does not

The envelope describes the **file**: who produced it (`schema`, `tool`), what format it
uses (`schema-version`), what project was analyzed (`source`), and when (`timestamp`).
See [envelope-rationale.md](envelope-rationale.md) for the full field reference.

The envelope does **not** duplicate per-atom facts:

- **Language.** Implied by `schema` for single-tool files (`probe-verus` = Rust,
  `probe-lean` = Lean). For merged files, atoms come from multiple languages, so a
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

The envelope is per-file metadata. The code-name is the canonical per-atom identifier.
Atom fields denormalize parts of the code-name (and add source location) for consumer
convenience. No information is duplicated across the envelope and per-atom layers.

## Versioning

This specification follows semver:

- **Major** (e.g., 2.0 to 3.0): Changes to required fields, their types, or their
  semantics. Removes fields. Changes the envelope structure.
- **Minor** (e.g., 2.0 to 2.1): Adds new optional fields. Adds new `kind` values.
  Registers new `schema` values.

Consumers should check `schema-version` major version for compatibility and ignore
unknown optional fields for forward compatibility.

## JSON Schema

A machine-readable JSON Schema for the envelope and core atom fields is maintained at
[`schemas/atom-envelope.schema.json`](../schemas/atom-envelope.schema.json). All
`probe-*` codebases should validate their output against this schema in tests. See the
project [README](../README.md#json-schema) for usage examples in Rust, Lean, and CI.
