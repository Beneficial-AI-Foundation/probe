# Probe Mappings Specification

> **Archived 2026-09-28.** Superseded by
> [kb/engineering/schema.md § Mappings file format](../../kb/engineering/schema.md#mappings-file-format)
> (normative). Kept for historical reference; not updated.

Version: draft
Date: 2026-09-28
Parent document: [SCHEMA.md](SCHEMA.md)

The cross-language mappings file format used by `probe merge` to attach correspondence
records across languages (see [ADR-006](../kb/decisions/006-correspondence-records.md)).

## Motivation

When merging atoms from different languages, the same logical function has different
code-names:

- **Rust (probe-verus)**: `probe:curve25519-dalek/4.1.3/field/u64/serial/backend/FieldElement51<[u64;/5]>#reduce()`
- **Lean (probe-lean)**: `probe:curve25519_dalek.backend.serial.u64.field.FieldElement51.reduce`

These cannot be matched by string equality. A mappings file provides the explicit
mapping between code-names so that `probe merge` can link them.

A single `from` key may map to multiple `to` targets (1-to-many), enabling scenarios
where one implementation corresponds to several formal constructs.

## File Format

A mappings file is a Schema 3.0 JSON document with the following envelope:

```json
{
  "schema": "probe/mappings",
  "schema-version": "3.0",
  "tool": {
    "name": "probe-aeneas",
    "version": "0.10.0",
    "command": "translate"
  },
  "timestamp": "2026-03-12T14:00:00Z",
  "sources": {
    "from": {
      "schema": "probe-verus/atoms",
      "package": "curve25519-dalek",
      "package-version": "4.1.3"
    },
    "to": {
      "schema": "probe-lean/atoms",
      "package": "Curve25519Dalek",
      "package-version": "0.1.0"
    }
  },
  "mappings": [
    {
      "from": "probe:curve25519-dalek/4.1.3/field/u64/serial/backend/FieldElement51<[u64;/5]>#reduce()",
      "to": "probe:curve25519_dalek.backend.serial.u64.field.FieldElement51.reduce",
      "confidence": "exact",
      "method": "rust-qualified-name"
    }
  ]
}
```

### Top-Level Fields

#### `schema` (string, required)

Must be `"probe/mappings"`.

#### `schema-version` (string, required)

Must be `3.x` (currently `"3.0"`).

#### `tool` (object, required)

Metadata about the tool that generated the mappings file. Same structure as
other probe envelopes.

#### `timestamp` (string, required)

ISO 8601 timestamp.

#### `sources` (object, required)

Describes what the mappings connect:

- `from` (object, required): The source side of mappings.
  - `schema` (string): Schema of the source atoms file.
  - `package` (string): Package name of the source.
  - `package-version` (string): Version of the source package.
- `to` (object, required): The target side of mappings.
  - Same fields as `from`.

#### `mappings` (array of objects, required)

Each entry maps one code-name to another. Multiple entries with the same `from`
key are allowed (1-to-many).

### Mapping Entry Fields

#### `from` (string, required)

The code-name in the source probe output (e.g., a probe-verus atom code-name).

#### `to` (string, required)

The corresponding code-name in the target probe output (e.g., a probe-lean atom
code-name).

#### `confidence` (string, required)

How the mapping was established:

| Value | Meaning |
|-------|---------|
| `exact` | Matched via rust-qualified-name or equivalent deterministic key |
| `exact-disambiguated` | Matched via rust-qualified-name with file/line disambiguation |
| `file-and-name` | Matched via same source file + display-name overlap |
| `file-and-lines` | Matched via same source file + overlapping line ranges |
| `heuristic` | Matched via fuzzy heuristics (lower confidence) |
| `manual` | Manually authored mapping |

#### `method` (string, optional)

A finer description of the matching method used. Examples:
`"rust-qualified-name"`, `"file+display-name"`, `"file+line-overlap"`, `"manual"`.

## Semantics

A mapping entry records a **correspondence** — "there is a mappings-file entry
linking these names, with this confidence" — not an identity and not a
dependency. The `from`/`to` fields are generic source/target roles: the
`sources` block describes each side but assigns no implementation/formal
roles (a Lean→Rust file is legal).

When `probe merge --mappings <file>` processes an atom merge
([ADR-006](../kb/decisions/006-correspondence-records.md)):

1. Load the full mapping records (`from`, `to`, `confidence`, optional
   `method`), normalizing endpoints (P8, trailing-dot strip) before any
   lookup.
2. After the base merge, attach a `maps-to` record
   (`{target, confidence, method?}`) to each mapping's `from` atom present
   in the merged map, and a mirror `mapped-from` record to each `to` atom
   present. `dependencies` is never modified. A record whose target is
   absent from the merged map is still attached (dangling target ⇒
   warning, not skip).
3. The merged output retains all atoms as separate entries with their
   original code-names; cross-language linkage is carried as
   correspondence records, and cross-language *resolution* is a derived
   consumer view over those records.

Mirrors are best-effort: if the target atom is absent at attachment time
and arrives in a later plain merge, it carries no `mapped-from`. The
correspondence relation is defined as the union over both fields, so a
missing mirror loses no information; re-running merge with the mappings
file (or regenerating) restores mirrors. Corrections and withdrawals take
effect only by regenerating from extracts + the corrected mappings file —
attachment never removes a record.

## Folder Convention

Mappings files live in `.verilib/mappings/` with the naming convention:

```
<from_tool>_<from_package>__<to_tool>_<to_package>.json
```

Example: `lean_SecureMessaging__rust_libsignal-protocol.json`

## Generation

Mappings can be generated by any tool that has access to both probe outputs
and a mapping source. Common approaches:

1. **probe-aeneas `translate`**: Automated three-strategy matching for
   Aeneas-transpiled projects (generates 1-to-1 mappings).

2. **Manual authoring**: For cross-language linking where no transpilation
   relationship exists (e.g., mapping Rust implementations to Lean formal
   specifications of the same protocol).

3. **`rust-qualified-name` matching**: If probe-rust atoms include the
   `rust-qualified-name` extension field, join with `functions.json` (which maps
   `rust_name` to `lean_name`) and then to probe-lean code-names.

4. **File + line matching**: Match atoms that share the same `code-path` (Rust
   source file) and have overlapping or nearby line ranges.
