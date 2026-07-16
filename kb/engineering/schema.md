---
title: Schema 2.0 Interchange Specification
last-updated: 2026-06-03
status: draft
---

# Schema 2.0 Interchange Specification

This is the authoritative specification for the JSON interchange format shared by all probe tools. Per-tool `docs/SCHEMA.md` files document tool-specific details; this file defines the contract they all share.

## Envelope

Every probe output file is wrapped in a metadata envelope:

```json
{
  "schema": "probe-verus/extract",
  "schema-version": "2.0",
  "tool": {
    "name": "probe-verus",
    "version": "5.0.0",
    "command": "extract"
  },
  "source": {
    "repo": "https://github.com/org/project.git",
    "commit": "abc123def456...",
    "language": "rust",
    "package": "my-crate",
    "package-version": "1.0.0"
  },
  "timestamp": "2026-03-19T12:00:00Z",
  "data": { ... }
}
```

### Envelope fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema` | string | yes | Format identifier: `<tool>/<type>`. Identifies both the producing tool and the data shape. |
| `schema-version` | string | yes | `<major>.<minor>`. Major bump = breaking change. Minor = new optional fields. |
| `tool.name` | string | yes | e.g. `"probe-lean"`, `"probe-verus"`, `"probe"` |
| `tool.version` | string | yes | Semver of the producing tool |
| `tool.command` | string | yes | Which subcommand produced this file (e.g. `"extract"`, `"atomize"`, `"merge"`) |
| `source.repo` | string | yes | Git remote URL |
| `source.commit` | string | yes | Full git commit hash |
| `source.language` | string | yes | `"rust"`, `"lean"`, `"latex"` |
| `source.package` | string | yes | Crate/project name |
| `source.package-version` | string | yes | Version identifier (semver for Rust; commit hash for Lean if no version) |
| `timestamp` | string | yes | ISO 8601 |
| `data` | object | yes | Payload. Structure depends on `schema`. |

### Merged envelope variant

When `probe merge` produces output, `source` is replaced by `inputs`:

```json
{
  "schema": "probe/merged-atoms",
  "schema-version": "2.0",
  "tool": { "name": "probe", "version": "0.1.0", "command": "merge" },
  "inputs": [
    { "schema": "probe-verus/atoms", "source": { ... } },
    { "schema": "probe-lean/atoms", "source": { ... } }
  ],
  "timestamp": "...",
  "data": { ... }
}
```

When a previously merged file is used as input, its `inputs` entries are flattened into the new output — provenance is carried forward recursively.

### Registered schema values

**Single-tool schemas**:
- `probe-rust/extract`
- `probe-verus/atoms`, `probe-verus/extract`, `probe-verus/specs`, `probe-verus/proofs`, `probe-verus/stubs`, `probe-verus/verification-report`
- `probe-lean/extract`, `probe-lean/viewify`
- `probe-aeneas/extract`

Note: Legacy schema values `probe-lean/atoms`, `probe-lean/enriched-atoms`, `probe-lean/specs`, `probe-lean/proofs`, `probe-lean/stubs` exist from Schema 1.x and may appear in older files or as input sources in merged envelopes. Current probe-lean only produces `probe-lean/extract` and `probe-lean/viewify`.

**Merged schemas**:
- `probe/merged-atoms`, `probe/merged-specs`, `probe/merged-proofs`

**Analysis**:
- `probe/summary`

**Special**:
- `probe/mappings` — cross-language mappings

### Schema categories

The `schema` field implicitly identifies the data category:

| Category | Matches | Merge strategy |
|----------|---------|---------------|
| **Atoms** | `*/atoms`, `*/enriched-atoms`, `*/extract`, `probe/merged-atoms` | First-wins with stub replacement |
| **Specs** | `*/specs`, `probe/merged-specs` | Last-wins |
| **Proofs** | `*/proofs`, `probe/merged-proofs` | Last-wins |

Category detection is implemented in `probe/src/types.rs::detect_category()`.

## Atom

