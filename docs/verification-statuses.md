# Verification Statuses

Defines the per-atom status fields (from the tool schemas) and the color scheme derived from them. Color counts are produced by [`scripts/count-colors.sh`](../scripts/count-colors.sh), which this document and the script must agree on — the script is currently out of date with the scheme below and will be reconciled in a follow-up PR (see [Counting](#counting)).

## Atom kinds

| Kind | Description | Examples |
|------|-------------|----------|
| **Implementation** | Executable code that can have specs attached | Rust functions, Verus exec-defs, Aeneas-generated Lean `def`s |
| **Specification** | Logical statements that define or prove properties | Verus spec-defs and `proof fn`, Lean `theorem`/`lemma`, non-translation `def`s |

Implementations can have specs attached; specifications cannot — they *are* the specs (always `unspecified`).

## Status fields

### `verification-status`

| Value | Meaning |
|-------|---------|
| `transitively-verified` | Verified, and every transitive dependency is verified or trusted ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)) |
| `verified` | Verified locally, but some transitive dependency is `unverified`/`failed` |
| `unverified` | Has sorries, admits, or warnings |
| `failed` | Compile/verification errors |
| `trusted` | Axiomatically assumed (`axiom`, `#[verifier::external_body]`, `admit()`) |
| `null` | Not subject to verification (tests, constants, external stubs) |

The `transitively-verified` vs `verified` split is computed by `probe enrich` (reverse-BFS contamination); probe-verus and probe-aeneas run it as the last step of `extract`.

What `"verified"` asserts is defined by each tool's schema and differs by pipeline:

