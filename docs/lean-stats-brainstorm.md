# Brainstorm: Meaningful Stats for Lean Projects

**Status:** brainstorming, non-normative. Captures the reasoning behind what stats probe-lean *could* report. Not a spec — the [Atom statuses and colours doc](https://docs.verilib.org/components/processor/atom-statuses-and-colours/) (VeriLib engineering docs) remains the normative doc.

## The problem

For Verus and Aeneas, "verified" is unambiguous: the spec is the function's `requires`/`ensures` (Verus) or the primary spec theorem (Aeneas), and a function is green when that spec is proven. For **Lean-only** projects this breaks down in two ways:

1. **The color counter doesn't apply.** `scripts/count-colors.sh` scopes to `language == "rust"` with non-empty `code-path`. A Lean-only project has no such atoms, so it reports nothing.
2. **Spec attribution is heuristic.** See below.

## What probe-lean can know (from the code)

- **Verification is sound.** A declaration is `verified` iff its proof is sorry-free (`mapVerifyStatus`: `success → verified`, `sorries → unverified`; `VerifyInternal`). There is no `failed` for Lean — a non-elaborating project doesn't build. Trusted = `axiom` kind, `@[externally_verified]`, or non-theorem decls in `*External.lean`. Transitive split (`transitively-verified` vs `verified`) via reverse-BFS enrichment (P23).
- **"Is it a spec?" is structural.** A spec = any atom of kind `theorem`; a `def`/`abbrev` is a specifiable implementation. This *is* derivable from the code.
- **"Whose spec is it?" is NOT in the syntax.** probe-lean over-approximates (`computeSpecs`): theorem `T` is attached as a spec of *every non-theorem it depends on*, so `specs(foo)` = "theorems that mention `foo`" — noisy. The **primary spec** is then picked by precedence: `@[primary_spec]` > framework attrs (`@[progress]`/`@[pspec]`/`@[step]`) > `<name>_spec` naming > sole-spec. Signals 1–2 are instrumentation (reliable); 3–4 are heuristics (fragile).

**Conclusion:** reliable spec→function attribution requires instrumentation. Without it, `specified` means no more than "some theorem mentions this def."

## Reference: what verso-blueprint tracks

[verso-blueprint](https://github.com/leanprover/verso-blueprint) (and the original [leanblueprint](https://github.com/PatrickMassot/leanblueprint)) track **two independent axes** per node, rendered as border (statement) + background (proof):

| Axis | States (worst → best) | Meaning |
|---|---|---|
| **Statement** | `not_ready` → `can_state` → `stated` (leanok) → `mathlib` | is the *statement* formalized in Lean? |
| **Proof** | (not ready) → `can_prove` → `proved` → `fully_proved` | is the *proof* complete (sorry-free)? |

- `proved` = sorry-free locally; `fully_proved` = sorry-free **and** all transitive ancestors are too.
- `can_state` / `can_prove` = not done, but all prerequisites are — "ready to work on."
- Progress = fraction of nodes whose associated Lean decl exists and is sorry-free.

**Key design point:** these stats are reliable because the **human authors the binding** in the blueprint (`\lean{...}`, `\leanok`, `\notready`) — it is *told*, not inferred. That is the same instrumentation we need; they just put it in the blueprint instead of `@[primary_spec]`. It also lets them report *planned-but-unformalized* work (the statement axis), which code alone cannot.

## Mapping to our model

verso-blueprint's proof axis maps one-to-one onto what probe-lean already computes:

| verso-blueprint | probe-lean |
|---|---|
| `fully_proved` | `transitively-verified` |
| `proved` (local) | `verified` |
| has `sorry` | `unverified` |
| axiom / trusted | `trusted` |

## Proposed stats

### Tier 1 — sound, no spec inference (the headline)

Proof-completion breakdown, split by kind:

| | `transitively-verified` | `verified` (local) | `unverified` (sorry) | `trusted` |
|---|---|---|---|---|
| **theorems** (specs) | … | … | … | axiom / `@[externally_verified]` / `*External.lean` |
| **defs** (implementations) | … | … | … | … |

Headline number: *fraction of theorems that are `transitively-verified`* (= blueprint's `fully_proved`). This needs **zero** spec attribution. Note `defs` are usually trivially `verified` (rarely contain `sorry`), so the real verification signal lives in the theorems.

### Tier 2 — spec coverage, instrumentation-dependent (label it)

- Of defs, how many have a primary spec (Dark Blue) → of those, how many proven (Green).
- Reliable **only** with `@[primary_spec]` / framework attributes (or a blueprint). On un-instrumented code, report with the "some theorem mentions this def" caveat, or suppress.

### Out of scope for code-only analysis

The **statement axis** (`not_ready` / `can_state`) describes intended-but-unformalized work. That lives in a blueprint, not the code — probe-lean cannot compute it.

## Open questions

1. Adopt verso-blueprint's two-axis vocabulary (statement vs proof) verbatim for legibility, or keep our single `verification-status` field and derive?
2. Extend `count-colors.sh` to handle `probe-lean/extract` (Tier-1 stats), or build a separate Lean stat reporter?
3. The spec-color table in `verification-statuses.md` has no "unverified theorem" (sorry) state — Lean needs one. Add it?
4. Should Tier-2 coverage be gated: only emitted when the project is instrumented (any `@[primary_spec]` present), else omitted with a note?
