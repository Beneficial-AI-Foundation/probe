# Verification Statuses

Defines the per-atom status fields (from the tool schemas) and the color scheme derived from them. Color counts are produced by [`scripts/count-colors.sh`](../scripts/count-colors.sh), which this document and the script must agree on — the script is currently out of date with the scheme below and will be reconciled in a follow-up PR (see [Counting](#counting)).

## Proved vs verified

Two different claims hide under the word "verified":

- A **theorem proves**. A logical statement stands on its own when its body is free of `sorry`/`admit` (relative to an intended [trust base](../kb/engineering/glossary.md#trust-base)). This is a claim about a proposition — no implementation is involved.
- An **implementation verifies** against a chosen spec. This needs *two* things: a spec selected for the implementation, and a proof that the implementation meets it. Verus checks this **directly** (the spec lives on the Rust function); Aeneas checks it **indirectly** (a Lean translation plus a primary-spec theorem about that translation).

So a single `verification-status: "verified"` reads as *"verifies against its spec"* for an implementation that has one, and simply *"proved"* for a bare theorem.

This distinction also separates the two kinds of project this document describes:

- **Verification projects** (Verus, Aeneas, or a Lean project that adopts a spec convention) make the strong claim: *this implementation verifies against its spec*.
- **Generic Lean projects** (mathlib-style — a body of definitions and theorems) make only the weak claim: *this theorem is proved*. In general, Lean projects carry no mechanical rule that says "this construct is the spec of that one", so probe-lean cannot separate an implementation from its specification; the most it can say is whether a construct is proved.

## Atom kinds

| Kind | Description | Examples |
|------|-------------|----------|
| **Implementation** | Executable code that can have a spec attached and *verify* against it | Rust functions, Verus exec-defs, Aeneas-generated Lean `def`s |
| **Specification** | A logical statement that is *proved* (or not) | Verus spec-defs and `proof fn`, Lean `theorem`/`lemma`, non-translation `def`s |

Implementations can have specs attached; specifications cannot — they *are* the specs (always `unspecified`). The Implementation/Specification split is only mechanically available in a verification project; in a generic Lean project every atom is treated uniformly, by whether it is proved.

## Status fields

### `verification-status`

| Value | Meaning |
|-------|---------|
| `transitively-verified` | Verified/proved, and every transitive dependency is verified or trusted ([P23](../kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination)) |
| `verified` | Verified/proved locally, but some transitive dependency is `unverified`/`failed` |
| `unverified` | Has sorries, admits, or warnings |
| `failed` | Compile/verification errors |
| `trusted` | Axiomatically assumed (`axiom`, `#[verifier::external_body]`, `admit()`) |
| `null` | Not subject to verification (tests, constants, external stubs) |

The `transitively-verified` vs `verified` split is computed by `probe enrich` (reverse-BFS contamination); probe-verus and probe-aeneas run it as the last step of `extract`.

What `"verified"` asserts differs by pipeline (this is the *verifies* claim; for a bare theorem the same status just means *proved* — locally sorry-free):

- **Aeneas / Lean (indirect).** A Rust function is `"verified"` only if (1) it has a Lean translation, (2) that translation has a [primary spec theorem](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/docs/SCHEMA.md#rust-specific-fields), and (3) that theorem is proved. A *translated* Rust atom inherits its translation's status — the primary spec theorem's status, or `"unverified"` when the translation has no primary spec (translated but unspecified → Yellow; a translation that is itself `"trusted"`/`"failed"` propagates that status). A Rust function with **no translation** gets **no** `verification-status` at all — it is out of scope (typically `is-disabled: true` → Grey), *not* `"unverified"`. So `"verified"` always implies a proven spec.
- **Verus (direct).** The spec (`requires`/`ensures`) lives on the Rust function and Verus [proves the body satisfies it](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/docs/SCHEMA.md#verification-status-mapping) (`success → "verified"`). A spec-less function is `is-disabled: true` and carries no `verification-status` — never `"verified"`, and (unlike Aeneas) not `"unverified"` either.

### `specified`

An implementation is `specified` if it has a spec attached, else `unspecified`. The carrier is `primary-spec` in every pipeline, but where it lives and what it holds differs:

- **probe-verus** — `primary-spec` is on the Rust function and holds the inline spec text (`requires` + `ensures`). Non-empty `primary-spec` ⇔ `is-disabled: false` ⇔ specified.
- **probe-lean** — `primary-spec` names the chosen spec theorem and `specs` lists every spec-theorem code-name; an atom is `specified` if `specs` is non-empty, equivalently if `primary-spec` is present ([P18](../kb/engineering/properties.md#p18-lean-specified-is-derived-not-stored)). `specs` is a *generic* signal — every theorem whose dependencies include the atom — so it names a spec *of an implementation* only under a spec convention: an `@[primary_spec]`/`@[progress]`/`@[pspec]`/`@[step]` attribute, a `_spec` suffix, or Aeneas-generated code (see [how `primary-spec` is selected](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/docs/SCHEMA.md#probe-leanextract-unified-atoms)). A generic Lean project adopts no such convention, so its atoms have empty `specs` and are all `unspecified` — they are colored by whether they are *proved*, not against a spec.
- **probe-aeneas** — a Rust function carries no spec of its own; `specified` is read off its Lean translation (the atom named by `translation-name`).

## Colors

A color is derived from per-atom JSON fields produced by `probe-<tool> extract`: `language`, `kind`, `is-disabled`, `primary-spec`/`specs`, `verification-status`, and (Aeneas) `translation-name`. The producing tool is identified by the envelope `schema` — or, in a merged `probe/merged-atoms` file, per atom (a `translation-name` marks an Aeneas Rust atom; otherwise by `language`/`kind`). Colors differ slightly by tool, so they are given as two tables below; `"trusted"` → **Purple** and `"failed"` → **Red** take precedence in both.

Atoms that should not appear in VeriLib — external-crate stubs (`code-path: ""`) and atoms flagged `is-hidden` / `is-ignored` / `is-extraction-artifact` — are dropped before coloring (they have no color because they are not shown). A pure-Rust project (no verification framework) carries no verification information at all — no specs, statuses, or translations — so every atom is shown **White** with no counts; there is simply nothing to claim about verification. (Grey is *not* used here — it signals "excluded from verification" *within* a verification project, which would wrongly imply verification intent.)

The two tables mirror the [proved-vs-verified](#proved-vs-verified) split:

- The first colors **implementations** by whether they *verify* — `specified` plus proof status. These are the Rust functions a verification project cares about.
- The second colors **specifications and proofs** by whether they are *proved* — `verification-status` alone. It applies to every Lean atom, so a **generic Lean project uses only this second table**.

### Implementations — does it *verify*? (Verus / Aeneas `exec` atoms)

The executable Rust atoms (`kind: "exec"`), colored by whether they are [`specified`](#specified) and, if so, the proof status. *Specified* is the tool-specific signal from that section — Verus: non-empty `primary-spec` (i.e. `is-disabled: false`); Aeneas: the function's Lean translation is `specified`.

| state | colour |
|-------|--------|
| Verus **unspecified** (i.e. `is-disabled: true`); Aeneas not translated | Grey |
| Aeneas: translated but **unspecified** | Yellow |
| **specified**, not yet proven (`"unverified"`) | Blue |
| `"verified"` | Light Green |
| `"transitively-verified"` | Dark Green |
| `"trusted"` | Purple |
| `"failed"` | Red |

Green requires being `specified` — a function can only *verify* against a spec, so a spec-less Verus function is Grey. (`"verified"`/`"transitively-verified"` already imply a proven spec, so those rows are necessarily `specified`.)

### Specifications and proofs — is it *proved*? (Lean atoms and Verus `proof` / `spec`)

Colored **directly by `verification-status`**. This covers *every* Lean atom — any `kind`, in both pure-Lean and Aeneas projects, **including Aeneas-translated `def`s** — and Verus `proof`/`spec` declarations. (In general, Lean projects do not distinguish implementation from specification, so the most probe-lean can say is whether a Lean construct is proved.)

| `verification-status` | colour |
|-----------------------|--------|
| absent (e.g. a Lean `structure`/`class`) | White |
| `"unverified"` | Blue |
| `"verified"` | Light Green |
| `"transitively-verified"` | Dark Green |
| `"trusted"` | Purple |
| `"failed"` | Red |

A Lean construct with a `sorry` is `"unverified"` → **Blue** (its statement is an unproven spec; for the rare `def`-with-`sorry`, Blue is an approximation, since in Lean projects in general there is no notion of implementation vs specification). A Lean atom with *no* `verification-status` — a type declaration like `structure`/`class` — is **White**: pure-Lean has no "disabled"/excluded notion, so it never goes Grey.

### Notes

- **Progression:** Grey → Yellow → Blue → Light Green → Dark Green, with **Purple** (intentional trust — axioms, `*External.lean`, `#[verifier::external_body]`/`admit()`) and **Red** (failure) as separate branches. **White** sits outside the ladder — pure-Rust browse-only projects and Lean atoms with no `verification-status`. **Blue** means "an unproven spec/obligation exists" (a Rust function with an attached, unproven spec, or a Lean statement with a `sorry`).
- **Color scoping.** **Yellow** is Aeneas-only (a translation exists but no spec yet); **Grey** is Verus/Aeneas-only (a Rust function excluded from verification). A pure-Lean project therefore uses only Dark Green, Light Green, Blue, Red, Purple, and White.
- **Trusted reasons:** `"trusted"` carries a `trusted-reason` — Verus `"admit"` / `"external-body"` / `"assume-specification"`; Lean/Aeneas `"axiom"` / `"external"` (`*External.lean`).
- **VeriLib legend reminder (Aeneas).** A translated Lean `def` is colored by its *own* `verification-status`, so it can show **Green** (the generated code compiles) even when its Rust function is **Yellow/Blue** (unspecified or unproven). VeriLib should add a legend note so a green translation isn't misread as a verified function — e.g. *"In an Aeneas project, a Rust function is verified (green) only if it has a Lean translation and that translation's spec is proven."*
- **Two states the tools cannot (yet) produce, intentionally omitted:** *(a)* a **White** "tracked but unspecified" implementation — the source carries no `in-scope` annotation, so a spec-less function is just Grey (Verus) or Yellow (Aeneas); *(b)* an **unvalidated spec** — every spec in the repo has passed PR review, so all specs count as validated and there is a single **Blue** (specified-but-unproven), not a light/dark split. If an `in-scope` annotation or a validation signal is added later, White and a second Blue can return.

### Counting

> ⚠️ `scripts/count-colors.sh` and KB property [P24](../kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope) still encode the older partition (separate White/Blue buckets, no Red). They will be reconciled with this scheme in a follow-up PR.

## Open questions

1. For Aeneas, `is-relevant == !is-disabled`; should the redundant `is-relevant` be dropped? See [probe-aeneas#20](https://github.com/Beneficial-AI-Foundation/probe-aeneas/issues/20).
