---
title: Merge soundness fix plan
date: 2026-09-22
status: agreed — not yet started; revised 2026-09-22 after Codex cross-model review; revised again 2026-09-22 after a second Codex review (projection rejection, Aeneas status transfer, trusted-semantics audit, executable schema targets); revised a third time 2026-09-22 after a third Codex review (translation-origin promotion blockers, legacy-projection rejection, Aeneas mapping-record preservation); revised a fourth time 2026-09-22 after a fourth Codex review (probe-lean kernel-taint evidence, unified blocker-seed BFS, translated-trusted precedence, summary consumer contract); revised a fifth time 2026-09-22 after a fifth Codex review (legacy producer-extract version gate — fresh extracts only, no provenance extension; raw-staging authority checks, imported-verified membership condition, status-origin/summary executable-schema targets, probe-project and closure doc targets); revised a sixth time 2026-09-22 (simplification pass: dropped §3e micro-optimizations, the summary JSON schema and `sources`-block parsing; deferred staleness warnings; split PR 3 into three PRs); revised a seventh time 2026-09-28 after a seventh Codex review (per-producer version gate covering composed artifacts — old hub merges and Aeneas extracts, not just raw probe-lean; probe-leanblueprint added as an authority-validating composer; structural composed-provenance detection preserving Aeneas source inventories; PR reordering — authority checks land before recomputation; collision record-union moved to PR 3b); revised an eighth time 2026-09-28 after an eighth Codex review (version gate extended to the read-only consumers probe summary and probe project; P16 Lean status table and schema.md status-row rewrite pulled into PR 1; projection-reachability claim corrected — projection BFS stays dependency-only; blueprint composed-envelope emit promoted into the rollout and its dead migration path removed; §10 restated as settled decisions with rationale); revised a ninth time 2026-09-28 after a ninth Codex review (probe-verus embedded-enrichment dependency bump — no gate entry; projection enriches-then-trims authoritative inputs so stale labels are not frozen into unrepairable views; §3f gate thresholds fixed as reserved release numbers upfront — composers compile the validator in, so placeholder-lowering follow-ups would strand them); revised a tenth time 2026-09-28 after a tenth Codex review (probe-verus `merge-atoms` composer bypass — the `probe` gate entry becomes a version interval with a reserved 1.0.0 ceiling plus a legacy-command rejection, and the command is retired or delegated; blueprint synthetic-status audit and status-origin propagation; summary excludes blueprint-language atoms; docs/merge-algorithm.md full phase-outline rewrite); revised an eleventh time 2026-09-28 after an eleventh Codex review (probe-vcvio annotation re-emitter — the composer rule widened from "runs enrichment over foreign atoms" to also cover re-emitting foreign verification evidence under one's own tool identity; the §3f gate table grows to five entries; probe-vcvio rollout and KB page); revised a twelfth time 2026-09-28 after a twelfth Codex review (probe-verus `--skip-atomize` cached-atom input boundary — the composer rule widened to cover building authoritative verification output over a previously-emitted dependency graph, with boundary validation but no Verus gate entry; normalize-then-enrich as the shared carrier-preparation step at every recomputation boundary, so mapping-endpoint normalization and atom-map normalization cannot diverge in `probe project` and `probe enrich`); revised a thirteenth time 2026-09-28 after a thirteenth Codex review (carrier-preparation ordering corrected — normalization is per-input *before* conflict resolution, as the code already does, not post-union; blueprint synthetic statuses scoped out of the code-atom assurance contract — marker propagation alone cannot cover a marker-free contaminated contributor; probe-vcvio added to the executable schema's single-tool branch)
scope: fixes for merge-soundness-review.md, aligning code, KB and docs/categorical-framework.md
---

# Merge soundness fix plan