When `schema` identifies an atoms-category file, `data` is a dictionary keyed by [code-name](glossary.md#code-name) strings. Each value is an atom:

### Core fields (required for all languages)

| Field | Type | Description |
|-------|------|-------------|
| `display-name` | string | Human-readable name (e.g. `"MyStruct::method"`) |
| `dependencies` | array of strings | [Code-names](glossary.md#code-name) of atoms this one references |
| `code-module` | string | Module/namespace path |
| `code-path` | string | Relative path to source file from project root. Empty string for [stubs](glossary.md#stub). |
| `code-text` | object | `{"lines-start": N, "lines-end": N}` (1-based, inclusive). `{0, 0}` for stubs. |
| `kind` | string | Language-specific classification (see below) |
| `language` | string | `"rust"`, `"verus"`, `"lean"`, `"latex"` |

### Kind values

| Language | Values | Notes |
|----------|--------|-------|
| Rust (standard) | `exec` | Always `exec` for non-Verus Rust |
| Rust (Verus) | `exec`, `proof`, `spec` | `exec` = compiled+verified, `proof` = verified+erased, `spec` = specification+erased |
| Lean | `def`, `theorem`, `abbrev`, `class`, `structure`, `inductive`, `instance`, `axiom`, `opaque`, `quot` | Maps to Lean declaration kinds |

### Language assignment for Verus atoms

For probe-verus output, `language` is determined by `kind`, not by lexical scope:

| `kind` | `language` | Rationale |
|--------|------------|-----------|
| `exec` | `"rust"` | Exec functions are Rust code, even when annotated with Verus specs inside `verus!{}` blocks |
| `proof` | `"verus"` | Proof functions are Verus-only constructs (erased at compilation) |
| `spec` | `"verus"` | Spec functions are Verus-only constructs (erased at compilation) |

See [P20](properties.md#p20-language-is-derived-from-kind-not-lexical-scope).

Note: `"latex"` appears as a reserved `source.language` value in some envelope examples. No probe tool currently handles LaTeX — this is a placeholder for potential future support. Do not implement LaTeX handling without a dedicated tool and KB entry.

### Common optional fields

| Field | Type | Tools | Description |
|-------|------|-------|-------------|
| `primary-spec` | string | probe-verus, probe-lean | Primary specification text (verus) or code-name of primary spec theorem (lean) |
| `verification-status` | string | probe-verus, probe-lean, probe-aeneas | `"transitively-verified"`, `"verified"`, `"failed"`, `"unverified"`, or `"trusted"`. After enrichment (P23): `"transitively-verified"` = all transitive deps verified/trusted; `"verified"` = locally verified only. |
| `trusted-reason` | string | probe-verus, probe-lean | Present only when `verification-status` is `"trusted"`. probe-verus: `"admit"`, `"external-body"`, `"assume-specification"`. probe-lean: `"axiom"`, `"external"`. |
| `is-disabled` | bool | probe-verus, probe-rust, probe-aeneas | Whether excluded from analysis scope |
| `specs` | array of strings | probe-lean | Theorem atoms referencing this atom |
| `dependencies-with-locations` | array of objects | probe-verus, probe-rust | Per-call location data: `{code-name, location, line}` |

### Tool-specific extension fields

Extensions are stored in a flat `extensions` map in Rust types but serialized as top-level JSON fields alongside core fields.

**probe-verus extensions**:
- `requires-dependencies` — functions called in `requires` clauses
- `ensures-dependencies` — functions called in `ensures` clauses
- `body-dependencies` — functions called in function body
- (`dependencies` = union of all three)

**probe-lean extensions**:
- `type-dependencies` — from declaration's type signature
- `term-dependencies` — from body/proof term
- (`dependencies` = deduplicated union of type + term)
- `is-in-package`, `is-relevant`, `is-hidden`, `is-extraction-artifact`, `is-ignored` — filtering flags
- `rust-source` — Rust source path from Aeneas docstring (null if not Aeneas project)
- `attributes` — Lean tag attributes (e.g. `["primary_spec"]`)

**probe-aeneas extensions** (on merged atoms):
- `translation-name` — corresponding name in other language
- `translation-path` — file path of translation
- `translation-text` — line range of translation
- `is-disabled` — computed from functions.json
- `is-public` — Rust item visibility: `true` if declared `pub` per Charon, `false` if private or visibility data unavailable (set on all Rust atoms; preserved from probe-rust when present, defaulted to `false` when absent)

**probe-rust extensions**:
- `rust-qualified-name` — Charon-derived fully qualified name (optional, with Charon enrichment: `--with-charon` or `--translation`)
- `is-public` — whether the Rust item is declared `pub` per Charon LLBC (optional, with `--with-charon`; absent when Charon not used or match failed)
- `charon-def-id` — the charon `FunDeclId` for this function; equals Aeneas's `translation.json` `def_id`, enabling a precise integer Rust↔Lean join (optional, with Charon enrichment; always emitted together with `charon-version`)
- `charon-version` — the charon version that produced `charon-def-id`; provenance-gates the def-id join (optional; emitted together with `charon-def-id`, both or neither)

## Code-name URI format

Code-names are the primary key for atoms. They are URIs that uniquely identify a definition.

### Rust code-names

Format: `probe:<crate>/<version>/<module-path>/<Type>#<Trait><TypeParam>#<method>()`

Examples:
- `probe:curve25519-dalek/4.1.3/field/reduce()` — free function
- `probe:curve25519-dalek/4.1.3/field/FieldElement51#square()` — inherent method
- `probe:curve25519-dalek/4.1.3/scalar/Scalar#Add<&Scalar>#add()` — trait impl method
- `probe:core/https://github.com/rust-lang/rust/library/core/option/impl#map()` — stdlib

### Lean code-names

Format: `probe:<FullyQualifiedName>`

Examples:
- `probe:ArkLib.SumCheck.Protocol.Prover.prove`
- `probe:Mathlib.Data.Nat.Basic.succ_pos`

Lean code-names do not embed version because Lean projects don't reliably have semver versions and the namespace hierarchy already encodes the package prefix.

## Stubs

An atom is a [stub](glossary.md#stub) when all three conditions hold:
- `code-path` is `""`
- `code-text.lines-start` is `0`
- `code-text.lines-end` is `0`

Stubs represent external dependencies referenced but not analyzed. They have `dependencies: []`. During merge, real atoms replace stubs with the same code-name.

## Merge algorithm

See [properties.md](properties.md) for the invariants merge must satisfy.

### Atoms: first-wins with stub replacement

| Base entry | Incoming entry | Action |
|-----------|---------------|--------|
| stub | real | **Replace**: incoming wins |
| real | real | **Conflict**: keep base, emit warning |
| stub | stub | Keep base |
| real | stub | Keep base |
| (absent) | any | **Add** |

### Specs and proofs: last-wins

| Base entry | Incoming entry | Action |
|-----------|---------------|--------|
| any | any (same key) | **Replace**: incoming wins |
| (absent) | any | **Add** |

### Cross-language mappings

When `--mappings <file>` is provided to `probe merge`:
- For each atom's dependencies, if a dependency has a mapping, the mapped code-name(s) are added as additional dependencies
- Both directions are checked (from→to and to→from)
- A single source may map to multiple targets (1-to-many)
- Each target must exist in the merged key set
- Each target must not already be a dependency

### Normalization

Before merging, all code-name keys and dependency references are normalized: trailing `.` characters are stripped (legacy verus-analyzer artifact).

## Mappings file format

Schema: `probe/mappings`. Contains bidirectional mappings between code-names across languages.

```json
{
  "schema": "probe/mappings",
  "schema-version": "2.0",
  "tool": { "name": "probe-aeneas", "version": "...", "command": "translate" },
  "timestamp": "...",
  "sources": {
    "from": { "schema": "probe-verus/atoms", "package": "...", "package-version": "..." },
    "to": { "schema": "probe-lean/extract", "package": "...", "package-version": "..." }
  },
  "mappings": [
    { "from": "probe:crate/1.0/mod/fn()", "to": "probe:Pkg.Mod.fn", "confidence": "exact", "method": "rust-qualified-name" }
  ]
}
```

Confidence levels: `exact`, `exact-disambiguated`, `file-and-name`, `file-and-lines`, `heuristic`.

See also: [mappings-spec.md](../../docs/mappings-spec.md) for the full format specification and [ADR-003](../decisions/003-mappings-design.md) for design rationale.

## Projection metadata

When `probe project` produces output, it reuses the `probe/merged-atoms` schema but adds a `projection` metadata block at the envelope level:

```json
{
  "projection": {
    "mappings-file": "mappings.json",
    "seeds": 10,
    "forward-depth": 3,
    "reverse-depth": 0,
    "atoms-in": 1261,
    "atoms-out": 67,
    "deps-trimmed": 42
  }
}
```

This field is accommodated by `additionalProperties: true` on the merged envelope and is ignored by consumers that don't know about it. See [probe-project.md](../tools/probe-project.md) for full details.

## Versioning

- **Major** (e.g. 2.0 → 3.0): Changes to required fields, field semantics, or field removals
- **Minor** (e.g. 2.0 → 2.1): New optional fields, new `kind` values
- Consumers validate `schema-version` starts with expected major version (currently `"2."`)

### Version history

| Version | Tool | Changes |
|---------|------|---------|
| 2.0 | all | Initial Schema 2.0 envelope format |
| 2.1 | probe-rust | Added optional `rust-qualified-name`, `is-disabled`, and `is-public` fields to atoms |

## Package versioning by language

| Language | Strategy | Example |
|----------|----------|---------|
| Rust (Cargo) | Use crate's semver version | `"4.1.3"` |
| Lean (Lake) | `version` from `lakefile.toml` if present; else short git commit hash | `"0.1.0"` or `"a1b2c3d"` |
| LaTeX | Short git commit hash | `"a1b2c3d"` |
