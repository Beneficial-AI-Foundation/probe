---
title: Schema 3.1 Interchange Specification
last-updated: 2026-09-30
status: draft
---

# Schema 3.1 Interchange Specification

This is the authoritative specification for the JSON interchange format shared by all probe tools. Per-tool `docs/SCHEMA.md` files document tool-specific details; this file defines the contract they all share.

The current version is **3.1** (hub-side minor bump: correspondence records and `status-origin`, see the [version history](#version-history)). 3.x is additive — producers may keep emitting `3.0`; consumers accept any `3.x`.

## Envelope

Every probe output file is wrapped in a metadata envelope:

```json
{
  "schema": "probe-verus/extract",
  "schema-version": "3.0",
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
| `source.language` | string | yes | `"rust"`, `"lean"` |
| `source.package` | string | yes | Crate/project name |
| `source.package-version` | string | yes | Version identifier (semver for Rust; commit hash for Lean if no version) |
| `timestamp` | string | yes | ISO 8601 |
| `data` | object | yes | Payload. Structure depends on `schema`. |

### Merged envelope variant

When `probe merge` produces output, `source` is replaced by `inputs`:

```json
{
  "schema": "probe/merged-atoms",
  "schema-version": "3.1",
  "tool": { "name": "probe", "version": "0.5.0", "command": "merge" },
  "inputs": [
    { "schema": "probe-verus/atoms", "source": { ... } },
    { "schema": "probe-lean/atoms", "source": { ... } }
  ],
  "timestamp": "...",
  "data": { ... }
}
```

When a previously merged file is used as input, its `inputs` entries are flattened into the new output — provenance is carried forward recursively. Provenance is a **deduplicated source inventory**: the `inputs` array records *which* sources were composed, not how many times ([P9](properties.md#p9-provenance-is-preserved)).

**Composed-provenance detection is structural**, not schema-string-based: an envelope with an `inputs` array is composed, one with a `source` object is single-tool. Producers other than the hub may emit composed envelopes under their own schema strings (e.g. `probe-aeneas/extract` with `inputs: [Rust, Lean]` and no `source`); loaders must preserve their inventories ([ADR-006](../decisions/006-correspondence-records.md)). An envelope carrying **both** `source` and `inputs` is ambiguous and rejected at load (it matches neither branch of the executable schema), and a composed envelope's `inputs` must be a non-empty well-formed array — malformed or empty inventories are load errors, never silently emptied or propagated.

### Registered schema values

**Single-tool schemas**:
- `probe-rust/extract`
- `probe-verus/atoms`, `probe-verus/extract`, `probe-verus/specs`, `probe-verus/proofs`, `probe-verus/stubs`, `probe-verus/verification-report`
- `probe-lean/extract`, `probe-lean/viewify`
- `probe-aeneas/extract`
- `probe-leanblueprint/extract`
- `probe-vcvio/extract`

Note: Legacy schema values `probe-lean/atoms`, `probe-lean/enriched-atoms`, `probe-lean/specs`, `probe-lean/proofs`, `probe-lean/stubs` exist from Schema 1.x and may appear in older files or as input sources in merged envelopes. Current probe-lean only produces `probe-lean/extract` and `probe-lean/viewify`.

**Merged schemas**:
- `probe/merged-atoms`, `probe/merged-specs`, `probe/merged-proofs`

**Projected views**:
- `probe/projected-atoms` — output of `probe project`. A projection is a **view** of an authoritative graph, not an authoritative artifact: its `dependencies` are trimmed to the included set, so recomputing enrichment over it would launder truncated views into stronger labels. `probe merge` and `probe enrich` **reject** any input carrying this schema — or, for legacy projections, the `probe/merged-atoms` schema plus a `projection` envelope field ([ADR-006](../decisions/006-correspondence-records.md), [P23](properties.md#p23-transitive-verification)). Read-only consumers (`probe summary`, `probe project`, probegraph) accept it; `detect_category` classifies it as atoms.

**Analysis**:
- `probe/summary`
- `probe-leanblueprint/summary` — node-aggregated two-axis blueprint progress counts (aggregated over blueprint nodes, not keyed per node; sidecar; not an atoms/specs/proofs category, so never merged)

**Special**:
- `probe/mappings` — cross-language mappings

### Schema categories

The `schema` field implicitly identifies the data category:

| Category | Matches | Merge strategy |
|----------|---------|---------------|
| **Atoms** | `*/atoms`, `*/enriched-atoms`, `*/extract`, `probe/merged-atoms`, `probe/projected-atoms` (read-only consumers; rejected by merge/enrich) | First-wins with stub replacement |
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
| `language` | string | `"rust"`, `"verus"`, `"lean"`, `"blueprint"` |

### Kind values

| Language | Values | Notes |
|----------|--------|-------|
| Rust (standard) | `exec` | Always `exec` for non-Verus Rust |
| Rust (Verus) | `exec`, `proof`, `spec` | `exec` = compiled+verified, `proof` = verified+erased, `spec` = specification+erased |
| Lean | `def`, `theorem`, `abbrev`, `class`, `structure`, `inductive`, `instance`, `axiom`, `opaque`, `quot` | Maps to Lean declaration kinds |
| Blueprint (synthetic) | `blueprint-definition`, `blueprint-theorem` | probe-leanblueprint planned atoms — blueprint nodes with no Lean binding yet (`language: "blueprint"`) |

### Language assignment for Verus atoms

For probe-verus output, `language` is determined by `kind`, not by lexical scope:

| `kind` | `language` | Rationale |
|--------|------------|-----------|
| `exec` | `"rust"` | Exec functions are Rust code, even when annotated with Verus specs inside `verus!{}` blocks |
| `proof` | `"verus"` | Proof functions are Verus-only constructs (erased at compilation) |
| `spec` | `"verus"` | Spec functions are Verus-only constructs (erased at compilation) |

The derivation rule and its rationale are owned by probe-verus:
[docs/SCHEMA.md § Language assignment](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#language-assignment).

### Common optional fields

| Field | Type | Tools | Description |
|-------|------|-------|-------------|
| `primary-spec` | string | probe-verus, probe-lean | Primary specification text (verus) or code-name of primary spec theorem (lean) |
| `verification-status` | string | probe-verus, probe-lean, probe-aeneas | `"transitively-verified"`, `"verified"`, `"failed"`, `"unverified"`, or `"trusted"`. After enrichment ([P23](properties.md#p23-transitive-verification)): `"transitively-verified"` = locally verified and no **seed** (explicit `"failed"`/`"unverified"` atom, or any `status-origin`-bearing atom) is reachable along a dependency path that does not pass through a trusted boundary; `"verified"` = locally verified only. Labels assert consistency with the graph they were computed on; they are not proof re-validation. |
| `trusted-reason` | string | probe-verus, probe-lean | Present only when `verification-status` is `"trusted"`. probe-verus: `"admit"`, `"external-body"`, `"assume-specification"`. probe-lean: `"axiom"`, `"external"`. |
| `status-origin` | string | probe-aeneas, probe-lean | Evidence marker on `verification-status`: `"translation"` (status copied from a corresponding atom in another language — imported evidence) or `"kernel-taint"` (probe-lean: the kernel-level taint walk found reachable taint the emitted graph cannot express). Enrichment treats every bearing atom as a **blocker seed**: never promoted to `"transitively-verified"`, unconditionally demoted from an imported `"transitively-verified"`, and blocking promotion of atoms that reach it along a non-trusted path. A `"trusted"` atom carrying `status-origin` is **not** a trust boundary. Enforcement is fail-closed: the enrichment predicates are presence-based (any bearing atom seeds, whatever the value), and `probe merge`/`probe enrich`/`probe summary`/`probe project` reject out-of-enum or non-string values at their load boundaries — the executable schema constrains the marker, but runtime inputs are not schema-validated. See [ADR-006](../decisions/006-correspondence-records.md). |
| `maps-to` / `mapped-from` | array of records | probe (merge) | Correspondence records attached by `probe merge --mappings`; see [Correspondence records](#correspondence-records-maps-to-mapped-from). |
| `untracked` | bool | probe-verus, probe-rust, probe-aeneas | Whether excluded from analysis scope |
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
- `untracked` — verification scope ([P25](properties.md#p25-atoms-not-in-the-verification-build-are-out-of-scope)), computed from probe-rust source facts and the Lean translation's attributes. **Not** from `functions.json`: absence from it means untranslated backlog, not out of scope. `untracked-reason` names the cause.
- `is-public` — Rust item visibility: `true` if declared `pub` per Charon, `false` if private or visibility data unavailable (set on all Rust atoms; preserved from probe-rust when present, defaulted to `false` when absent)

**probe-leanblueprint extensions** (on enriched Lean atoms and synthetic planned atoms):
- `blueprint-label` — blueprint node label
- `blueprint-kind` — blueprint node kind: `definition`/`theorem` (recovers node kind on bound atoms whose `kind` is the Lean kind)
- `blueprint-statement-status` — statement axis: `none`/`blocked`/`ready`/`formalized`
- `blueprint-proof-status` — proof axis: `none`/`ready`/`proved`/`fully-proved`
- `blueprint-status-source` — `code-derived` (Verso) or `declared` (Massot `\leanok`)
- `blueprint-group`, `blueprint-chapter`, `blueprint-title`, `blueprint-discussion` — sub-construction group, chapter, display title, GitHub issue (all optional)
- `blueprint-statement-uses`, `blueprint-proof-uses` — code-names used by the statement/proof (informal roadmap edges; never merged into `dependencies`)
- `blueprint-status-mismatch` — set when the blueprint over-claims a proof vs the machine `verification-status` (owned by probe-leanblueprint; see [properties.md § Single-probe invariants](properties.md#single-probe-invariants-owned-by-each-probes-repo))
- `blueprint-decl-missing` — `true` when **all** bound Lean decls are absent from the atom set (synthetic planned node)
- `blueprint-missing-decls` — for a bound node, the subset of `\lean{...}` decls absent from the atom set (partial miss; recorded on the present atom(s))

Blueprint fields are additive: `verification-status` remains probe-lean's machine value (a probe-leanblueprint invariant, see [properties.md § Single-probe invariants](properties.md#single-probe-invariants-owned-by-each-probes-repo)).

### Correspondence records (maps-to, mapped-from)

Attached by `probe merge --mappings` ([ADR-006](../decisions/006-correspondence-records.md), superseding the edge-injection semantics of [ADR-003](../decisions/003-mappings-design.md)). A *correspondence* ("there is a mappings-file entry linking these names, with this confidence") is a different relation from a *dependency* (calls, or proof-uses) and is never written into `dependencies`.

Extension fields are flattened, so the records appear as top-level atom fields. On the mapping's `from` atom:

```json
{
  "code-name": "probe:crate/1.0/mod/g()",
  "maps-to": [
    { "target": "probe:Pkg.Mod.ga", "confidence": "exact", "method": "rust-qualified-name" }
  ]
}
```

On the `to` atom, the mirror field `mapped-from` with `"target"` pointing back.

- **Record fields**: `target` (required), `confidence` (required, the mappings-file vocabulary: `exact`, `exact-disambiguated`, `file-and-name`, `file-and-lines`, `heuristic`, `manual`), `method` (optional; omitted when absent — an empty `method` is canonicalized to absent, and no other fields are allowed). Record shape is validated fail-closed at every recomputation boundary ([P27](properties.md#p27-correspondence-records-are-unioned-and-inert)): merge re-emits these fields and must not violate this schema.
- **Direction**: the mappings file's `from`/`to` are generic source/target — they assign no implementation/formal roles. `maps-to` goes on the `from` atom, `mapped-from` on the `to` atom, whichever languages the sides are.
- **Determinism** ([P14](properties.md#p14-deterministic-output)): arrays sorted by `(target, confidence, method)`, absent `method` ordering as the empty string. Record identity is the same triple; duplicates collapse by that identity. Records with the same target but different confidence/method are distinct assertions and both kept.
- **Attachment is unconditional and key-local** ([P13](properties.md#p13-correspondence-records-attach-unconditionally)): a record attaches whether or not its target exists in the invocation's key set (dangling target ⇒ warning, not skip).
- **Union through conflicts** ([P27](properties.md#p27-correspondence-records-are-unioned-and-inert)): on every equal-key merge resolution the surviving atom carries the set union of both sides' records.
- **Inert to enrichment**: correspondence records never participate in contamination/promotion BFS ([P23](properties.md#p23-transitive-verification)).
- **Mirrors are best-effort**: the correspondence relation is defined as the union over both fields; a missing mirror (target absent at attachment time) loses no information. Regenerating the merge restores mirrors.
- Cross-language *resolution* (treating `g` and `ga` as one node) is a derived consumer view over the records; a derived `verified-by-translation` status is deferred to a future ADR.

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

When `--mappings <file>` is provided to `probe merge`, mappings attach [correspondence records](#correspondence-records-maps-to-mapped-from) — they never modify `dependencies`:
- For each mapping entry `from → to`, the `from` atom (if present in the merged map) gets a `maps-to` record and the `to` atom (if present) gets a `mapped-from` record
- Attachment is unconditional and key-local ([P13](properties.md#p13-correspondence-records-attach-unconditionally)): a dangling target is warned about, never skipped
- A single source may map to multiple targets (1-to-many); each target yields its own record
- Mapping-file endpoints are normalized ([P8](properties.md#p8-code-name-normalization)) before lookup

### Authority validation and re-enrichment

`probe merge` and `probe enrich` validate every input envelope's authority before recomputing over it ([ADR-006](../decisions/006-correspondence-records.md)):

- **Projection rejection**: inputs carrying `probe/projected-atoms`, or the legacy form (`probe/merged-atoms` plus a `projection` envelope field), are errors — projections are views; deleting edges must not improve assurance.
- **Version gate**: atoms envelopes whose `tool.name` is in the per-producer gate table with `tool.version` below that producer's contract threshold are errors (pre-contract artifacts can carry unmarked imported or graph-inexpressible evidence). The `probe` entry is an interval (`threshold ≤ version < 1.0.0`) and additionally rejects `tool.command: "merge-atoms"` at any version. `probe summary` and `probe project` run the version-gate component too.
- **Validator edge cases** (fail-closed): missing or malformed `tool` metadata on an atoms envelope is a rejection, as is an unparsable `tool.version` on a gated tool name (versions parse as numeric `major.minor[.patch]` only — pre-release suffixes do not parse). The legacy-projection predicate is presence-based: `projection: null` counts as present. Unknown `tool.name`s pass at any version ([ADR-006](../decisions/006-correspondence-records.md) Decision 7's audited-population scope), and non-atoms categories (specs/proofs) are not gated. Unknown schema strings keep their ordinary category-detection error.

After combining all inputs, merge **re-enriches** the atoms category via the shared enrichment recomputation ([P23](properties.md#p23-transitive-verification)) — stub resolution can invalidate labels computed at extract time, so merged output labels are recomputed, never inherited.

### Normalization

Per input, before conflict resolution, all code-name keys and dependency references are normalized: trailing `.` characters are stripped (legacy verus-analyzer artifact). Normalization covers code-name-bearing extension arrays, correspondence-record targets, and mapping endpoints ([P8](properties.md#p8-code-name-normalization)). The per-input ordering is semantic: aliases collapse within their own input before cross-input conflicts are resolved. A post-normalization collision between distinct real atoms (ignoring correspondence records) within one input is a merge **error**, the same fail-closed rule `probe enrich` applies; benign collapses (stub, identical-modulo-records) union their correspondence records ([P27](properties.md#p27-correspondence-records-are-unioned-and-inert)).

## Mappings file format

Schema: `probe/mappings`. Contains bidirectional mappings between code-names across languages.

```json
{
  "schema": "probe/mappings",
  "schema-version": "3.0",
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

The `sources` block describes each side of the mappings (`schema`, `package`, `package-version`). `from`/`to` are generic source/target roles — they assign no implementation/formal roles (a Lean→Rust file is legal). Multiple entries with the same `from` key are allowed (1-to-many): one implementation may correspond to several formal constructs.

Each mapping entry carries a required `confidence` and an optional `method` (a finer description of the matching method, e.g. `"rust-qualified-name"`, `"file+display-name"`). `confidence` is validated against the vocabulary below at load and at the merge boundary, fail-closed — merge writes it into correspondence records, and an unchecked typo would make merge emit output violating the executable schema. An empty `method` is canonicalized to absent at load ([P27](properties.md#p27-correspondence-records-are-unioned-and-inert)):

| Confidence | Meaning |
|------------|---------|
| `exact` | Matched via `rust-qualified-name` or equivalent deterministic key |
| `exact-disambiguated` | Matched via `rust-qualified-name` with file/line disambiguation |
| `file-and-name` | Matched via same source file + display-name overlap |
| `file-and-lines` | Matched via same source file + overlapping line ranges |
| `heuristic` | Matched via fuzzy heuristics (lower confidence) |
| `manual` | Manually authored mapping |

**Folder convention**: mappings files live in `.verilib/mappings/`, named `<from_tool>_<from_package>__<to_tool>_<to_package>.json` (e.g. `lean_SecureMessaging__rust_libsignal-protocol.json`).

**Generation**: any tool with access to both probe outputs can generate a mappings file — probe-aeneas `translate` (three-strategy matching for Aeneas-transpiled projects), manual authoring (cross-language linking without a transpilation relationship), `rust-qualified-name` joins, or file + line matching.

See [ADR-003](../decisions/003-mappings-design.md) for generation rationale and [ADR-006](../decisions/006-correspondence-records.md) for application semantics ([correspondence records](#correspondence-records-maps-to-mapped-from), attached per the [merge rules](#cross-language-mappings) above).

## Projection metadata

`probe project` writes the distinct `probe/projected-atoms` schema (see [Registered schema values](#registered-schema-values)) and adds a `projection` metadata block at the envelope level:

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

The distinct schema string is an **authority boundary**, not a hint: `probe merge` and `probe enrich` error on projected inputs (in both the new and the legacy `probe/merged-atoms`+`projection` form), so a consumer routing through envelope validation fails loudly instead of silently treating a view as authoritative. Verification labels inside a projection describe the **original** graph: `probe project` recomputes enrichment on the full authoritative input graph *before* trimming, and never recomputes from the trimmed view (already-projected inputs stay readable with labels untouched). See [probe-project.md](../tools/probe-project.md) and [ADR-006](../decisions/006-correspondence-records.md).

## Versioning

- **Major** (e.g. 2.0 → 3.0): Changes to required fields, field semantics, or field removals
- **Minor** (e.g. 2.0 → 2.1): New optional fields, new `kind` values
- Consumers validate `schema-version` starts with expected major version (currently `"3."`)

### Version history

| Version | Tool | Changes |
|---------|------|---------|
| 2.0 | all | Initial Schema 2.0 envelope format |
| 2.1 | probe-rust | Added optional `rust-qualified-name`, `is-disabled`, and `is-public` fields to atoms |
| 3.0 | all | **Breaking**: renamed atom field `is-disabled` → `untracked` (identical semantics: `untracked: true` = out of verification scope). Unified every producer on `schema-version` `3.0`. |
| 3.1 | probe (hub) | Added optional `maps-to`/`mapped-from` correspondence records and the `status-origin` marker; added the `probe/projected-atoms` schema. Hub-side only — producers keep emitting 3.0; the behavioral change (no cross-language edges in `dependencies`) is coordinated through the [ADR-006](../decisions/006-correspondence-records.md) rollout, not the schema number. |

### Bumping the interchange schema-version (major)

The `schema-version` major is a **cross-repo contract**: every producer stamps it, every consumer validates it (`schema_version.starts_with("<major>.")` in `parse_envelope`, `src/types.rs` — the single load path all hub commands route through). A major bump is breaking and must land **in lockstep across the ecosystem** — a partial bump makes atom-loading fail with `… incompatible schema-version "X.0" (expected <major>.x)`.

Checklist for a major bump (N → N+1):

1. **Hub (this repo).** Move the gate to `starts_with("N+1.")` in `parse_envelope` (`src/types.rs`); bump the version emitted by `merge`/`summary`/`project`; update this file and the version-history table above. Cut a **tagged release** — downstream pins tags, not `main`.
2. **Producers** (`probe-rust`, `probe-lean`, `probe-verus`, `probe-leanblueprint`, `probe-aeneas`). Change the emitted `schema-version` to N+1; pin the hub dep to the new tag; relock; cut a **tagged release** each.
3. **Consumers** (`probe-aeneas`, `probe-verus`). Pin the hub tag — this is the validator — and ensure the sub-extractors they invoke emit N+1 (probe-aeneas installs `probe-rust`/`probe-lean` unpinned, see [probe-aeneas#53](https://github.com/Beneficial-AI-Foundation/probe-aeneas/issues/53)); relock; release.
4. **Images / verilib.** Rebuild every ECR image from the new releases, repoint verilib, and build `--locked` so a dependency can't silently float.
5. **Verify** end-to-end: run atomization on real N+1 atoms and confirm they load.

Gotchas (from the 2 → 3 bump):

- **Pin the hub by `tag`, never a floating `git` dep.** Otherwise released/ECR builds freeze whatever validator their lockfile held while local `cargo install` floats to `main` — the "works locally, breaks in verilib" split.
- **Emit and validate must ship in the same release.** probe-aeneas 0.17.0 emitted `3.0` but linked a `2.x` validator, so it rejected its own schema.
- **Don't rely on gitignored `.cargo/config.toml` path-patches** (`probe = { path = "../probe" }`) — they mask the mismatch locally.

## Package versioning by language

| Language | Strategy | Example |
|----------|----------|---------|
| Rust (Cargo) | Use crate's semver version | `"4.1.3"` |
| Lean (Lake) | `version` from `lakefile.toml` if present; else short git commit hash | `"0.1.0"` or `"a1b2c3d"` |