> **Archived 2026-09-29.** Full version preserved for provenance — the
> complete 13-round Codex review history is in the front matter above.
> The **live, condensed plan is
> [`merge-soundness-fix-plan.md`](../../merge-soundness-fix-plan.md)**
> at the repo root; execute from there, not from this copy (its status
> line and doc paths are frozen at 2026-09-28 — e.g. PR 1 has since
> landed as #63, and `docs/categorical-framework.md` /
> `docs/mappings-spec.md` / `docs/merge-algorithm.md` were moved into
> the KB or archived by #65).

Self-contained plan for resolving the findings in
[merge-soundness-review.md](../../merge-soundness-review.md). Written to be
executable from a fresh session: it records the design decisions, the
rationale, the exact current behavior with code locations, and the target
behavior.

## 1. Background: what the review found

`merge-soundness-review.md` (repo root, same date) reviewed `probe merge`
and enrichment against the KB. Findings, numbered as there:

1. **Soundness bug**: `merge --mappings` adds cross-language edges after
   enrichment ran, invalidating `transitively-verified` labels (P23), and
   re-running `probe enrich` cannot repair them because enrichment only
   upgrades, never downgrades.
2. **P15 violation**: mapped edges are inserted into `dependencies` only,
   so `dependencies` ≠ union of the categorized subsets.
3. **No staleness checks**: merge never compares `source.commit` /
   `timestamp` across inputs; freshness is proxied entirely by argument
   order (first-wins atoms, last-wins specs/proofs).
4. **P10 vs leanblueprint**: merge picks whole atoms, never unions
   extension fields; conflicting real atoms lose one side's extensions
   wholesale. (Not addressed by this plan — needs its own decision. A
   narrow carve-out for correspondence records is made in §5.)
5. **Algebraic laws overclaimed**: P4 (associativity) fails with
   `--mappings` across groupings; P5 (identity) holds only up to
   normalization; envelope merge is not idempotent (provenance
   duplicates).
6. **Inconsistent collision policies after normalization**: within one
   input, `f()` vs `f().` collide; `normalize_atoms` resolves silently
   (first-wins with stub replacement — see the correction in §6),
   `normalize_generic` keeps last silently, neither counted.

Minor observations: conflict detection is key-only (byte-identical atoms
count as conflicts); P8 normalization misses code-name-bearing extension
arrays (`requires-dependencies` etc.).

Design question (end of review): mapping edges attach to the *callers* of
a mapped atom, never the atom itself; a leaf function gets no
cross-language linkage.

## 2. Core design decision: correspondence records, not dependencies

Injecting mapped edges into `dependencies` conflates two relations with
different verification semantics:

- *Dependency* (calls, or proof-uses): Rust `f → g`; in Lean, the
  security theorem about `f`'s model depends on `ga`. Both already exist
  inside their own extracts.
- *Correspondence* (abstraction/translation): `g ↔ ga` — "same logical
  definition" (ADR-003) or, for manual mappings, "ga is a hand-written
  abstraction of g". This is what the mappings file records.

Motivating scenario: Rust `f` calls `g`; secure-messaging proves a
property about `f`'s model and uses `ga`, an abstraction of `g`, which
contains a sorry. Current merge fabricates `f → ga` (false: `f` doesn't
call `ga`) and, bidirectionally, gives Lean atoms depending on `ga` a
dependency on Rust `g` (almost always `unverified`), so proven Lean
theorems would be contaminated by the mere existence of unverified Rust
code — backwards: the Lean proof's validity doesn't depend on the Rust
code's verification status. The sorry in `ga` legitimately weakens only
the Lean theorem, and that contamination path is intra-Lean, handled by
enrichment. The correspondence is the *trust boundary*, not a dependency.

Consequences:

- Issue 2 disappears: `dependencies` untouched by mappings.
- Issue 1 shrinks but does not disappear: mappings no longer touch
  `dependencies`, so they can no longer invalidate P23 labels — but
  *plain* merge still can (§3), so merge must re-enrich.
- The design question dissolves: correspondence attaches to the mapped
  atom itself (`g maps-to ga`), not to its callers; leaf functions get
  linkage.
- ADR-003's promise "mappings are confidence-tagged, enabling consumers
  to filter by confidence" is restored: injected `dependencies` strings
  couldn't carry confidence; records do.
- Cross-language resolution becomes a *derived view* computed by
  consumers over the records at read time. (The old implementation never
  resolved anything either: a stub `g` stayed a stub even when `ga` was
  real; it only added parallel edges.)

Cross-language status flow ("Rust `f` is assured via its Lean proof") as
a *derived status* (e.g. `verified-by-translation` computed over
`maps-to` records with a confidence threshold) remains **deferred** to a
future ADR. `maps-to` records never participate in contamination BFS.

**But status transfer already exists downstream and cannot be deferred
wholesale** (second Codex review). probe-aeneas copies a Lean
primary-spec theorem's `verification-status` onto the corresponding Rust
atom (`resolve_verification_status`, ../probe-aeneas/src/extract.rs:858,
written at :935) and then runs generic enrichment (:829). Laundering
counterexample that survives everything else in this plan: Lean theorem
`T [verified] → lemma [unverified]`; Rust leaf `g` gets `verified`
copied from `T` and has no Rust dependencies, so enrichment promotes `g`
to `transitively-verified` — contaminated imported proof evidence turned
into a clean transitive label, with fresh, matching inputs. Interim
decision, in scope (ADR-006 + §8): Aeneas marks every copied status with
a `status-origin: "translation"` extension, and enrichment **never
promotes** such atoms to `transitively-verified` (contamination /
downgrade still applies; the copied base status is imported evidence,
not a locally established result).

**The guard must be compositional** (third Codex review): the per-atom
rule alone lets callers launder. Counterexample: Rust `f [locally
verified] → g [verified, status-origin: translation]`, where `g`'s
status came from Lean `T [verified] → lemma [unverified]`. `g` is not a
contamination source and the lemma is unreachable in the Rust graph, so
`f` would come out `transitively-verified` — the same imported evidence,
one hop removed, and it satisfies the §3b definition as first drafted.
So translation-origin atoms are additionally **promotion blockers**: any
atom that reaches one along a non-trusted dependency path keeps
`verified` and is never labelled `transitively-verified` (§3a/§3b —
blocker seeds in the *same* BFS as contamination: on a locally verified
atom, blocking and contamination are the same operation, the label is
set to `verified`; neither downgrades a seed's own base status). They are deliberately *not*
trusted boundaries — treating them as trusted would promote callers and
reopen the laundering path. And because Aeneas copies the Lean status
verbatim (../probe-aeneas/src/extract.rs:868-878), an already-enriched
`transitively-verified` can arrive pre-populated: Aeneas normalizes the
copied value to `verified` at copy time, and enrichment defensively
rewrites translation-origin `transitively-verified` → `verified`
**unconditionally**, not only when contaminated. The probe-aeneas
rollout adds end-to-end regression tests for exactly these graphs (leaf
promotion; caller laundering; transfer from an already-enriched
theorem).

**Producer-known taint: the same laundering exists with no translation
at all** (fourth Codex review). probe-lean's authoritative statuses come
from a kernel-level taint walk, not from the emitted graph
(../probe-lean/ProbeLean/Transitive.lean:20 — `verified` = locally
proved but an unexcused sorry is *kernel-reachable*,
`transitively-verified` = clean); the graph BFS is a diagnostic that is
"reported, never reconciled" (../probe-lean/ProbeLean/Extract.lean:385).
That kernel-reachable taint is not reconstructible from the emitted
dependencies, and maintained fixtures assert exactly the divergence:
`probe:ownSorry` is `verified` with an *empty* dependency list (its
sorry sits in a compiler-generated proof auxiliary the graph has no node
for), and `probe:viaNoRange` is `verified` with its taint path through a
non-emitted node
(../probe-lean/tests/fixtures/aux-fold/TaintCheck.lean:313, :327).
Graph recomputation (§3) would relabel both `transitively-verified` —
plain merge with an empty map suffices. This hole predates the plan:
today's upgrade-only `probe enrich` already promotes `ownSorry`; §3
merely makes the laundering systematic unless fixed. The §3b
"selected graph" caveat does not cover it — preserving a producer's
negative finding is not proof re-validation.

Decision (ADR-006): generalize the marker into one mechanism.
`status-origin` is an enumerated extension with two values —
`"translation"` (Aeneas-copied evidence, above) and `"kernel-taint"`
(probe-lean: the kernel walk found reachable taint the emitted graph
cannot express). Enrichment treats every `status-origin`-bearing atom
uniformly as a **blocker seed**: never promoted to
`transitively-verified`, unconditionally demoted from an imported or
stale `transitively-verified`, and blocking promotion of every atom
that reaches it along a non-trusted path (§3a/§3b). The two values
differ only in *whose evidence* the status is — imported vs local —
which matters to consumers (§6 summary), not to enrichment. probe-lean
stamps the marker whenever its taint pass runs, including under
`--skip-enrich` (which caps labels but still knows the taint), so an
*unmarked* `verified` atom is safely promotable; probe-lean joins the
§8 rollout.

## 3. Derived statuses: merge re-enriches, enrichment recomputes

**Plain merge alone can invalidate P23 labels.** Counterexample: input A
has `f [transitively-verified] → g [stub, no status]` (enrichment at
extract time treated the status-less stub as transparent). Input B has
`g [real, unverified]`. Merge resolves the stub (P6); `f` keeps a label
that is now false, and nothing in the pipeline repairs it. Stub
resolution is merge's central operation, so labels on merged output
cannot be trusted unless merge recomputes them.

The fixes (3a–3d and 3f), all in scope; 3e records the measured cost:

**3a. Enrichment recomputes rather than upgrades.** Current behavior
(src/commands/propagate.rs): `is_verified` (lines 14-16) treats
`transitively-verified` as verified; reverse-BFS contamination seeds at
explicit `unverified`/`failed`; step 5 (lines 152-166) only *writes*
`transitively-verified` onto non-contaminated verified atoms — a
contaminated atom already carrying `transitively-verified` keeps its
label. Fix: **one reverse BFS from one seed set** — explicit
`failed`/`unverified` atoms plus every `status-origin`-bearing atom
(§2) — then, for every atom whose status is `verified` or
`transitively-verified`, set the label fresh: reaches a seed along a
non-trusted path, or is itself a seed → `verified`; otherwise →
`transitively-verified`. The seed classes need no separate traversals
(fourth Codex review): on a locally verified atom, contamination and
promotion-blocking are the same operation, and the §2 rules
(never-promote, unconditional demotion of an imported
`transitively-verified`) fall out of "is itself a seed" — stats may
still count the classes separately. Seeds keep their own base status
(`failed`/`unverified`/`trusted` untouched; blocking never downgrades
below `verified`). Recomputation
makes the result a function of the final graph and base statuses only —
which is also what makes it compatible with the merge algebra (§4).

**3b. Contamination traverses per P23's definition.** P23 says atoms
with missing `verification-status` are "transparent", and its headline
definition quantifies over *every transitively reachable* dependency.
The implementation makes them barriers: propagation only continues
through callers in the verified set (src/commands/propagate.rs:127-150),
so in `v [verified] → helper [no status] → bad [unverified]`,
contamination stops at `helper` and `v` is wrongly labelled
`transitively-verified`. Fix the code and restate P23 with one
executable definition that *replaces* (not merely qualifies) the old
wording:

> A locally verified atom is `transitively-verified` iff no **seed** —
> an explicit `failed`/`unverified` atom, or any atom carrying
> `status-origin` — is reachable from it along a dependency path that
> does not pass through a **trusted boundary** (a zero-length path
> counts: a seed is never promoted). A `trusted` atom is a boundary
> only when it carries no `status-origin`. Seeds keep their own base
> status, so blocking never downgrades below `verified`.

The boundary qualification is required, not defensive (fourth Codex
review): Aeneas copies Lean `trusted` verbatim
(../probe-aeneas/src/extract.rs:868), so `trusted` + `status-origin:
"translation"` is a normal producer case. §2 already rules that
translation-origin atoms are not trusted boundaries; without the
executable precedence — `trusted_boundary(atom) = status == trusted AND
status-origin absent` — an implementation applying the generic boundary
check first would let copied trust shield its callers and reopen the
laundering path. A translated-trusted atom seeds like every other
`status-origin` atom.

This makes missing-status atoms transparent by construction and makes
**`trusted` a trust boundary** — trusted atoms are intentional axioms,
so what is behind them is irrelevant to their consumers. Underneath the
boundary rule is a policy choice ADR-006 must make explicitly: trusting
an atom trusts its **entire dependency closure** (whole-atom trust),
not just its body. We choose whole-atom trust: enrichment traverses the
unified `dependencies` set, and the schema's body/spec/type split is
not available on plain atoms. Revisit only if a concrete case shows a
trusted atom's spec/type dependency must contaminate.

Whole-atom trust is a **semantic strengthening**, not a restoration of
old wording (second Codex review): the emitted `trusted-reason` values
(`external-body`, `admit`, `assume-specification` —
kb/engineering/schema.md) do not self-evidently authorize closure-wide
trust, and under the rule a `failed` predicate hanging off a trusted
atom's `requires-dependencies` is hidden from callers. Before PR 1
lands: audit what each producer (probe-verus, probe-lean,
probe-leanblueprint) actually asserts with `trusted`/`trusted-reason`,
record the findings in ADR-006, and document whole-atom trust there as
a deliberate semantic change to P23. If the audit surfaces a real
counterexample, the body-trust/closure-trust split becomes the fallback
design; do not build it speculatively.

The audit covers status *synthesis*, not only emission (tenth Codex
review): probe-leanblueprint creates statuses after enrichment —
`derive_synthetic_verification`
(../probe-leanblueprint/src/main.rs:1065) aggregates a bound node's
decl statuses ranking `trusted` above `verified`
(../probe-leanblueprint/src/enrich.rs:689) and writes status plus
trusted-reason without propagating `status-origin` (enrich.rs:841). A
node binding a kernel-tainted `verified` theorem and a trusted axiom
therefore becomes a synthetic **unmarked `trusted`** atom — blueprint
means "the aggregate contains attested evidence", while revised P23
reads the same label as an intentional whole-closure trust boundary.
The §8 blueprint rollout has synthesized statuses propagate their
contributors' markers — but **marker propagation alone cannot close the
mismatch** (thirteenth Codex review): a marker-free counterexample
exists. `T [verified, no origin] → bad [unverified]` (contaminated
through ordinary graph edges after merge recomputation — the negative
finding is graph-expressible, so no `kernel-taint` marker exists and
§3a correctly leaves `T` at `verified`) plus `axiom [trusted, no
origin]`, bound by node `N`: blueprint's ranking puts `trusted` above
`verified` and emits `N [trusted]` with no marker to propagate, a label
revised P23 would read as whole-closure trust covering the reachable
`bad`. Resolution (ADR-006/P23): the **assurance contract — including
the whole-closure trusted-boundary reading — is stated over code
atoms**; synthetic `language: "blueprint"` statuses are presentation
aggregates, never trust-boundary assertions. This is a scoping of the
claim, not a laundering fix: blueprint emits no code→node dependencies,
so the aggregate can shield only other presentation atoms, and summary
already excludes blueprint-language atoms from every partition (§6).
The §8 rollout pins the marker-free case by test alongside the
marked-contributor case.

Scope caveat, stated in P23 and ADR-006: recomputed labels assert
consistency with the *selected merged graph and the retained base
statuses*. They do not re-validate proofs — a `verified` status
extracted against one implementation of a dependency is carried over
unchanged when merge selects another implementation; even the staleness
diagnostics recorded (and deferred) in §6 could not close this.

**3c. The shared merge implementation runs enrichment recomputation.**
The guarantee lives in `merge_atom_maps` itself (atoms category only;
specs/proofs unaffected), executed **once after all input maps are
combined** — not per incoming file. `cmd_merge`, `merge_atom_files`
(src/commands/merge.rs:181 — a public entry point that never passes
through `cmd_merge`) and every library caller inherit it, so the tested
operation is the same μ that §4 defines. A raw no-enrichment staging
primitive is **required, not optional** (second Codex review):
probe-aeneas mutates verification statuses *after* merging (metadata
phase, ../probe-aeneas/src/extract.rs:817-830), so auto-enrichment
inside `merge_atom_files` would run a wasted first pass and Aeneas
would pay enrichment twice; it must stage on the raw path and enrich
once at the end, which also keeps its existing `--skip-enrich` contract
intact (the flag guards the only enrichment pass in its pipeline). The
raw primitive is named and documented as returning potentially stale
derived statuses. **Raw means skip recomputation, not skip authority
validation** (fifth Codex review): the raw staging entry point loads
envelopes and applies the same rejection predicates as the public entry
points — projections (§3d) and pre-marker legacy extracts (§3f) — only
the enrichment pass is deferred. This matters concretely: probe-aeneas
accepts precomputed input files (../probe-aeneas/src/extract.rs:641)
and currently routes them through `merge_atom_files` (:791); migrating
it to an unchecked raw loader would discard the projection marker
before its final enrichment — the very boundary failure §3d closes
elsewhere. Test: the raw path rejects both projection formats. Merged
output of the public entry points is therefore always
enrichment-consistent.

**Authority validation is one shared function, and downstream composers
are callers too (seventh Codex review).** The rejection predicates (§3d
projections in both formats, §3f version gate) are implemented once as
a shared validator invoked at every envelope boundary: `cmd_merge`,
`merge_atom_files`, `cmd_enrich`, the raw staging primitive — and every
downstream producer that runs enrichment over foreign atoms.
probe-leanblueprint is the case the plan missed: it loads an atoms
envelope rejecting only *its own* previous output
(../probe-leanblueprint/src/main.rs:344), calls the hub's bare-map
`enrich_verification_status` directly (main.rs:1058) — the API §3d
documents as unable to check authority — and re-emits the result as
`probe-leanblueprint/extract` under its own tool identity
(../probe-leanblueprint/src/emit.rs:33). That makes it a laundering
channel around both boundaries: a pre-marker Lean extract or a
projection goes in, an apparently authoritative extract with a fresh
tool version comes out, and the hub gate sees neither `probe-lean` nor
a projection marker. Fix (§8 rollout): blueprint bumps its hub
dependency, calls the shared validator before its enrichment pass, and
preserves `status-origin` markers through its pipeline; its own
pre-release outputs are rejected by the §3f table. ADR-006 states the
rule generally: **any producer that runs enrichment over foreign atoms
must validate authority first** — the hub, probe-aeneas staging,
probe-leanblueprint, probe-vcvio (below) and (if delegated rather than
retired) probe-verus's `merge-atoms` (§3f, §8) are the current
instances.

**Re-emission alone bypasses the gate — the rule covers annotators too
(eleventh Codex review).** probe-vcvio is a composer the inventory
missed, and it never runs enrichment: it accepts an existing probe-lean
extract (`--lean`, ../probe-vcvio/src/main.rs:116), validates only the
schema string and 3.x schema version — never the input's `tool`
metadata (../probe-vcvio/src/model.rs:311) — and re-emits every atom,
verification statuses included, as `probe-vcvio/extract` under its own
tool identity (../probe-vcvio/src/emit.rs:167). The hub reads any
`*/extract` schema as atoms (src/types.rs:163) and `probe-vcvio` has no
gate entry, so a pre-marker probe-lean extract laundered through it
passes the §3f gate at any version, and §3a then promotes `ownSorry`
exactly as if the gate did not exist — no enrichment, spoofed identity
or malformed envelope required. The composer rule is therefore widened
(ADR-006): **any producer that runs enrichment over foreign atoms *or
re-emits foreign verification evidence under its own tool identity*
must validate authority first** — re-stamping the envelope conceals the
evidence's origin from the gate as effectively as recomputing over it.
probe-vcvio joins the gate table (§3f) and the rollout (§8). Because it
reuses the hub's `Atom` type (../probe-vcvio/src/model.rs:7),
`status-origin` markers already survive its round-trip; the missing
pieces are input validation and its own gate entry, not marker
plumbing.

**Consuming a foreign graph is the third composer shape (twelfth Codex
review).** probe-verus's ordinary extract pipeline has an unchecked
atom-input boundary: with `--skip-atomize` it reuses a cached atom file
(../probe-verus/src/commands/extract.rs:248), loaded by
`load_enveloped_data` → `unwrap_envelope` with no schema, projection or
version check (extract.rs:831), and combines that graph with fresh
verification results into a new `probe-verus/extract` envelope
(extract.rs:975). Feed it a depth-0 projection of `f [verified] → g
[failed]` and the output is `f [verified]` with no dependencies under a
fresh authoritative identity — the projection marker is erased, and
unlike a stale label this truncation is unrepairable: no later BFS can
restore deleted edges (reproduced against the compiled library). The
composer rule is therefore widened a second time (ADR-006): a producer
that runs enrichment over foreign atoms, re-emits foreign verification
evidence under its own identity, *or constructs authoritative
verification output over a previously-emitted dependency graph* must
validate that input's authority first. Fix (§8 rollout): the Verus
cached-atom boundary runs the shared validator (projection rejection +
version gate) before unwrapping. Deliberately **no `probe-verus` gate
entry** — see §10: unlike pre-marker Lean or Aeneas extracts, a
pre-fix Verus extract is unsound only if a hub projection was manually
routed into a cache flag, an artifact class with no known member.

**3d. Projected artifacts are views — decide before merge auto-enriches.**
`probe project` trims `dependencies` to the included set but keeps
verification statuses, and writes the ordinary `probe/merged-atoms`
schema with no marker surviving `load_envelope`
(src/commands/project.rs:117, 247; src/types.rs:183). Enriching a
projection therefore *upgrades* labels: `f [verified] → bad
[unverified]` projected at depth 0 becomes `f` with no deps, and
recomputation relabels it `transitively-verified`. With §3c, merging
any projected artifact would silently launder truncated views into
stronger labels. Decision (ADR-006): `transitively-verified` describes
the **original dependency graph**; a projection is a *view*, and
deleting edges must not improve assurance.

Implementation: **rejection, not warn-and-skip** (second Codex review —
skipping is not a defined operation: in `merge(A, B, P)` with an
unrelated projection P, skipping recomputation for the invocation
preserves exactly the stale label §3c repairs, and exempting only
projected atoms needs conflict/path rules that are no longer cheap).
Projection writes a distinct schema string, `probe/projected-atoms`
(the existing `projection` envelope field stays as metadata);
`probe merge` and `probe enrich` **error** on any input carrying it.
**Detection covers the legacy form too** (third Codex review): every
projection produced so far carries `schema: "probe/merged-atoms"` plus
a `projection` field (src/commands/project.rs:247), and `load_envelope`
discards that field — so the rejection predicate at the envelope
boundary is *new schema string OR presence of a `projection` field*,
tested against an actual legacy-format projection. Regeneration is not
a substitute for this check: legacy views are identifiable before the
metadata is discarded.
A marker alone cannot be enforced deeper in the stack: `load_atom_file`
(src/types.rs:273) discards envelope metadata, `merge_atom_maps` takes
bare maps, `project_atoms` returns a bare map
(src/commands/project.rs:30), and `cmd_enrich` parses raw JSON without
`load_envelope` (src/commands/propagate.rs:172) — so authority checking
lives at the envelope boundary (`cmd_merge`, `merge_atom_files`,
`cmd_enrich`, the last rerouted through `load_envelope` or an
equivalent schema check), and the bare-map library API is documented as
unable to check authority. Plumbing for the new schema string:
`schemas/atom-envelope.schema.json` accepts it, `detect_category`
(src/types.rs:163) recognizes it as atoms for read-only consumers, and
composed-provenance detection becomes structural (§6, seventh review:
an envelope with `inputs` is composed, one with `source` is
single-tool), which flattens projected provenance without widening the
old `probe/merged-` prefix check. P23
documents that labels inside projections describe the **original**
graph, and projection makes that true rather than assuming it (ninth
Codex review): on an authoritative (non-projected) input, `probe
project` recomputes enrichment on the full input graph *before*
trimming — otherwise a stale label from a producer's embedded old
enrichment (probe-verus, §3f) is frozen into a depth-limited view that
this very rejection rule then makes unrepairable (reproduced against
the current binary: the §3f Verus-shaped chain projected at depth 0
keeps `f [transitively-verified]` with no dependencies). Labels are
never recomputed from the trimmed view; an already-projected input
stays readable (§3f) and keeps its labels untouched. The §3f version
gate on project's input is what makes the recomputation sound — no
pre-marker evidence can reach it. The distinct schema string is deliberate: old
consumers going through `load_envelope` fail loudly on a projected file
instead of silently treating a view as authoritative — a scoped claim,
not universal: today's `cmd_enrich` checks only the schema *version*
(src/commands/propagate.rs:183-190) and would accept the new string,
which is exactly why it is rerouted through a schema-identifier check.
No boundary-evidence machinery, no separate view-composition operation.

**Carrier preparation is normalize-then-enrich, shared by every
recomputation boundary (twelfth Codex review).** Normalization today
lives only inside merge (`normalize_atoms`, src/commands/merge.rs:31);
`cmd_enrich` and `probe project` operate on the raw loaded map, and
`project_atoms` seed-matches mapping endpoints by exact key
(src/commands/project.rs:39). Two reproduced consequences: (a) §5
normalizes mapping endpoints at load, so a legacy input keyed `f().`
with a mapping naming `f().` would select **zero** seeds after the
loader change — an empty-projection regression introduced by the plan
itself; (b) enrichment over an unnormalized map resolves names
differently — `f [verified] → g.` with `g [failed]` promotes `f`
(dangling dep, treated as trusted) where the normalized graph
contaminates it, so project's enrich-then-trim would freeze the wrong,
stronger label into a view. Fix: the §4 carrier already requires
*normalized* enrichment-consistent maps, so normalization and
enrichment are **shared primitives** applied at every recomputation
boundary — but not one fused post-union function. **In μ, normalization
runs per input, before conflict resolution** (thirteenth Codex review):
that is today's actual behavior (`merge_atom_maps` normalizes the first
input at src/commands/merge.rs:103 and each incoming input at :107
before the P6 loop), and the twelfth revision's "applies it after
combining inputs (as today)" misdescribed it. The ordering selects
evidence, it is not cosmetic: with input A `f [verified] → g.`,
`g. [failed, real]` first and input B `g [verified, real, no deps]`
second, per-input normalization collapses both definitions to `g` and
A's `failed` wins under P6 (`f` stays `verified`); normalizing the raw
union instead lets `g` and `g.` coexist until normalization visits `g`
first and B's `verified` wins — enrichment then promotes `f` to
`transitively-verified` over silently mis-selected evidence, and no
later BFS or gate can repair a P6 selection. So:
`μ(A, B) = enrich(resolve_conflicts(normalize(A), normalize(B)))`, with
one enrichment pass after all inputs are combined (§3c). Unary
boundaries (`cmd_enrich`, `probe project`) apply the composed
`prepare = enrich ∘ normalize` to their single authoritative input
before recomputation, seed matching or trimming (already-projected
inputs keep §3d's rule: readable, labels untouched, no recomputation) —
with one input there is no conflict resolution and the two orderings
coincide. Mapping endpoints and atom keys are then normalized by the
same rule on both sides of every lookup. Regressions: a dotted-key atom
is found by a normalized projection seed (PR 3c); the dotted-alias
contamination graph above enriches to `verified`, not
`transitively-verified` (PR 2); and the two-input evidence-selection
graph above, asserting both that A's `failed` `g` is the selected atom
and that `f` stays `verified` (PR 3b, against `merge_atom_maps` —
single-input alias tests cannot catch it).

**3e. Cost of recomputation (measured — not a blocker).** Enrichment is
one multi-source reverse BFS over the whole map, ~O((V+E)·log(V+E))
with the current BTree collections; "upgrade-only" today is a *write*
policy, not a cheaper algorithm, so §3a adds no asymptotic cost.
Release-mode measurements on the current functions (second Codex
review, 2026-09-22, three samples per case): real 1,547-atom /
18,617-edge artifact — 10.4–11.9 ms enrichment vs 2.5–3.0 ms
normalize/merge and ~73–77 ms load + 14 ms serialize; synthetic 100k
atoms / ~400k edges — 289–350 ms enrichment vs 68–81 ms merge;
synthetic 100k atoms / ~1.2M edges with 118-byte names — 706–1,181 ms
enrichment vs 109–142 ms merge. So: cheap on real artifacts, but on
large dense graphs enrichment *dominates* the merge pass — "not free"
means up to ~1 s at 100k atoms, acceptable for a batch CLI whose
parse/serialize costs are also in the hundreds of ms at that size.
Proportionate mitigations, in scope: run it once per completed merge in
memory (§3c), and stage multi-step pipelines on the raw primitive so
enrichment runs exactly once (§3c — probe-aeneas currently would pay it
twice). Everything further is **dropped** (sixth revision): the
fast-path pre-scans, per-orphan warning capping, mapping-record
endpoint indexing and the `Vec` reverse index discussed in earlier
revisions bought milliseconds on real artifacts (~11 ms total) at the
price of delicate conditions — the fourth review already caught a
laundering bug in one (skipping on "no contamination sources" alone
would let `f [verified] → g [verified, status-origin: translation]`
promote `f`). The ~1 s dense-synthetic worst case is acceptable for a
batch CLI whose parse/serialize costs are comparable; revisit only
against a measured complaint. Out of scope (§10): those mitigations,
incremental maintenance, dense node IDs.

**3f. Pre-contract artifacts are rejected — a per-producer version gate
(fifth Codex review; generalized by the seventh).** The §9 rollout
constraint ("don't merge/re-enrich probe-lean artifacts until the marker
ships") is operational only: shipping the probe-lean release changes
nothing about existing extracts, and an old `ownSorry [verified]` with
empty dependencies and no marker is indistinguishable from a safely
promotable atom — regenerating a *merge* from that old extract still
launders. `load_envelope` accepts every 3.x version (src/types.rs:209)
and nothing else establishes that an accepted input implements the
evidence contract.

The fifth-review formulation gated only single-tool `probe-lean`
envelopes; the seventh review showed that misses the *composed* form of
the same evidence: an old hub merge already containing `ownSorry
[verified]` with no dependencies carries `tool.name: "probe"`, and an
old probe-aeneas extract with unmarked copied statuses carries
`tool.name: "probe-aeneas"` — both sail past a probe-lean-only gate,
and their re-enriched output bears a fresh version, laundering
permanently. "Legacy merged artifacts are unsupported wholesale" (§8)
was policy without mechanism: the claimed induction ("every merged
artifact was produced from gated inputs") had no enforced base case.

Fix, at the same envelope boundary as §3d: merge and enrich **error**
on any atoms envelope whose `tool.name` is in a five-entry table with
`tool.version` below that producer's contract threshold —

- `probe-lean` — the kernel-taint-marker release (§2);
- `probe-aeneas` — the translation-marker + raw-staging release (§8);
- `probe-leanblueprint` — the authority-validation release (§3c);
- `probe-vcvio` — the input-validation release (§3c, eleventh review):
  it re-emits probe-lean evidence verbatim under its own identity, so
  pre-contract outputs can carry pre-marker statuses past a gate keyed
  on the original producer's name;
- `probe` — the release of this plan (pre-plan hub merges contain
  fabricated cross-language edges and unmarked evidence).

Tool names not in the table pass at any version: probe-verus and
probe-rust extracts carry no imported or graph-inexpressible evidence,
so there is nothing for re-enrichment to launder and they need no gate
entry. probe-verus does still need a **dependency-bump release outside
the gate** (ninth Codex review): its extract pipeline compiles in the
pinned hub v0.4.0 enrichment (../probe-verus/Cargo.toml:37, calling
`enrich_verification_status` at
../probe-verus/src/commands/extract.rs:1044), and spec-less in-scope
functions are ordinary status-less atoms (extract.rs:726), so a fresh
Verus extract can still emit `f [transitively-verified] → helper [no
status] → bad [failed]` — a label revised P23 forbids. Every hub
recomputation path (merge, enrich, and projection's enrich-then-trim,
§3d) repairs it, so soundness does not rest on the bump; it is a
conformance fix so fresh extracts satisfy P23 directly (§8). The thresholds are constants recorded in ADR-006 — a fixed
five-entry match (the `probe` entry an interval, the rest floors —
below), not a capability-negotiation framework — and there is
**no schema-version bump in any producer**: the gate keys on
`tool.name`/`tool.version`, which every envelope already carries;
producers keep emitting schema 3.0 (3.x is additive, and the hub's 3.1
bump in §5 is hub-side only). A uniform "reject everything below schema
3.1" gate was considered and rejected: it would force releases and
re-extraction on unaffected producers for no soundness gain.

**The `probe` entry is an interval, not a floor (tenth Codex review).**
probe-verus ships a second composer the producer inventory missed:
`probe-verus merge-atoms` is an independent merge implementation, not a
caller of the hub's (../probe-verus/src/commands/merge_atoms.rs:58),
and its envelope writer hard-codes `tool.name: "probe"` while stamping
**probe-verus's own package version**, currently 8.0.1
(../probe-verus/src/metadata.rs:197, ../probe-verus/Cargo.toml:5).
Against a floor-only gate that pre-contract output appears *newer* than
any hub 0.x threshold, so the §3c induction ("a post-threshold composed
artifact was produced by a tool that itself validates authority") is
false for artifacts producible today. The output also loses evidence,
independently of validation: `AtomWithLines`
(../probe-verus/src/lib.rs:245) carries no verification-status field
and no extension map, so the command silently drops statuses,
`status-origin` markers and correspondence records — `g [failed]` goes
in, a status-less real `g` comes out, and a later hub merge resolving a
verified caller's stub against it re-enriches the caller to
`transitively-verified` where the retained `failed` would have blocked
it. Fix: the `probe` gate entry accepts only
`threshold ≤ tool.version < 1.0.0`, and additionally rejects at any
version a `probe` envelope whose `tool.command` is `merge-atoms` — a
command the hub never shipped, so the check catches hypothetical
0.x-era Verus composer outputs the interval alone would pass (one
string comparison on the raw envelope, not identity verification). The
ceiling is a reserved constant exactly like the thresholds (§9); it is
safe to fix upfront because the hub's own releases stay on the 0.x
line, and crossing 1.0 is a coordinated major release that re-bumps
composers anyway. The regression test uses the actual legacy envelope
shape (test l). The command itself is retired or delegated in the
probe-verus rollout (§8).

The gate still does **not** recurse into merged inputs:
`InputProvenance` carries only `schema` + `source` (src/types.rs:63).
The table supplies the missing base case instead — fresh extracts are
gated directly, pre-threshold composed artifacts are rejected outright,
and a post-threshold composed artifact was produced by a tool that
itself validates authority (§3c), so "composer output is built only
from contract-compliant inputs" now holds by a real induction with no
provenance change. The §8 regeneration policy extends from merged
artifacts to affected producer extracts: pre-marker probe-lean extracts
are re-extracted, not re-used.

**The version gate also guards the hub's read-only consumers (eighth
Codex review).** As formulated above the gate protected only
recomputation paths, but the §6 summary contract fails without it: a
pre-marker Aeneas extract carries `verified` with **no**
`status-origin`, so the `imported-verified` classification — keyed on
the marker — cannot catch it, and `probe summary` (which loads through
`load_atom_file`, src/commands/summary.rs:108, with no validation)
would present imported evidence as an unqualified local result —
reproduced against the current binary. `probe project` is the second
hole: it loads unvalidated (src/commands/project.rs:195) and re-stamps
its output `tool.name: "probe"` at the *current* version (:240),
concealing a pre-contract origin from any later outer-envelope check.
So `probe summary` and `probe project` run the **version-gate
component** of the shared validator (§3c) at their load boundary. The
projection-rejection component deliberately does not travel with it:
projections are views with inherited labels (§3d) and remain readable
by summary and project (§10).

Tests: keep `test_idempotency_transitively_verified` (propagate.rs:673 —
still passes); add (a) a downgrade test: atom labelled
`transitively-verified` whose dep is `unverified` is rewritten to
`verified`; (b) the three-node chain above (transparency); (c) a chain
through a `trusted` atom (boundary: no contamination), plus the
whole-atom-trust variant where the unverified atom hangs off the
trusted atom's spec/type dependency; (d) a test that
`maps-to`/`mapped-from` records do not contaminate; (e) a merge test:
stub resolution against an unverified real atom downgrades the caller's
label in the merged output; (f) merge and enrich reject both a
`probe/projected-atoms` input and a legacy-format projection
(`probe/merged-atoms` schema + `projection` field) (§3d); (g) an atom
with `status-origin: "translation"` is never promoted to
`transitively-verified` but is still downgraded when contaminated
(§2/§3a); (h) a locally verified caller of a translation-origin
`verified` atom is not promoted (promotion blocker, §2), while the same
caller behind a `trusted` atom is; (i) a translation-origin atom
arriving as `transitively-verified` is unconditionally rewritten to
`verified` (§2); (j) trusted-precedence pair (§3b): a caller of
`trusted` + `status-origin: "translation"` is not promoted, a caller of
a plain local `trusted` is; (k) kernel-taint synthetic replicas of the
probe-lean fixtures (§2): a `verified` atom with empty dependencies and
`status-origin: "kernel-taint"` stays `verified` through merge with an
empty map, likewise with its taint dep absent from the map, and a
locally verified caller of either is not promoted — hub-side synthetic
graphs, not cross-repo fixture imports (probe-lean's own rollout tests
its aux-fold fixtures); (l) legacy-gate tests (§3f): pre-threshold envelopes from each gated
producer — a probe-lean single-tool extract, a hub-merged artifact
(`tool.name: "probe"`), an Aeneas composed extract and a probe-vcvio
re-emitted extract — are rejected by merge and enrich; post-threshold equivalents and an ungated-tool
envelope (e.g. probe-verus) are accepted, and the legacy Verus-composer
shape — `tool.name: "probe"` at version 8.x with `tool.command:
"merge-atoms"` — is rejected both by the `probe` interval's ceiling and
by the command check (tenth review, §3f); (m) an Aeneas-shaped composed
envelope (`probe-aeneas/extract`, two `inputs`, no `source`)
round-trips load → merge → serialize → validate with both source
entries surviving into the merged inventory (§6); (n) read-only gate
(§3f): summary and project reject a pre-threshold Aeneas envelope, and
summary over a modern projection whose atoms carry `status-origin:
"translation"` still lists them under `imported-verified` (projections
stay readable; imported evidence stays qualified); (o) summary over an
extract containing a verified Lean theorem plus its bound blueprint
node atom reports exactly one verified lemma — `language: "blueprint"`
atoms appear in no verified list and do not affect entrypoint
classification (§6, tenth review).

## 4. Algebra: what must hold and why

The laws live in docs/categorical-framework.md §Algebraic Laws, with
normative copies in `kb/engineering/properties.md` P4/P5 and
`kb/tools/probe-merge.md`. They are currently stated unconditionally and
fail in specific ways. Target formulation:

**Factor merge into two operations.**

- `μ(A, B)` — plain merge: normalize **each input** (P8, before
  conflict resolution — the ordering selects evidence, §3d), union with
  the category's conflict rule (P6 atoms first-wins with stub
  replacement, P7 specs/proofs last-wins), then enrichment
  recomputation (§3c).
- `F_M(A)` — attachment: for fixed mappings M, attach correspondence
  records (§5) to the atoms of A.

`probe merge` without `--mappings` computes `μ`; with `--mappings` it
computes `F_M ∘ μ`.

**Laws, stated on the carrier.** The carrier is the set of normalized,
enrichment-consistent atom maps. The carrier is a *precondition
established by normalization + enrichment*, not an automatic property
of tool outputs: P23 permits `--skip-enrich`, so even modern extractor
output can be off-carrier; μ's own normalize+enrich brings any
*authoritative* input onto it. Projected artifacts are excluded from
the carrier and from μ's domain entirely — they are rejected, not
normalized (§3d). Every merge output is a fixed point.

1. **Identity**: `μ(A, ∅) = A` exactly on the carrier (legacy inputs are
   first brought onto the carrier: `μ(A, ∅) = enrich(normalize(A))`).
   Note identity does *not* hold for the mapped merge:
   `F_M(μ(A, ∅)) = F_M(A) ≠ A` when A lacks records — which is correct
   behavior, not a bug; the law to state is F_M's idempotence.
2. **Associativity**: `μ` is associative on the carrier. Idempotence of
   enrichment alone does not give this; the argument is that enrichment
   preserves graph structure, stub classification and the recoverable
   base statuses (it only rewrites `verified` ↔ `transitively-verified`),
   so enriching an intermediate merge can never change which atom a
   later conflict selects. Test exactly that: enrichment of
   intermediates does not change the eventual selected base data. With
   mappings, associativity follows from the compatibility laws below.
3. **Commutativity for disjoint keys**: as before. Stub-vs-real also
   resolves order-independently (real wins either way); the genuine
   order-dependence is real-vs-real (P6) and spec/proof key overlap (P7)
   — deliberate: argument order is the user's freshness knob.
4. **Mapping compatibility** (replaces "functoriality holds naturally"):
   `F_M(μ(A, B)) = μ(F_M(A), F_M(B))` and `F_M(F_M(A)) = F_M(A)`.

**What makes law 4 hold.** Attachment must be:

- **Unconditional** — an atom gets its record whether or not the target
  is present in this invocation's key set. Keeping a P13-style existence
  check makes attachment depend on the surrounding key set and breaks
  compatibility across groupings. A dangling `maps-to` target is no
  worse than a dangling dependency (already tolerated: warned, treated
  as trusted). Dangling target ⇒ warning, not skip.
- **Set-like** — records are a set (identity: the full
  `(target, confidence, method)` triple); re-application is a no-op.
- **Key-local** — attachment depends only on the atom's own code-name.

Then first-wins picks the same winner regardless of grouping and both
sides of law 4 attach the same records. Doing cross-language *resolution*
at merge time would rewrite `dependencies` from co-present atoms and
destroy law 4 again — resolution stays a derived consumer view (§2).

**Envelope level.** Laws are stated modulo envelope meta (`timestamp`,
tool — merge writes fresh values) and over provenance as a
**deduplicated source inventory**: the `inputs` array records *which*
sources were composed, not how many times. `cmd_merge` extends
provenance from every input (src/commands/merge.rs:311) and
`merge_atom_files` flattens recursively (merge.rs:191); both dedup
identical entries, giving envelope idempotence up to meta. Preserving
the inventory also requires the loader fix in §6: composed detection is
structural (an `inputs` array), not keyed on the `probe/merged-` schema
prefix — the prefix check silently discards the Rust+Lean inventory of
a `probe-aeneas/extract` envelope. This definition of provenance goes
in the KB.

## 5. Target design: the maps-to format

**Wire format.** `Atom.extensions` is `#[serde(flatten)]`
(src/types.rs:107), so extension fields are top-level fields on the atom
JSON (precedent: `verification-status`). On the `from` atom
(implementation side, e.g. Rust `g`):

```json
{
  "code-name": "probe:crate/1.0/mod/g()",
  "...": "...",
  "maps-to": [
    { "target": "probe:Pkg.Mod.ga", "confidence": "exact", "method": "rust-qualified-name" }
  ]
}
```

On the `to` atom (formal side, e.g. Lean `ga`): the mirror field
`mapped-from` with `"target": "probe:crate/1.0/mod/g()"`.

- **Direction**: the mappings file's `from`/`to` are generic
  **source/target** (docs/mappings-spec.md — the `sources` block
  describes each side but assigns no implementation/formal roles; a
  Lean→Rust file is legal). `maps-to` goes on the `from` atom,
  `mapped-from` on the `to` atom, whichever languages the sides are.
  Naming: `maps-to` deliberately claims only "there is a
  mappings-file entry linking these names, with this confidence" —
  `translates-to` over-claims for manual abstraction mappings,
  `corresponds-to`/`same-definition-as` over-claims for abstractions
  (`ga` is a model of `g`, not the same definition).
- **Record fields**: `target`, `confidence`, `method`, copied from the
  mappings file. Confidence levels (per docs/mappings-spec.md): `exact`,
  `exact-disambiguated`, `file-and-name`, `file-and-lines`, `heuristic`,
  `manual`. `method` is optional in the spec (src/types.rs:306) and
  stays optional in records (omitted when absent).
- **Determinism** (P14): arrays sorted by `(target, confidence, method)`
  with absent `method` ordering as the empty string; record identity is
  the same triple under the same convention, and duplicates collapse.
  Records with the same target but different confidence/method are
  distinct assertions and both kept.
- Records attach only to atoms present in the merged map (you attach to
  `g`; `g` exists; the *target* may dangle → warning).
- Kebab-case throughout (schema convention).

**Mirrors are best-effort, not an invariant.** If the target atom is
absent at attachment time and arrives in a later *plain* merge, it
carries no `mapped-from` (different key — the union rule below cannot
repair it). The KB defines the correspondence relation as the **union
over both fields**: consumers index `maps-to` and `mapped-from` into
one relation and derive reverse lookups, so a missing mirror loses no
information. Re-running merge with the mappings file (or regenerating,
§8) restores mirrors.

**Preservation through merge (new property).** Whole-atom conflict
resolution (P6) would silently drop records: a record-carrying stub
replaced by a record-less real atom, or a record-less first atom winning
over an annotated second, loses the records — and later merges cannot
re-derive them without the mappings file. So correspondence records get
a narrow carve-out from whole-atom semantics: **on every equal-key
resolution — stub replacement, real-vs-real first-wins, stub-vs-stub,
and post-normalization collisions within a single input (§6) — the
surviving atom's
`maps-to`/`mapped-from` arrays are the set union of both sides'**. This
is decidable and specific to correspondence records; general extension
union (review issue 4) remains out of scope.

**Implementation changes required** (beyond the attachment loop):

- `load_mappings` (src/types.rs:324) currently returns endpoint-string
  lookup maps and discards `confidence`/`method`. Change it to return
  full `Mapping` records.
- `MappingsFile` (src/types.rs:312) does not parse the `sources` block —
  and keeps not parsing it (sixth revision): it is descriptive metadata
  that assigns no roles, nothing reads it, and serde tolerates the
  unknown field. The Direction note above stands on the mappings-spec
  alone.
- Normalize mapping-file endpoints (P8, trailing-dot strip) *before*
  lookup, not just stored record targets.
- Remove the current edge-injection block in `merge_atom_maps`
  (src/commands/merge.rs:87, injection around lines 132-168).
- Attachment cost (seventh Codex review): one pass over the mapping
  records with two map lookups per record (source atom, mirror atom) —
  no endpoint index (dropped in §3e) and no per-atom scan of the
  mappings list.

**Schema**: bump 3.0 → 3.1 (minor: new optional fields, no cross-repo
lockstep; see the bump runbook in kb/engineering/schema.md). The
*behavioral* break — cross-language edges no longer appear in
`dependencies` — is coordinated through the downstream rollout (§8), not
the schema number.

## 6. Other code fixes

**Issue 6 — collision policy.** `normalize_atoms`
(src/commands/merge.rs:31) already applies stub-replacement +
first-wins on a post-normalization collision (merge.rs:60-68) — but
silently; `normalize_generic` (src/commands/merge.rs:204)
unconditionally inserts, last-wins, silently. Fix: keep each category's
existing rule, add a counted warning feeding `stats.conflicts`, and
apply the correspondence-record union (§5) on atom collisions. PR
ownership (seventh Codex review): the record union is part of §5's
preservation guarantee ("every equal-key case" includes normalization
collisions), so the union helper lands in PR 3b, shared between the
merge conflict path and `normalize_atoms`; only the counted warnings
are PR 3c.

**Composed-provenance detection is structural (seventh Codex review).**
`load_envelope` recognizes composed provenance only when the schema
starts with `probe/merged-` (src/types.rs:233); anything else takes the
single-tool path, and when `source` is absent the loader *manufactures*
an empty fallback. probe-aeneas emits a composed envelope under its own
schema string — `probe-aeneas/extract` with `inputs: [Rust, Lean]` and
no `source` (../probe-aeneas/src/extract.rs:1267) — so the hub silently
replaces the flagship composed artifact's inventory with an empty
fabricated entry, contradicting the §4 source-inventory contract (dedup
cannot restore destroyed entries). Fix: detect the composed shape by
the presence of `inputs` and the single-tool shape by `source` — one
structural check replacing prefix matching, automatically covering
`probe/merged-atoms`, `probe/projected-atoms` and
`probe-aeneas/extract` (this supersedes the §3d prefix-widening). Side
note for the §8 blueprint issue: probe-leanblueprint composes but emits
a single-`source` envelope, so its inventory is unrecorded — fixed in
the §8 blueprint rollout (composed `inputs` emit), no longer a deferred
follow-up (eighth Codex review).

**Provenance dedup.** In `merge_atom_files` flatten
(src/commands/merge.rs:191) and the `cmd_merge` path (merge.rs:311),
dedup identical provenance entries (§4, envelope idempotence).

**P8 coverage.** Extend normalization to all code-name-bearing extension
arrays (`requires-dependencies`, `ensures-dependencies`,
`body-dependencies`, `type-dependencies`, `term-dependencies`,
`dependencies-with-locations` — verify the full list against schema.md)
and to `maps-to`/`mapped-from` targets and mapping-file endpoints.

**P15 in `probe project`.** Projection trims `dependencies` to the
included set but clones the categorized extension arrays unchanged
(src/commands/project.rs:117-129), breaking the P15 decomposition. Fix:
trim the categorized arrays with the same filter.

**Executable schema (`schemas/atom-envelope.schema.json`) — second
Codex review.** The plan changes behavior this schema constrains, so it
is an implementation target, not just docs: relax the merged
specs/proofs `inputs` `minItems: 2` (line ~119) to 1 — provenance dedup
(§4) can legitimately collapse two identical sources to one entry; add
the `probe/projected-atoms` envelope and rework the branch split to key
on provenance shape — a `source` branch and an `inputs` branch with the
schema strings constrained per branch — so the Aeneas composed envelope
validates instead of matching neither branch (§3d and the
structural-detection fix above; today the `mergedAtomsEnvelope` `const`
accepts only `probe/merged-atoms` and the single-tool branch requires
`source` and forbids `inputs`); add `probe-vcvio` to the single-tool
branch's tool pattern (thirteenth Codex review —
schemas/atom-envelope.schema.json:21 lists only
rust/lean/verus/aeneas/leanblueprint, so a `probe-vcvio/extract`
envelope the loader and §3f gate accept as contract-compliant would
still fail the executable schema), with a positive validation fixture
carrying `status-origin`; add
`maps-to`/`mapped-from` record definitions (§5); add the optional
`status-origin` field as an enum over exactly `"translation"` and
`"kernel-taint"` (§2 — fifth Codex review: without it the validation
tests cannot establish the evidence contract). The
summary contract (below) gets **no JSON schema** (sixth revision): the
earlier phrasing "update the summary envelope schema" was vacuous —
`schemas/` contains only the atom envelope — and summaries are
human/CI-facing output, not a cross-tool interchange contract; the
`imported-verified` list is covered by plain unit tests on
`SummaryResult`. PRs 3a/3b include envelope-level validation tests
(serialize a merged/projected envelope, validate against the JSON
schema), not only map-level algebra tests.

**`probe summary` consumer contract (fourth Codex review).** The origin
qualifier §2 introduces disappears in the hub's own summary consumer:
`summarize_atoms` treats `verified` and `transitively-verified`
identically with no origin check (src/commands/summary.rs:34), can place
a translated Rust leaf into `verified_entrypoints` (summary.rs:79), and
serializes bare name lists that drop the qualifier permanently
(summary.rs:24) — so the §2 laundering counterexample, even correctly
kept off `transitively-verified`, still surfaces as an unqualified
verified entrypoint. Decision (ADR-006): summary must not present
imported evidence as a locally established result. Membership in the
new fourth list, `imported-verified` (names only, deterministic order),
is **`status-origin == "translation"` AND status ∈ {`verified`,
`transitively-verified`}** — the conjunction is required (fifth Codex
review) because the marker rides on *every* copied status: Aeneas also
copies `trusted`, `failed` and yields `unverified` when no primary-spec
evidence exists (../probe-aeneas/src/extract.rs:868). Today those never
reach the lists only because of summary's initial verified-status
filter (src/commands/summary.rs `is_verified`); the new contract states
the condition explicitly rather than inheriting it, and a translated
non-verified status appears in **no** verified list. Add a
status-matrix test (both origins × the five statuses).
`status-origin: "kernel-taint"` atoms stay where
they are — that evidence is local (a real, locally checked proof; the
marker only makes it non-promotable), like any other contaminated
locally-verified atom summary already includes. Update `SummaryResult`,
its serialization and unit tests (no JSON schema — see the executable
schema paragraph above). The contract is enforceable only behind the
§3f version gate (eighth Codex review): an unmarked legacy `verified`
is indistinguishable from local evidence, which is why summary itself
runs the gate.

**Summary excludes blueprint-language atoms (tenth Codex review).** For
a clean theorem `T`, probe-leanblueprint emits both `T` and its bound
node atom with an inherited verified status; `summarize_atoms` places
every verified non-Rust-exec atom into `verified_lemmas`
(src/commands/summary.rs:69), so one checked theorem reports as two
verified lemmas, and multiple bindings inflate further. Neither atom
carries `status-origin`, so the `imported-verified` list cannot catch
it — and blueprint's own normative schema already instructs generic
consumers to filter these atoms
(../probe-leanblueprint/docs/SCHEMA.md:175). Contract addition
(ADR-006): summary's verification partitions are computed over **code
atoms** — atoms with `language: "blueprint"` appear in no verified list
and are likewise excluded from the `depended_upon` set used for
entrypoint classification, so presentation-layer binding edges cannot
alter code entrypoints. Test: theorem plus bound node yields exactly
one verified lemma (test o).
Warning-only diagnostics with no soundness content; dropped from the
current change set to keep it small (was PR 4). Recorded design for
whenever it is picked up:
warn when two inputs have the **same `source.repo` but different
`source.commit`** (cross-repo disagreement is the normal heterogeneous
case — Rust and Lean extracts come from different repos — and must not
warn); warn when a last-wins override replaces an entry from an input
whose envelope timestamp is newer than the replacement's. Both checks
are **input-level diagnostics** for a single invocation: per-entry
origin is not tracked through recursive merges (a merged file's
timestamp is merge time, not extraction time), and the KB documents this
limit.

## 7. KB and doc edits (Phase A, PR 1 — lands first; KB is source of truth)

1. **New ADR-006 "Correspondence records (maps-to)"** superseding
   ADR-003's *application* semantics only (generation — probe-aeneas
   strategies, 1-to-1 generation, 1-to-many acceptance, bidirectional
   files — is untouched). Content: §2 rationale; the `status-origin`
   mechanism (§2 — two values, `"translation"` and `"kernel-taint"`;
   every bearing atom is a blocker seed: never promoted, blocking for
   reaching atoms, unconditionally demoted from imported
   `transitively-verified`, and explicitly not a trusted boundary,
   including the copied-`trusted` precedence rule §3b); the summary
   consumer contract (§6 — translation-origin atoms qualified as
   `imported-verified`; kernel-taint evidence is local); §3
   merge/enrichment contract, including the
   whole-atom-trust choice with the producer-audit findings on
   `trusted`/`trusted-reason` (§3b), the
   labels-are-not-revalidated-proofs caveat (§3b) and the
   projection-rejection decision (§3d); mirror/lifecycle semantics (§5 —
   mirrors best-effort, the relation is the union of both fields); the
   §8 migration policy extended to mapping corrections/withdrawals,
   affected producer extracts and composed artifacts, with the §3f
   per-producer gate table (five `tool.version` thresholds recorded as
   constants, the `probe` entry an interval with a reserved 1.0.0
   ceiling plus the `merge-atoms` command rejection — tenth review; no
   producer schema bumps); the composer rule — any producer that runs
   enrichment over foreign atoms *or re-emits foreign verification
   evidence under its own tool identity* validates authority first
   (hub, probe-aeneas staging, probe-leanblueprint, probe-vcvio, and
   probe-verus's retired-or-delegated `merge-atoms` and its
   `--skip-atomize` cached-atom input — the rule's third shape,
   constructing authoritative verification output over a
   previously-emitted dependency graph, twelfth review,
   §3c/§3f/§8); carrier preparation from shared normalize/enrich
   primitives at every recomputation boundary — per-input normalization
   before conflict resolution in μ, composed `enrich ∘ normalize` on
   unary boundaries (§3d, twelfth and thirteenth reviews); the blueprint
   synthesis rule — synthesized node statuses propagate contributors'
   `status-origin` markers, exact precedence recorded with the §3b
   audit findings (§8) — together with the code-atom scope of the
   assurance contract: synthetic `language: "blueprint"` statuses are
   presentation aggregates, never whole-closure trust boundaries (§3b,
   thirteenth review);
   provenance as a structurally detected source inventory (§6);
   defers `verified-by-translation`.
2. **kb/engineering/schema.md**: rewrite §Cross-language mappings
   (currently: "mapped code-name(s) are added as additional
   dependencies", both directions, existence-checked, no-dup) to the §5
   design; add `maps-to`/`mapped-from` and `status-origin` (both values,
   §2) to the extensions documentation (flattened, top-level); rewrite
   the `verification-status` row (~line 149) — its transitive-status
   wording is still the old unrestricted closure; align with revised
   P23 (eighth Codex review: adding a `status-origin` row does not
   repair the definitions themselves); bump
   schema-version to 3.1 + version-history row.
3. **kb/engineering/properties.md**:
   - **P13** (cross-language edges require existence) rewritten:
     correspondence records attach unconditionally; dangling targets
     warned, not skipped.
   - **P15**: scoped honestly — the decomposition equality applies to
     atoms that carry categorized subsets (plain Rust atoms carry none);
     new clause: every transformation (merge, project) must preserve the
     decomposition where present.
   - **P4/P5**: restated per §4 — μ/F_M factoring, carrier = normalized
     enrichment-consistent maps, modulo envelope meta and provenance
     inventory; the compatibility law replaces "functoriality".
   - **P23**: restated as the single path-based definition in §3b,
     which *replaces* the "every transitively reachable dependency"
     wording (missing-status atoms transparent by construction,
     `trusted` a boundary only when it carries no `status-origin`,
     whole-atom trust); enrichment is a
     recomputation (downgrades contaminated `transitively-verified` →
     `verified`); merge re-enriches its output via the shared
     implementation (§3c); `maps-to` excluded from contamination;
     one seed set — `failed`/`unverified` plus every
     `status-origin`-bearing atom (`"translation"` and
     `"kernel-taint"`), seeds never promoted, blocking atoms that reach
     them along non-trusted paths, unconditionally demoted from
     imported `transitively-verified` (§2);
     labels assert graph-consistency, not proof re-validation; carrier
     is a precondition (`--skip-enrich` outputs are off-carrier);
     projected inputs are views and are rejected by merge/enrich (§3d);
     projection labels are recomputed on the full source graph at
     projection time, never from the trimmed view (§3d, ninth review);
     the assurance contract, including the trusted-boundary reading, is
     stated over code atoms — `language: "blueprint"` synthetic
     statuses are presentation aggregates (§3b, thirteenth review).
   - **P16** (kb/engineering/properties.md:136): its enrichment
     description ("upgrades verified → transitively-verified") conflicts
     with recomputation semantics; align with revised P23. The whole
     Lean status table is rewritten too (eighth Codex review):
     properties.md:125 (statuses "determined by sorry detection" —
     warning-based) and :129 (`*External.lean` ⇒ `trusted`,
     unconditional) describe a classification probe-lean no longer
     implements — statuses are kernel-based
     (../probe-lean/ProbeLean/Transitive.lean:25) and External-module
     trust excludes proofs (../probe-lean/ProbeLean/Trust.lean:71; the
     aux-fold fixture pins `extThm` as `unverified`). The hub KB states
     a short evidence contract and points to probe-lean's own docs for
     classification detail rather than maintaining a competing table.
     Must land in PR 1: the §3b producer audit would otherwise read the
     stale table as ground truth.
   - **New property**: `maps-to`/`mapped-from` are unioned through every
     atom-level conflict, covered by P8 normalization, deterministic
     order, never contaminate.
   - **P8**: extended to code-name-bearing extension arrays and mapping
     endpoints.
   - Provenance defined as a deduplicated source inventory.
4. **kb/tools/probe-merge.md**: restate the laws per §4 (currently
   claims associativity, identity, commutativity without caveats);
   document post-merge enrichment and input authority validation
   (projection rejection §3d, version gate §3f). Staleness diagnostics
   are deferred (§6) and not documented as existing behavior.
5. **docs/categorical-framework.md** (non-normative, but must match):
   - Laws section: replace the four laws with the §4 formulation.
   - DOTS table: "Composition with functor — `probe merge --mappings`"
     reworded: the tight morphism is *structural data carried alongside*
     the composition (correspondence records), not edges inside the
     interaction ("state separation: only the interface participates in
     composition").
   - SSProve table: "Translation / simulation ... so linking can resolve
     cross-language dependencies" → "resolution is a derived view over
     maps-to records".
   - SSProve table, "State separation" row (third Codex review): the
     opacity claim is wrong as stated — `Atom::is_stub` inspects
     `code-path` and the `code-text` line fields to decide which atom
     wins (src/types.rs:113-117), and merge/enrichment interpret
     verification status, status origin, trust and correspondence
     metadata. Limit opacity to "merge never interprets source-code
     *bodies*"; structural stub classification and the interpreted
     metadata are part of the composition interface.
   - §Universal composition operator, heterogeneous merging: "supply
     `--mappings` to mediate cross-language dependencies" reworded to
     record-attachment semantics; same for the "doctrine signature"
     paragraph.
   - §Functor factory: "merge with cross-language edges" / "`probe merge
     --mappings` handles the rest" reworded: merge attaches the
     correspondence; consumers derive linkage.
   - Opening claim "cross-language mappings are functors between
     language categories" (line 8) and the DOTS/SSProve tables overall:
     label these as architectural *analogies* — the implemented algebra
     is annotation-preserving composition; cross-language linking is a
     deferred capability, not an achieved one.
   - §Testable Compositionality: law list must match §4 (drop
     unconditional functoriality).
   - §Provenance as Composition Trace: provenance is a deduplicated
     *source inventory*, not a record of the composition process or its
     conflict decisions.
   - Closure claims (fifth Codex review): "the single composition
     operator for all probe outputs" (line 61) and "closed over the
     space of all probe outputs" (line 110) are false under §3d/§3f —
     restate closure over *authoritative, same-category inputs
     satisfying the evidence contract*, i.e. the §4 carrier.
   - §Consistency for New Probes (line ~103): "produce atoms in the
     standard envelope format" understates the doctrine obligation —
     add that a producer whose verification evidence is not expressible
     in the emitted graph must stamp `status-origin` (§2), or hub
     enrichment over its output is unsound.
6. **kb/engineering/glossary.md**: update "cross-language mapping"
   ("Applied by `probe merge --mappings` to add cross-language
   dependency edges" → attaches correspondence records).
7. **docs/mappings-spec.md §Semantics** ("the merge tool treats the two
   code-names as representing the same logical entity", plus the
   edge-adding description) and **docs/merge-algorithm.md**
   §cross-language stub resolution (still describes dependency-edge
   injection): rewrite both to record-attachment semantics.
   merge-algorithm.md additionally gets a **full phase-outline
   rewrite**, not just its cross-language section (tenth Codex review):
   its loading phase still claims compatible major version `2`
   (docs/merge-algorithm.md:33) and its phase sequence omits authority
   validation (§3d/§3f) and post-merge enrichment recomputation (§3c) —
   left as is, the algorithm doc and the categorical framework describe
   different operations.
8. **docs/SCHEMA.md** (second Codex review): line ~196 keeps the old
   upgrade-only transitive-status description (align with revised P23);
   line ~318 says projection metadata "is ignored by unaware consumers"
   — the opposite of the §3d authority boundary; document
   `probe/projected-atoms` and the rejection rule.
9. **kb/tools/probe-aeneas.md** (~line 36): still mandates
   existence-checked cross-language mapping *edges*; rewrite to
   record-attachment semantics and document the `status-origin:
   "translation"` marker and single-final-enrichment pipeline (§2, §3c).
10. **kb/tools/probe-lean.md**: document the `status-origin:
    "kernel-taint"` marker — stamped whenever the taint pass runs,
    including under `--skip-enrich`; unmarked `verified` atoms are
    promotable by hub enrichment, marked ones never are (§2).
11. **kb/tools/probe-summary.md**: document the `imported-verified`
    list, its membership condition (translation origin AND verified
    status), and the rule that translation-origin atoms never appear in
    the three verified partitions (§6); document the blueprint
    exclusion — `language: "blueprint"` atoms appear in no partition
    and are excluded from entrypoint classification (§6, tenth review).
12. **kb/tools/probe-project.md** (fifth Codex review): lines ~53 and
    ~80 mandate reuse of the `probe/merged-atoms` envelope and P1
    conformance to it — update to `probe/projected-atoms` (§3d) and
    document that projections are rejected by merge/enrich; line ~86's
    pointer to the `projection` field on the *merged* envelope moves
    with it; document normalize-then-enrich-then-trim — the
    authoritative input is normalized (P8) and labels recomputed on the
    full graph before trimming, mapping seeds matched on normalized
    names; labels inherited unchanged from already-projected inputs
    (§3d, ninth and twelfth reviews).
13. **kb/tools/probe-vcvio.md** (new — eleventh Codex review): the
    ecosystem KB has no page for probe-vcvio at all; add one covering
    its role (security-protocol classification annotator over
    probe-lean extracts), the re-emitter composer obligation (§3c —
    validate input authority before re-stamping under its own
    identity) and its §3f gate entry.
14. **kb/tools/probe-leanblueprint.md**: document the composer
    obligation (§3c) — call the shared authority validator before
    enrichment, preserve `status-origin` markers — and note that
    pre-release blueprint outputs are rejected by the §3f gate;
    document the synthesis rule (§3b/§8) — `derive_synthetic_verification`
    propagates contributors' `status-origin` markers, never emitting an
    unmarked `trusted`/`verified` over marked evidence — and the scope
    note that blueprint synthetic statuses are presentation aggregates
    outside the code-atom assurance contract (§3b, thirteenth review).

## 8. Migration and downstream (separate repos, after hub merges)

**Existing merged artifacts** contain fabricated cross-language edges
with no origin marker; they cannot be safely repaired in place. Policy
(recorded in ADR-006): merged files are derived artifacts —
**regenerate** from the original extracts and mappings after upgrading.
"Original extracts" is qualified (fifth Codex review): a pre-marker
probe-lean extract is itself an affected artifact — regenerating a
merge from it still launders — so the policy covers **producer
extracts too**: pre-threshold probe-lean extracts are re-extracted with
the marker-capable release, and the §3f version gate makes the
prerequisite checkable at the file boundary instead of relying on
release ordering. The gate covers composed artifacts too (seventh
review): pre-threshold hub merges, Aeneas extracts and blueprint
outputs are rejected outright, making "unsupported wholesale" a
mechanism rather than a policy.
No edge-stripping tooling. The same policy covers **mapping corrections
and withdrawals**: attachment is set-union and never removes a record,
so a corrected or retracted mapping takes effect only by regenerating
from extracts + the corrected mappings file, never by re-merging over
the old artifact.

- **probe-aeneas**: its `extract` pipeline calls hub merge with
  mappings; output changes from injected deps to records. Additionally
  (second Codex review): migrate to the raw staging primitive so
  enrichment runs exactly once, after the metadata phase (§3c — avoids
  a wasted double pass and keeps `--skip-enrich` meaning "no enrichment
  at all"); mark every status copied from a Lean atom with
  `status-origin: "translation"` and normalize the copied value to
  `verified` when the source is `transitively-verified` (§2); keep the
  generated `Vec<Mapping>` authoritative through the pipeline (third
  Codex review) — `run_translate` currently reduces to endpoint-only
  maps, discarding `confidence`/`method`
  (../probe-aeneas/src/extract.rs:745-750), and passes those to merge
  (:791) without touching the hub's `load_mappings`, so fixing the
  loader alone leaves the primary producer unable to deliver the
  confidence filtering that motivates §2; endpoint maps become derived
  indexes over the P8-normalized records (twelfth review — one
  normalization rule on both sides of every lookup) and confidence is
  never reconstructed from endpoints. Add
  end-to-end regression tests: the laundering counterexample (Lean
  theorem `verified` over an `unverified` lemma, copied onto a leaf
  Rust atom, must not come out `transitively-verified`); the
  caller-laundering variant (a locally verified Rust caller of the
  translation-marked atom must not come out `transitively-verified`,
  §2); transfer from an already-enriched Lean theorem (imported
  `transitively-verified` demoted); a copied `trusted` arriving marked
  and not shielding its callers (§3b precedence); a non-exact
  confidence and its
  method surviving into both correspondence fields. Update docs and
  tests; regenerate any checked-in merged fixtures.
- **probe-lean** (fourth Codex review): stamp `status-origin:
  "kernel-taint"` on every atom whose `verified` label reflects
  kernel-reachable taint the emitted graph cannot express (the
  `pt.taint.tainted` branch of `taintVerdict`,
  ../probe-lean/ProbeLean/Transitive.lean:20) — whenever the taint pass
  runs, including under `--skip-enrich`. Regression: run the existing
  aux-fold fixtures (`ownSorry`, `viaNoRange`) through its own pipeline
  and assert the marker; the hub covers the merge/enrich behavior with
  synthetic equivalents (§3 test k). Until this lands, probe-lean
  extracts must not be re-enriched by the hub — enforced by the §3f
  version gate, and pre-threshold extracts are re-extracted, not
  re-used (§8 policy).
- **probe-verus** (ninth Codex review): no marker, no gate entry — but
  its extract pipeline compiles in the pinned hub v0.4.0 enrichment
  (../probe-verus/Cargo.toml:37,
  ../probe-verus/src/commands/extract.rs:1044) with the pre-§3b
  transparency barrier, and spec-less in-scope functions are ordinary
  status-less atoms (extract.rs:726), so fresh extracts can emit labels
  revised P23 forbids. Rollout: bump the hub dependency to the
  recomputation release and regenerate checked-in fixtures. Hub
  recomputation paths (merge, enrich, project's enrich-then-trim)
  repair old extracts regardless, so this is conformance, not a gate
  matter. The same dependency bump wires the shared validator into the
  `--skip-atomize` cached-atom input boundary (twelfth Codex review,
  §3c): `load_enveloped_data` on the atoms path
  (../probe-verus/src/commands/extract.rs:831) rejects projections in
  both formats and pre-threshold envelopes before unwrapping, so a
  truncated view can no longer be re-emitted as a fresh authoritative
  extract. Test: a legacy-format projection fed as the cached atom file
  is rejected. Additionally (tenth Codex review): **retire `merge-atoms` or
  reimplement it as a caller of the hub's shared authority-validating
  merge**, stamping its own accurate tool identity
  (`tool.name`/`tool.version`), never the hub's — the current
  independent implementation drops verification statuses, markers and
  correspondence records (`AtomWithLines`, ../probe-verus/src/lib.rs:245)
  and masquerades as `probe` at version 8.x
  (../probe-verus/src/metadata.rs:197). The hub-side `probe` gate
  interval and command check (§3f) reject its legacy outputs
  regardless, so the retire-vs-delegate choice is decided in that repo
  without blocking the hub PRs.
- **probe-leanblueprint** (seventh Codex review): an authority-changing
  composer — it enriches foreign atoms with the hub's bare-map function
  and re-emits under its own tool identity (§3c). Rollout: bump the hub
  dependency; call the shared authority validator on the atom base
  before enrichment (rejecting projections in both formats and
  pre-threshold inputs); preserve `status-origin` markers end to end
  (PR 2's recomputation semantics then apply unchanged, since it reuses
  the hub function). The rollout also covers its status **synthesis**
  (tenth Codex review, §3b): `derive_synthetic_verification`
  (../probe-leanblueprint/src/main.rs:1065) ranks `trusted` above
  `verified` (../probe-leanblueprint/src/enrich.rs:689) and writes the
  aggregate without `status-origin` (enrich.rs:841), so a node binding
  a kernel-tainted `verified` theorem plus a trusted axiom would become
  an unmarked `trusted` whole-closure boundary under revised P23. Rule:
  synthesized statuses propagate contributors' markers — exact
  value/precedence recorded in ADR-006 alongside the §3b audit
  findings. The marker-free case is scoped, not synthesized around
  (thirteenth review, §3b): a contaminated `verified` contributor with
  no origin marker plus a trusted axiom still aggregates to an unmarked
  `trusted` — legal precisely because blueprint statuses sit outside
  the code-atom assurance contract. Tests: a legacy projection and a
  pre-marker Lean extract are rejected; markers survive into the
  emitted extract; a mixed marked-verified + trusted-axiom binding
  synthesizes a marked status, never an unmarked `trusted`; and the
  marker-free variant above is pinned against the ADR-006
  presentation-aggregate semantics. Its
  emit shape is fixed in the same rollout, no longer a follow-up
  (eighth Codex review): blueprint carries the loaded
  `Vec<InputProvenance>` through and emits a composed `inputs` envelope
  — today it selects a single `Source` and discards the rest
  (../probe-leanblueprint/src/main.rs:352,
  ../probe-leanblueprint/src/emit.rs:33), so `merged inventory →
  blueprint → hub merge` permanently loses source-inventory entries
  that structural detection and dedup cannot recover, contradicting the
  §4 contract; until it lands, the KB scopes the inventory guarantee to
  inventory-preserving producers. The same PR deletes the 2.x→3.0
  migration path (main.rs:237): the §3f gate rejects every input it
  could rescue — migrate-then-reject is dead work — and removing it
  collapses the triple parse of one input (read :237, `load_envelope`
  :344, `load_atom_file` :352) to a single load. Pre-release blueprint
  outputs are rejected at the hub by the §3f table.
- **probe-vcvio** (eleventh Codex review): an annotation re-emitter —
  it never enriches, but re-stamps probe-lean evidence under its own
  identity (§3c). Rollout: bump the pinned hub dependency
  (../probe-vcvio/Cargo.toml:34) and run the shared validator's
  version-gate component on the input envelope before re-emitting. Its
  exact-schema input check already rejects projections and other
  non-`probe-lean/extract` shapes (../probe-vcvio/src/model.rs:311), so
  only the version gate is missing; and it reuses the hub's `Atom`
  type, so `status-origin` markers already survive. Hub-side (PR 3a,
  thirteenth review): the executable schema's single-tool branch
  accepts `probe-vcvio/extract`, with a positive validation fixture
  carrying `status-origin` (§6). Tests: a pre-marker
  probe-lean extract is rejected; a marked input's `status-origin`
  survives into the emitted extract. Pre-release probe-vcvio outputs
  are rejected at the hub by the §3f table; pre-release outputs still
  needed are regenerated from re-extracted inputs (§8 policy).
- **scip-callgraph UI**: currently renders cross-language edges from
  `dependencies`; must read `maps-to`/`mapped-from` instead (and can now
  style/filter by confidence). Recommendation, decided in that repo:
  visually distinguish `status-origin: "translation"` statuses instead
  of rendering them as locally established.
- **probe project**: seeds from the mappings *file* are unaffected, but
  the earlier claim that the *same* mappings file preserves the selected
  vertex set was false (eighth Codex review, reproduced against the
  current binary): projection seeds only endpoints that *exist*
  (src/commands/project.rs:39), while old injection required only the
  mapped *target* to exist (src/commands/merge.rs:140). With `f → g`
  where `g` is absent (a dangling dependency — §5 deliberately supports
  these), `ga` present, mapping `g ↔ ga` and reverse-depth 1, the
  fabricated `f → ga` edge used to select `{f, ga}`; without it the
  selection is `{ga}`. Decision, made now rather than deferred:
  projection BFS stays **dependency-only** — the lost reachability came
  from fabricated edges, and traversing correspondence records to
  reproduce them would re-complicate projection for a case regeneration
  covers (§10). The reachability change is documented in
  kb/tools/probe-project.md, and a regression pinning dependency-only
  selection lands in PR 3c. Project also runs the §3f version gate on
  its input before re-stamping (its output carries `tool.name: "probe"`
  at the current version, src/commands/project.rs:240, which would
  otherwise conceal a pre-contract origin), and **recomputes enrichment
  on the full authoritative input graph before trimming** (ninth Codex
  review, §3d) — projected inputs keep inherited labels, everything
  else is brought onto the carrier so stale producer labels are not
  frozen into unrepairable views. Regression in PR 3c: a Verus-shaped
  extract with `f [transitively-verified] → helper [no status] → bad
  [failed]` projected at depth 0 comes out `f [verified]`.
  The bigger upfront blocker is the projection boundary itself (§3d):
  projected artifacts get the `probe/projected-atoms` schema and are
  rejected by merge/enrich, so downstream consumers of projections must
  be checked against the new schema string.

## 9. Execution

Repo conventions (CLAUDE.md + user global): GitHub issue first, draft PR
closing it, conventional commits referencing KB files (e.g.
`feat: attach maps-to correspondence records in merge (ADR-006, P13)`),
Ralph Loop (`/ambiguity-auditor`, `/code-quality-auditor`,
`/test-quality-auditor`, fix, repeat until clean) mandatory since merge +
schema are touched, then `cargo test`. Hub Cargo.toml minor version bump
+ CHANGELOG entries under `## [Unreleased]`.

| PR | Content | Depends on |
|----|---------|-----------|
| 1 | §7 KB/ADR-006 spec change (incl. §3b trusted-semantics producer audit + whole-atom-trust + copied-`trusted` precedence + code-atom scope of the assurance contract / blueprint presentation aggregates (thirteenth review), §2 `status-origin` mechanism with both values, §3d projection rejection, §6 summary contract) | — |
| 3a | §3d projected-atoms schema and rejection incl. legacy `projection`-field detection (types.rs plumbing) + §3f per-producer version gate (five entries, incl. composed-artifact test l; `probe` entry as interval with reserved ceiling + `merge-atoms` command rejection incl. the legacy Verus-composer regression, tenth review) + §6 structural composed-provenance detection (`inputs`/`source`) + shared authority validator (§3c) called from `cmd_merge`/`merge_atom_files`/`cmd_enrich` + version-gate hook in `probe summary`/`probe project` (§3f, test n) + two-branch executable schema incl. `probe/projected-atoms` and `probe-vcvio` in the single-tool branch with a positive `status-origin` fixture (thirteenth review) + projected-envelope and Aeneas-envelope validation tests (test m) | PR 1 |
| 2 | §3a/§3b enrich recomputation as one BFS over the unified seed set (`failed`/`unverified` + `status-origin` atoms) + traversal fix + trusted-boundary precedence + `status-origin` enum in the atom schema + §6 summary `imported-verified` list (membership condition + status-matrix unit tests) + §6 summary blueprint-language exclusion (test o, tenth review) + shared normalize-then-enrich preparation applied by `cmd_enrich` incl. the dotted-alias contamination regression (§3d, twelfth review) + tests (incl. j, k) | PR 1, PR 3a |
| 3b | §5 maps-to records in merge.rs (attachment, union rule, loader changes, edge-injection removal) + §3c shared-path enrichment incl. authority-checked raw staging primitive (same §3d/§3f rejection) + `maps-to`/`mapped-from` record definitions in the executable schema + merged-envelope validation test + law tests (associativity with mappings across groupings; identity exact on carrier, up-to-normalization on legacy; F_M idempotence and compatibility; commutativity disjoint keys; record preservation through every equal-key case; enrichment of intermediates does not change selected base data) + record union on every equal-key case incl. normalization collisions (§6 — helper shared with `normalize_atoms`) + two-input dotted-alias evidence-selection regression (per-input normalization before conflict resolution, §3d, thirteenth review) — tests run against `merge_atom_maps`, the operation §4 defines | PR 1, PR 2, PR 3a |
| 3c | §6 issue 6 counted collision warnings feeding `stats.conflicts` (the record union itself is PR 3b) + provenance dedup + `minItems` relaxation in the executable schema + envelope idempotence test (modulo meta, after dedup) + P8 extension + project.rs P15 trim + P15 decomposition test on merged output + dependency-only projection-selection regression (§8) + projection normalize-then-enrich-then-trim on authoritative inputs with seed matching over normalized keys, incl. the Verus-shaped stale-label regression (§3d/§8, ninth review) and the dotted-key projection-seed regression (§3d, twelfth review) | PR 3b |

Staleness warnings (former PR 4) are deferred (§6, §10). Each PR runs
the full Ralph Loop independently; the split is deliberate — the
riskiest changes (BFS rewrite, records) land in separate, reviewable
units. Landing order is **1 → 3a → 2 → 3b → 3c** (seventh Codex
review): `probe enrich` reaches the recomputation function without
going through merge, so the authority boundary (PR 3a) must exist
before any code path recomputes (PR 2).

Rollout-ordering constraint (fourth Codex review): once PR 2 lands,
`probe enrich` recomputes (and from PR 3b, hub merge always
re-enriches) — which is unsound over probe-lean extracts until
probe-lean ships the `kernel-taint` marker (§2, §8). The hub PRs may
land first; the §3f version gate (PR 3a) enforces the ordering
mechanically — pre-threshold extracts are rejected, not merely deferred
by convention (fifth Codex review). The ADR records the ordering and
the thresholds. Gate thresholds are the **actual contract-release
version numbers, reserved upfront** and compiled into the shared
validator before PR 3a lands — not placeholder-high values lowered in a
follow-up (ninth Codex review): the validator is compiled into each
composer through its pinned hub dependency
(../probe-aeneas/Cargo.toml, ../probe-leanblueprint/Cargo.toml:28,
../probe-vcvio/Cargo.toml:34), so
a hub-side lowering would strand any composer built against the
placeholder table until another round of dependency bumps and
releases. A reserved number rejects every existing artifact today (the
release does not exist yet) and becomes accepting the moment the
producer ships under it; each producer's contract release must ship
with exactly its reserved version. The `probe` entry's 1.0.0 ceiling is
reserved the same way (§3f, tenth review): safe because crossing 1.0 is
a coordinated major release that re-bumps composers anyway.

Downstream work is tracked as one issue per repo (repo conventions:
issue first, draft PR closing it), all depending on hub PR 1 for the
ADR-006 spec: **probe-lean** (kernel-taint marker, §8), **probe-aeneas**
(translation marker, raw staging, authoritative `Vec<Mapping>`,
regression tests, §8), **probe-leanblueprint** (authority validation,
§3c/§8), **probe-vcvio** (input version gate before re-emission,
§3c/§8), **scip-callgraph UI** (render `maps-to`/`mapped-from`, §8),
**probe-verus** (hub-dependency bump + `merge-atoms` retirement or
delegation + shared-validator check on the `--skip-atomize` cached-atom
input, §3c/§8 — no marker, no extract gating; its legacy composer
outputs are rejected by the `probe` gate interval and command check,
§3f). probe-rust needs no changes. Regeneration of pre-marker
extracts, merged artifacts and legacy projections is a checklist item
inside each issue, not a separate issue.

## 10. Explicitly out of scope / deferred — settled decisions

Each entry below is a deliberate decision with its reason, not an
oversight. Reviewers (including cross-model reviews): treat these as
agreed scope — re-raise one only with a *new* argument against the
recorded rationale, not by restating the concern.

- Review issue 4 (P10 whole-atom extension conflicts vs leanblueprint
  field-union) — separate decision; only the correspondence-record union
  carve-out (§5) is done here.
- `verified-by-translation` derived status — future ADR, which is also
  where staleness anchors on correspondence records (mapping-artifact
  revision) belong if ever needed. The interim `status-origin` rules
  (§2 — both values; non-promotion, promotion blocking, unconditional
  demotion) close the laundering paths now without implementing the
  derived status.
- Mechanical body-trust/closure-trust split for `trusted` atoms
  (separate traversal of spec/type dependencies) — fallback design only
  if the §3b producer audit surfaces a real counterexample.
- A separate view-composition operation for projected artifacts —
  rejection (§3d) is the chosen mechanism; revisit if a use case for
  merging projections appears.
- Content-equality conflict detection (review minor observation) —
  nice-to-have, not planned.
- Staleness diagnostics (§6, former PR 4) — deferred wholesale (sixth
  revision), including any `--force` gating; warning-only, no soundness
  content.
- Per-entry origin/timestamp metadata to support deep freshness checks
  through recursive merges — disproportionate to a diagnostic.
- Migration tooling to strip injected edges from old merged files —
  regeneration is the policy (§8). Same for legacy projections: they
  are rejected at the boundary (§3d) and regenerated if still needed,
  never repaired.
- §3e performance mitigations beyond single-pass staging (fast-path
  pre-scans, warning capping, mapping-record endpoint indexing, the
  `Vec` reverse index), incremental enrichment maintenance, dense node
  IDs / adjacency-vector rewrites — measured cost (§3e) does not
  justify them, and the fast-path conditions were a proven bug source
  (fourth review).
- Mirror repair at merge time (attaching `mapped-from` when a target
  arrives in a later plain merge) — the relation is the union of both
  fields (§5); regeneration restores mirrors.
- Boundary-evidence preservation for projections — rejection at the
  envelope boundary (§3d) is the chosen mechanism.
- A general producer-evidence framework — the two-value `status-origin`
  marker (§2) is the whole mechanism; richer evidence records belong to
  the deferred `verified-by-translation` ADR. Likewise the §3f gate is
  a fixed five-entry version-threshold table, not capability
  negotiation; a uniform schema-3.1 evidence gate was considered and
  rejected (§3f) because it forces releases and re-extraction on
  unaffected producers (probe-verus, probe-rust).
- Recursive gating of merged inputs (per-input `tool` in provenance) —
  unnecessary: the §3f table rejects pre-threshold composed artifacts
  at the outer envelope, and post-threshold composed files are built
  from validated inputs by construction (§3c/§3f).
- Mandating origin preservation on all status-rendering consumers — hub
  `probe summary` gets the contract (§6); scip-callgraph gets a
  recommendation (§8), decided in that repo. The §3f version gate
  likewise stops at the hub's own commands (merge, enrich, summary,
  project); gating external consumers is outside this plan's blast
  radius.
- Rejecting projections in `probe summary`/`probe project` — views with
  inherited labels are legitimate to *read* (§3d, P23); only
  recomputation paths (merge/enrich) refuse them. The two rejection
  predicates — projection marker and version gate — travel together
  only on recomputation boundaries (§3f).
- Projection BFS traversal of `maps-to`/`mapped-from` records —
  rejected (eighth review, §8): the reachability it would restore came
  from fabricated dependency edges; consumers that relied on it
  regenerate, and correspondence traversal would re-complicate
  projection for exactly that case.
- Provenance preservation beyond carrying the existing
  `Vec<InputProvenance>` through blueprint (§8) — one bounded fix in a
  producer already being changed; no general provenance-preservation
  framework for composers.
- Cross-repo test coupling — hub tests use synthetic replicas of the
  probe-lean aux-fold graphs (§3 test k), never the fixtures themselves.
- Adding probe-verus (or probe-rust) **extracts** to the §3f gate table
  — nothing to gate there: extracts carry no imported or
  graph-inexpressible evidence, and probe-verus's stale
  embedded-enrichment labels are repaired by every hub recomputation
  path (merge, enrich, projection's enrich-then-trim); the dependency
  bump (§8) is conformance, not authority (ninth review). The tenth
  review's Verus finding is different in kind — its `merge-atoms`
  *composer* masquerades as the hub — and is handled by the `probe`
  gate interval, the command rejection and the command's
  retirement/delegation (§3f/§8), still with no extract gating. The
  twelfth review's new argument — a projection routed through
  `--skip-atomize` yields a truncated extract no BFS can repair — is
  acknowledged and changes the *boundary*, not the gate: the
  cached-atom input runs the shared validator going forward (§3c/§8),
  but a gate entry would reject every existing Verus extract to catch
  an artifact class producible only by manually feeding a hub
  projection into a cache flag, with no known member; the §8
  regeneration policy covers any suspect extract.
- A general tool-identity verification framework (proving `tool.name`
  matches the producing binary) — the one identifiable masquerading
  shape is the Verus `merge-atoms` composer, closed by the `probe`
  version interval plus one command-string check and the command's
  retirement (§3f/§8, tenth review); no other producer emits under
  another tool's name. Re-emission under an *accurate* identity
  (probe-vcvio, eleventh review) is a different shape and is handled by
  the widened composer rule plus a gate entry (§3c/§3f), still without
  identity verification.
- Hub-native blueprint awareness beyond the summary filter —
  `probe project` reverse traversal can pull node atoms in via their
  `dependencies` (blueprint SCHEMA.md notes this), but that is
  presentation-layer inflation, not a laundering path; blueprint's own
  filter guidance covers generic consumers, and projection's selection
  semantics are settled above (tenth review). This includes keeping the
  enrichment boundary predicate blueprint-unaware (thirteenth review):
  a `language != "blueprint"` carve-out in `trusted_boundary` would
  protect nothing — blueprint emits no code→node dependencies, so a
  blueprint trusted node can shield only other presentation atoms, and
  the ADR-006 code-atom scoping (§3b) already states the semantics.
- Redesigning blueprint's aggregation ordering (`trusted` ranked above
  `verified`) — the ranking is blueprint's own design decision. The
  thirteenth review's new argument is accepted but resolved by scoping,
  not redesign: marker propagation alone cannot cover a marker-free
  contaminated `verified` contributor (its negative finding is
  graph-expressible, so no marker exists), and instead of a synthesis
  distinction the assurance contract — including the whole-closure
  trusted-boundary reading — is stated over code atoms; blueprint
  synthetic statuses are presentation aggregates (§3b), consistent with
  the summary exclusion (§6) and pinned by the §8 marker-free test.
- Post-union normalization / relitigating P6 first-wins — the
  thirteenth review's ordering finding is fixed by restating what the
  code already does (per-input normalization before conflict
  resolution, one final enrichment, §3d), not by changing conflict
  semantics: normalizing the combined map instead silently re-selects
  which evidence wins a dotted-alias collision, and argument order
  stays the user's freshness knob (§4 law 3).
- Dynamic or negotiated gate thresholds — the fix for
  compiled-into-composers validators is reserving the final release
  numbers upfront (§9), not runtime threshold configuration.
- Recomputing enrichment inside `probe summary` — summary reports the
  artifact's own labels behind the §3f gate; over a pre-bump Verus
  extract its ephemeral output may show a stale transitive label until
  regeneration, but unlike projection it freezes no artifact, so
  enrich-on-read is not worth the contract change (summary would stop
  describing the file).