- **Aeneas/Lean** — [derived from the primary spec theorem](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md#rust-specific-fields): the spec's status *is* the function's status; no spec ⇒ `"unverified"`. So `"verified"` always implies a proven spec.
- **Verus** — [mapped from the proof run](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#verification-status-mapping) (`success → "verified"`), independent of spec presence.

### `specified`

An implementation is `specified` if its `specs` list is non-empty, else `unspecified`. The signal is pipeline-specific: probe-lean uses the atom's own `specs`; probe-verus emits no `specs` array, so `is-disabled: false` (i.e. the function has `requires`/`ensures`) is the equivalent "has a spec" signal; a probe-aeneas Rust atom holds no spec of its own, so "specified" is read off its Lean translation (the atom named by `translation-name`).

## Colors

Coloring is two steps. **(1) Identify the producing tool** — for a single-tool `extract` file this is the envelope `schema`; it picks the column group. **(2) Within that tool, the per-atom fields decide the color**: `language`, `kind`, `rust-source`, `is-disabled` / `primary-spec` / `specs`, and `verification-status`. (`language` discriminates the columns *except* for Verus, where `kind` is authoritative — see Notes. The Light vs Dark Blue split additionally needs VeriLib validation state, also in Notes.)

Merged (`probe/merged-atoms`) files carry **no per-atom producer field** (only envelope-level `inputs`), so the tool must be inferred per atom: a `translation-name` marks an Aeneas Rust atom; a `language: "rust"` `exec` atom carrying `verification-status`/`primary-spec` but no translation is Verus; a `language: "rust"` `exec` atom with none of these is pure-Rust → Grey. Everything else follows from `language` + `kind` + `rust-source` regardless of tool.

**Excluded before coloring** (never assigned a color): external-crate stubs (`code-path: ""`) and atoms flagged `is-hidden`, `is-ignored`, or `is-extraction-artifact`. A pure-Rust project (no verification framework) leaves every atom Grey.

**Assumes full `extract` output.** VeriLib runs each tool with no `--skip-*` flags, so `primary-spec` (Verus), `specs` (Lean), `is-disabled`, and `verification-status` are populated. An absent `verification-status` therefore means the atom is *inherently* not verification-tracked (e.g. a Lean type declaration like `structure`/`class` → Grey) or a pure-Rust atom — never a skipped pipeline step.

### Roles

Each colored atom is either an *implementation* or a *specification*:

- **Implementation** — executable code that can carry a spec: a Rust `kind: "exec"` atom, or an Aeneas-translated Lean atom (`kind` `def`/`abbrev` with `rust-source` non-null). Uses the full ladder Grey → White → Yellow → Light Blue → Dark Blue → Light Green → Dark Green.
- **Specification** — everything else that is colored: Verus `kind: "proof"` or `"spec"`; **every** atom in a pure-Lean project (`rust-source: null`, any `kind`); and Lean spec kinds (`theorem`, `axiom`, …) in an Aeneas project even when `rust-source` is non-null. Colored *directly* by `verification-status` — only Yellow / Light Green / Dark Green / Red / Purple, never the White/Blue ladder (a specification is not itself "specified").

### Decision order

To color an atom, evaluate top to bottom; the **first match wins**:

1. **Excluded** (above) → no color.
2. `verification-status` is `"trusted"` → **Purple**.
3. `verification-status` is `"failed"` → **Red**.
4. Otherwise, by role:
   - **Specification** — `"transitively-verified"` → Dark Green; `"verified"` → Light Green; `"unverified"` → Yellow; absent → Grey.
   - **Implementation** — (a) **no spec** → White (Verus `is-disabled: true`, no `requires`/`ensures`) · Grey (Aeneas `is-disabled: true`, not translated; `#[test]`; pure-Rust; or `is-disabled`/`primary-spec` absent — untracked) · Yellow (Aeneas translated but its translation is unspecified); (b) **has spec, not validated\*\*\*** → Light Blue; (c) **has spec, validated\*\*\*** → Dark Green if `"transitively-verified"`, Light Green if `"verified"`, else Dark Blue (covers `"unverified"` and absent status).

The table is the per-column realization of this order. Steps 2–3 are **global overrides**: a `"trusted"`/`"failed"` atom is Purple/Red even though those rows sit at the bottom of the table. An unqualified quoted value (e.g. `"verified"`) is a `verification-status` value; any other condition names its field explicitly. `—` means the color is unreachable for that column.

<table>
<thead>
<tr>
<th rowspan="2">Color</th>
<th colspan="3"><code>probe-verus</code></th>
<th><code>probe-lean</code> (pure Lean)</th>
<th colspan="3"><code>probe-aeneas</code></th>
</tr>
<tr>
<th><code>exec</code><br>(implementation)</th>
<th><code>spec</code><br>(specification)</th>
<th><code>proof</code><br>(specification)</th>
<th>any <code>kind</code><br>(specification)</th>
<th><code>exec</code><br>(implementation)</th>
<th><code>def</code> translation*<br>(implementation)</th>
<th>non-translation<br>(specification)</th>
</tr>
</thead>
<tbody>
<tr>
<td><b>Grey</b></td>
<td><code>#[test]</code> (heuristic**)</td>
<td>no <code>verification-status</code></td>
<td>no <code>verification-status</code></td>
<td>no <code>verification-status</code></td>
<td><code>is-disabled: true</code> (not translated)</td>
<td>—</td>
<td>no <code>verification-status</code></td>
</tr>
<tr>
<td><b>White</b></td>
<td><code>is-disabled: true</code> (no <code>requires</code>/<code>ensures</code>)</td>
<td>—</td>
<td>—</td>
<td>—</td>
<td>—</td>
<td>—</td>
<td>—</td>
</tr>
<tr>
<td><b>Yellow</b></td>
<td>—</td>
<td><code>"unverified"</code></td>
<td><code>"unverified"</code></td>
<td><code>"unverified"</code></td>
<td><code>is-disabled: false</code> (translated), translation <code>specs</code> absent/empty</td>
<td><code>specs</code> absent/empty</td>
<td><code>"unverified"</code></td>
</tr>
<tr>
<td><b>Light Blue</b></td>
<td><code>is-disabled: false</code> (has spec), not validated***</td>
<td>—</td>
<td>—</td>
<td>—</td>
<td>translation <code>specs</code> non-empty, not validated***</td>
<td><code>specs</code> non-empty, not validated***</td>
<td>—</td>
</tr>
<tr>
<td><b>Dark Blue</b></td>
<td><code>is-disabled: false</code> + validated***; <code>"unverified"</code>/absent</td>
<td>—</td>
<td>—</td>
<td>—</td>
<td>translation <code>specs</code> non-empty + validated***; <code>"unverified"</code>/absent</td>
<td><code>specs</code> non-empty + validated***; not yet proven</td>
<td>—</td>
</tr>
<tr>
<td><b>Light Green</b></td>
<td><code>is-disabled: false</code> + validated***; <code>"verified"</code></td>
<td><code>"verified"</code></td>
<td><code>"verified"</code></td>
<td><code>"verified"</code></td>
<td>validated***; <code>"verified"</code></td>
<td><code>specs</code> non-empty + validated***; <code>"verified"</code></td>
<td><code>"verified"</code></td>
</tr>
<tr>
<td><b>Dark Green</b></td>
<td><code>is-disabled: false</code> + validated***; <code>"transitively-verified"</code></td>
<td><code>"transitively-verified"</code></td>
<td><code>"transitively-verified"</code></td>
<td><code>"transitively-verified"</code></td>
<td>validated***; <code>"transitively-verified"</code></td>
<td><code>specs</code> non-empty + validated***; <code>"transitively-verified"</code></td>
<td><code>"transitively-verified"</code></td>
</tr>
<tr>
<td><b>Red</b></td>
<td><code>"failed"</code></td>
<td><code>"failed"</code></td>
<td><code>"failed"</code></td>
<td><code>"failed"</code></td>
<td><code>"failed"</code></td>
<td><code>"failed"</code></td>
<td><code>"failed"</code></td>
</tr>
<tr>
<td><b>Purple</b></td>
<td><code>"trusted"</code></td>
<td><code>"trusted"</code></td>
<td><code>"trusted"</code></td>
<td><code>"trusted"</code></td>
<td><code>"trusted"</code></td>
<td><code>"trusted"</code></td>
<td><code>"trusted"</code></td>
</tr>
</tbody>
</table>

### Notes

- **`*` Roles and translation detection.** A Lean atom is an Aeneas *translation* (implementation) iff its `rust-source` is non-null **and** its `kind` is `def`/`abbrev`. Lean spec kinds (`theorem`, `axiom`, …) are **specifications** even when `rust-source` is non-null — role wins over `rust-source` for them. `rust-source: null` ⇒ hand-written. (Equivalently, the matched primary translation is the target of some `rust` atom's `translation-name`; `rust-source` is the intrinsic per-atom signal.)
- **`**` Verus `#[test]` → Grey is a heuristic.** There is no canonical test field in the JSON; consumers detect tests by attribute/name (see [Open questions](#open-questions)).
- **`***` Validation (Light Blue vs Dark Blue).** Validation is *not* recorded in the probe JSON; it is a VeriLib **frontend** action — a user presses a "validate" button to approve a spec as correct for its function. **Default: an attached spec is treated as validated**, so from the probe JSON alone every attached spec clears the validation gate. After that the proof state decides: proven → Green; not-yet-proven → **Dark Blue**. Concretely, **an implementation with a spec but an incomplete proof (`verification-status: "unverified"`/sorry, or absent) is Dark Blue** — validated but not yet proven — and becomes Green only once `"verified"`/`"transitively-verified"`. **Light Blue never arises from the JSON alone**; it appears only when VeriLib explicitly marks a spec *not* validated (the validation gate precedes the proof check, so an unvalidated-but-proven spec is Light Blue, not Green).
- **Verus columns key on `kind`, not `language`.** Within `probe-verus`, the column is chosen by `kind` (`exec` / `proof` / `spec`). Do **not** key on `language`: [P20](../kb/engineering/properties.md#p20-language-is-derived-from-kind-not-lexical-scope) specifies `proof`/`spec` → `language: "verus"`, but some checked-in fixtures still emit `language: "rust"` for them — that inconsistency is tracked separately.
- **Pure-Lean projects.** In a `probe-lean` project every atom has `rust-source: null`, so all are specifications colored directly by `verification-status` — this is why a transitively-verified Lean `def` is Dark Green, not White.
- **Aeneas Rust atoms hold no spec of their own.** A `probe-aeneas` Rust `exec` atom carries no `specs`/`primary-spec`; the spec lives on its Lean translation, and the Rust atom's `verification-status` is *derived* from that translation's primary spec (no spec ⇒ `"unverified"`). So a Rust `exec` is keyed on its own `verification-status`; the Yellow vs Dark Blue split (both `"unverified"`) is decided by whether its translation is specified (translation `specs` non-empty). **If `translation-name` does not resolve to a present Lean atom, treat it as having no spec → Yellow.**
- **`specs` absent or empty = no spec.** Lean omits `specs` when empty, so "no spec" means `specs` **absent or empty** — treat both as unspecified; do not read absent as "unknown". This is the `specified` signal ([P18](../kb/engineering/properties.md#p18-lean-specified-is-derived-not-stored)): Lean uses `specs` non-empty, Verus uses `is-disabled: false` (equivalently a non-empty `primary-spec`), and an Aeneas Rust atom reads it off its translation.
- **`null` status = absent.** A `verification-status` of `null` is treated identically to an absent `verification-status` (→ Grey for specifications; untracked for implementations).
- **Yellow is context-dependent.** In specification columns Yellow = `"unverified"` (a sorry / incomplete proof). In the Aeneas implementation columns Yellow = translated but unspecified. (A Verus implementation with no spec is White, not Yellow.)
- **Trusted reasons.** `"trusted"` carries a `trusted-reason`: Verus `"admit"` / `"external-body"` / `"assume-specification"`; Lean/Aeneas `"axiom"` / `"external"` (`*External.lean`).
- **Progression.** Grey → White → Yellow → Light Blue → Dark Blue → Light Green → Dark Green. Two branches sit outside the ladder: **Purple** (intentional trust — axioms, external bodies) and **Red** (failure).
- **Green ⊆ Dark Blue.** Green requires a *proven*, validated spec, so on implementations it is always a subset of Dark Blue — never just "the code compiles".

### Counting

> ⚠️ **`scripts/count-colors.sh` is currently out of date** with this scheme: it counts `is-disabled: true` as Grey (correct for Aeneas, wrong for Verus, where it means "no spec"), has no Red bucket, and scopes to `language == "rust"` (so pure-Lean projects count as zero). It — together with KB property [P24](../kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope), which still describes the old Grey/White/Yellow/Dark Blue/Purple partition — will be reconciled with this document in a follow-up PR.

```bash
scripts/count-colors.sh input.json   # auto-detects probe-aeneas / probe-verus extract JSON
```

Once updated, counting should follow the table. Each atom has exactly one color, so colored atoms partition cleanly across Grey, White, Yellow, Dark Blue, Light Green, Dark Green, Red, and Purple (no Light Blue from the JSON alone, since validation state is absent). If the script additionally reports a cumulative "has a validated spec" total, Green is the proven subset of it — a reporting overlay, not a second color on the atom. Specifications are counted separately by `verification-status`.

## Open questions

1. For Aeneas, `is-relevant == !is-disabled`; should the redundant `is-relevant` be dropped? See [probe-aeneas#20](https://github.com/Beneficial-AI-Foundation/probe-aeneas/issues/20).
2. Should an `is-test` field be added so Verus test functions can be identified deterministically? Today `#[test]` → Grey relies on a name/attribute heuristic with no canonical JSON signal.
