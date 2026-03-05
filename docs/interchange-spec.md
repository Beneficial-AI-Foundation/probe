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

Every output file is a JSON object with metadata fields and one data field:

```json
{
  "schema": "<tool>/<type>",
  "schema-version": "2.0",
  "tool-version": "<semver>",
  "source": "<project-name>",
  "data": { ... }
}
```

### `schema` (string, required)

Identifies the producing tool and data type. Format: `<tool>/<type>`.

Known values:

| schema | Description |
|--------|-------------|
| `probe-verus/atoms` | Rust/Verus call graph atoms |
| `probe-verus/specs` | Rust/Verus function specifications |
| `probe-verus/proofs` | Rust/Verus verification results |
| `probe-lean/atoms` | Lean atoms (reserved, not yet defined) |
| `probe-latex/atoms` | LaTeX atoms (reserved, not yet defined) |
| `probe/merged-atoms` | Merged atoms from multiple tools |

New tools register their `schema` values by adding them to this table.

The `schema` field also implicitly identifies the source language (`probe-verus` produces
Rust/Verus atoms, `probe-lean` produces Lean atoms, etc.). There is no separate `language`
field in the envelope because merged files contain atoms from multiple languages; the
per-atom `language` field handles that case. See [Design Rationale](#design-rationale).

### `schema-version` (string, required)

The version of this interchange specification that the file conforms to.
Uses `<major>.<minor>` format. A change to required fields or their semantics
increments the major version. Adding optional fields increments the minor version.

### `tool-version` (string, required)

The semver version of the tool that produced the file (e.g., `"2.0.0"`).
Informational; consumers should key compatibility decisions on `schema-version`, not this.

### `source` (string, optional)

Human-readable name of the project or repository that was analyzed
(e.g., `"curve25519-dalek"`, `"libsignal"`). Informational only; consumers
should not depend on this for logic. Omitted or set to null for merged files
that combine atoms from multiple projects.

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
- Contain a scheme prefix followed by `:` (e.g., `probe:`, `lean:`)

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

#### `mode` (string)

Classification of the atom's role. The allowed values are language-specific
(see [Mode Values](#mode-values)), but the field name and semantics are universal:
it answers "what kind of unit is this?"

#### `language` (string)

The source language of the atom. This field allows consumers to distinguish
atoms in a merged file without parsing the code-name URI.

Known values: `"rust"`, `"lean"`, `"latex"`

### Mode Values

Mode values are scoped by language. A consumer that does not recognize a mode value
should treat it as opaque (display it, but do not assign special semantics).

#### Rust/Verus

| Value | Meaning |
|-------|---------|
| `exec` | Executable code (compiled and verified) |
| `proof` | Proof code (verified but erased at runtime) |
| `spec` | Specification (defines logical properties, erased at runtime) |

Default for external stubs: `exec`.

#### Lean (reserved, not yet finalized)

Anticipated values: `def`, `theorem`, `lemma`, `noncomputable def`, `instance`, `class`.
These will be defined when probe-lean is implemented.

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

## Code-Name URI Conventions

Each language defines its own URI scheme. The scheme prefix (before `:`) must be
unique per language.

### Rust (`probe:`)

Format:

```
probe:<crate>/<version>/<module>/<Type>#<Trait><TypeParam>#<method>()
```

Examples:

- Free function: `probe:curve25519-dalek/4.1.3/field/reduce()`
- Inherent method: `probe:curve25519-dalek/4.1.3/field/FieldElement51#square()`
- Trait impl: `probe:curve25519-dalek/4.1.3/scalar/Scalar#Add<&Scalar>#add()`

### Lean (`lean:`) -- reserved

Format to be defined when probe-lean is implemented. Anticipated:

```
lean:<package>/<version>/<namespace>/<name>
```

### LaTeX (`latex:`) -- reserved

Format to be defined when probe-latex is implemented.

## External Function Stubs

An atom that represents a dependency without a local source definition is a stub.
Stubs are identified by:

- `code-path` is `""`
- `code-text` has `lines-start: 0` and `lines-end: 0`
- `dependencies` is empty

Stubs allow the dependency graph to reference external functions without requiring
their source to be indexed. The `merge-atoms` operation can resolve stubs: if a stub
in one file matches a real atom in another, the real atom replaces the stub.

## Merged Atoms

A merged atom file combines atoms from multiple sources. Its envelope uses
`"schema": "probe/merged-atoms"`. The `data` dictionary may contain atoms
with different `language` values and code-name URI schemes.

Merge rules:

1. If the same code-name appears as a stub in one file and a real atom in another,
   the real atom wins.
2. If the same code-name appears as a real atom in multiple files, this is a conflict.
   The merge tool should report it and keep the first occurrence.
3. Atoms with distinct code-names are concatenated.

## Complete Example

```json
{
  "schema": "probe-verus/atoms",
  "schema-version": "2.0",
  "tool-version": "2.0.0",
  "source": "curve25519-dalek",
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
      "mode": "exec",
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
      "mode": "exec",
      "language": "rust"
    }
  }
}
```

## Specs and Proofs

The envelope structure applies to all probe-verus output file types. The `data`
field is always a dictionary keyed by code-name, but the value schema differs:

- **`probe-verus/specs`**: Values contain specification metadata (see probe-verus
  documentation for the full field list: `specified`, `mode`, `language`,
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

The envelope describes the **file**: who produced it (`schema`, `tool-version`), what
format it uses (`schema-version`), and optionally what project was analyzed (`source`).

The envelope does **not** contain:

- **Language.** Implied by `schema` for single-tool files (`probe-verus` = Rust,
  `probe-lean` = Lean). For merged files, atoms come from multiple languages, so a
  per-file language field would not apply. The per-atom `language` field handles both
  cases.
- **Crate name or version.** These are per-atom facts embedded in the code-name URI.
  A file may contain atoms from multiple crates (workspace indexing, merging). The
  optional `source` field captures the project name for informational purposes but is
  not authoritative.
- **Source hashes or cache invalidation data.** Staleness detection is a build system
  concern, not a schema concern. See the project's schema evolution plan for discussion.

### Summary of information layers

```
Envelope    "about the file"    schema, schema-version, tool-version, source
Code-name   "about the atom"    crate, version, module, type, function (canonical ID)
Atom fields "convenient access" display-name, code-module, code-path, code-text, mode, language
```

The envelope is per-file metadata. The code-name is the canonical per-atom identifier.
Atom fields denormalize parts of the code-name (and add source location) for consumer
convenience. No information is duplicated across the envelope and per-atom layers.

## Versioning

This specification follows semver:

- **Major** (e.g., 2.0 to 3.0): Changes to required fields, their types, or their
  semantics. Removes fields. Changes the envelope structure.
- **Minor** (e.g., 2.0 to 2.1): Adds new optional fields. Adds new `mode` values.
  Registers new `schema` values.

Consumers should check `schema-version` major version for compatibility and ignore
unknown optional fields for forward compatibility.
