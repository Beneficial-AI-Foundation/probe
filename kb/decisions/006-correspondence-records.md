---
title: "ADR-006: Correspondence records and the verification evidence contract"
last-updated: 2026-09-28
status: accepted
---

# ADR-006: Correspondence records (maps-to) and the verification evidence contract

## Context

A soundness review of `probe merge` and enrichment (2026-09; working notes
in the hub repo's `merge-soundness-review.md` and
`merge-soundness-fix-plan.md`) found that:

1. `merge --mappings` injected cross-language edges into `dependencies`
   after enrichment ran, invalidating `transitively-verified` labels, and
   the upgrade-only `probe enrich` could not repair them.
2. Injected edges broke the dependency decomposition (P15) and conflated
   two relations with different verification semantics.
3. Plain merge (stub resolution) can invalidate labels on its own, and
   nothing recomputed them.
4. Verification evidence flows between tools in ways the graph cannot
   express: probe-aeneas copies Lean statuses onto Rust atoms; probe-lean's
   kernel taint walk sees taint the emitted graph does not contain.
   Re-enriching such artifacts launders contaminated or imported evidence
   into clean transitive labels.
5. The algebraic laws (P4/P5) were overclaimed, and several composers
   re-emit foreign evidence without validating its authority.

This ADR records the design that resolves these findings. It supersedes
[ADR-003](003-mappings-design.md)'s **application** semantics only —
mapping *generation* (probe-aeneas strategies, 1-to-1 generation,
1-to-many acceptance, bidirectional files) is untouched.

## Decision 1: Correspondence records, not dependency edges

A *dependency* (calls, or proof-uses) and a *correspondence*
(abstraction/translation: "there is a mappings-file entry linking these
names, with this confidence") are different relations. Mappings applied
during merge attach **correspondence records** — `maps-to` on the
mapping's `from` atom, `mapped-from` on the `to` atom, each record
carrying `target`, `confidence`, and optional `method` — and never touch
`dependencies`.

Motivating counterexample for the old edge injection: Rust `f` calls `g`;
a Lean development proves a property of `f`'s model using `ga`, a
hand-written abstraction of `g` containing a sorry. Injected edges
fabricate `f → ga` (false: `f` does not call `ga`) and give Lean atoms
depending on `ga` a dependency on Rust `g` (almost always `unverified`),
contaminating proven Lean theorems with the mere existence of unverified
Rust code. The correspondence `g ↔ ga` is the *trust boundary*, not a
dependency; the sorry in `ga` legitimately weakens only the Lean theorem,
and that contamination path is intra-Lean.

Consequences:

- Correspondence attaches to the mapped atom itself, so leaf functions get
  cross-language linkage (the old design attached edges only to callers).
- ADR-003's promise that consumers can filter by confidence is restored:
  records carry `confidence`/`method`; injected edge strings could not.
- Cross-language *resolution* becomes a derived view computed by consumers
  over the records at read time.
- Attachment is **unconditional, key-local, and set-like**
  ([P13](../engineering/properties.md#p13-correspondence-records-attach-unconditionally)):
  a dangling target warns, never skips. This is what makes the mapping
  compatibility law hold
  ([P4](../engineering/properties.md#p4-merge-associativity-on-the-carrier)).
- Records are **unioned through every equal-key conflict** and are inert
  to enrichment and projection
  ([P27](../engineering/properties.md#p27-correspondence-records-are-unioned-and-inert)).
  This is a narrow carve-out from whole-atom conflict semantics; general
  extension union (review issue 4) remains a separate, undecided question.
- Wire format and determinism rules:
  [schema.md § Correspondence records](../engineering/schema.md#correspondence-records-maps-to-mapped-from).
  Hub-side schema-version bump 3.0 → 3.1 (minor; no producer lockstep).

A derived cross-language status (e.g. `verified-by-translation` computed
over `maps-to` records with a confidence threshold) is **deferred** to a
future ADR. `maps-to` records never participate in contamination BFS.

## Decision 2: The `status-origin` marker

Status transfer already exists and cannot be deferred: probe-aeneas copies
a Lean primary-spec theorem's status onto the corresponding Rust atom, and
probe-lean's authoritative statuses come from a kernel-level taint walk
whose findings are not reconstructible from the emitted graph (maintained
fixtures pin `verified` atoms with empty dependency lists whose sorry sits
in non-emitted auxiliaries). Graph recomputation over either would launder
contaminated or graph-inexpressible evidence into clean transitive labels.

`status-origin` is an enumerated atom extension with exactly two values:

- `"translation"` — the status is **imported evidence**, copied from a
  corresponding atom in another language (probe-aeneas). Aeneas normalizes
  a copied `transitively-verified` to `verified` at copy time.
- `"kernel-taint"` — the status reflects a **producer-known negative
  finding the emitted graph cannot express** (probe-lean: kernel-reachable
  taint). Stamped whenever the taint pass runs, including under
  `--skip-enrich`, so an *unmarked* `verified` atom is safely promotable.

Enrichment treats every `status-origin`-bearing atom uniformly as a
**blocker seed** ([P23](../engineering/properties.md#p23-transitive-verification)):

- never promoted to `transitively-verified`;
- unconditionally demoted from an imported or stale
  `transitively-verified` (not only when contaminated);
- blocking promotion of every atom that reaches it along a non-trusted
  dependency path (the per-atom rule alone would let callers launder the
  same evidence one hop removed);
- deliberately **not** a trusted boundary — treating marked atoms as
  trusted would promote their callers and reopen the laundering path.
  This includes copied `trusted`: `trusted` + `status-origin` seeds like
  any other marked atom.

The two values differ only in *whose evidence* the status is — imported vs
local — which matters to consumers (Decision 9), not to enrichment. On
blueprint presentation aggregates (Decision 5), a propagated marker means
"the aggregate contains evidence of that class", not "this status was
copied verbatim" — the aggregate reading is part of the value definitions.

## Decision 3: Enrichment recomputes; merge re-enriches

Enrichment is a **recomputation**, not an upgrade pass: one reverse BFS
from one seed set (explicit `failed`/`unverified` atoms plus every
`status-origin`-bearing atom), then every `verified`/
`transitively-verified` atom has its label set fresh. The result is a
function of the final graph and base statuses only. The full executable
definition, including transparency of missing-status atoms and the trusted
boundary precedence, is normative in
[P23](../engineering/properties.md#p23-transitive-verification).

Plain merge alone invalidates labels (a status-less stub, transparent at
extract time, resolved by a real `unverified` atom), so the shared merge
implementation re-enriches the atoms category **once after all inputs are
combined**. All public entry points inherit this. A **raw staging
primitive** exists for multi-step pipelines (probe-aeneas mutates statuses
after merging and must enrich exactly once at the end): it defers
recomputation but applies the **same authority validation** as the public
entry points (Decisions 6–7) — raw means skip recomputation, not skip
validation. Its output is documented as carrying potentially stale derived
statuses.

**Carrier preparation.** In merge, normalization (P8) runs **per input,
before conflict resolution** — the ordering selects evidence (which atom
wins a post-normalization dotted-alias collision) and no later pass can
repair a mis-selected P6 winner. Unary recomputation boundaries
(`probe enrich`, `probe project`) apply `prepare = enrich ∘ normalize` to
their single authoritative input before recomputation, seed matching, or
trimming. Mapping endpoints and atom keys are normalized by the same rule
on both sides of every lookup. Formally:
`μ(A, B) = enrich(resolve_conflicts(normalize(A), normalize(B)))`.

**Cost** (measured 2026-09-22, release mode): ~10–12 ms on a real
1,547-atom/18,617-edge artifact; up to ~1.2 s on a synthetic 100k-atom/
1.2M-edge graph. Acceptable for a batch CLI; fast-path pre-scans and
indexing mitigations were dropped after one proved a laundering bug
source. Mitigation kept: enrich once per completed merge; stage multi-step
pipelines on the raw primitive.

## Decision 4: Whole-atom trust (with producer audit findings)

Revised P23 makes `trusted` a **trust boundary**, and the boundary covers
the atom's **entire dependency closure** (whole-atom trust): enrichment
traverses the unified `dependencies` set, and the body/spec/type split is
not available on plain atoms. A `trusted` atom is a boundary **only when
it carries no `status-origin`** (Decision 2).

This is a deliberate semantic strengthening — the emitted `trusted-reason`
values do not all self-evidently authorize closure-wide trust. The
producer audit (2026-09-28, all three status-emitting producers, verified
against code and shipped artifacts):

- **probe-lean**: `trusted` (reasons `axiom`, `externally_verified`,
  `external` — the last excluding proofs) is a *deliberate closure-wide
  cut*: the kernel taint walk blocks at trusted declarations, expanding
  neither type nor value; its docs state "a trusted declaration is a
  leaf", and its soundness spec defines `transitively-verified` as "no
  sorry reachable except through a trusted declaration". Whole-atom trust
  matches producer semantics exactly. **Known gap (statement position)**:
  an axiom whose *statement* is built from unverified project code
  (`axiom a : p` with `def p : Prop := sorry`) is vacuously trusted;
  probe-lean detects only the literal one-hop case, as a warning. In every
  shipped artifact examined, trusted atoms are dependency leaves, so no
  real instance exists. Remediation is producer-side (extend the
  type-taint check to a type-closure walk); a resulting demotion is
  graph-inexpressible under whole-atom trust and flows to the hub via the
  existing `kernel-taint` marker — no hub mechanism change needed.
- **probe-verus**: `trusted` (`admit`, `external-body`,
  `assume-specification`) asserts a purely **local** claim — "the
  obligation attached to this atom was discharged by assumption rather
  than by proof"; the assignment reads no other atom, and trusted atoms
  retain their full dependency arrays. The pre-ADR implementation already
  behaved as a closure-wide barrier, but *accidentally* (a side effect of
  the verified-set guard), undocumented and untested. Adopting whole-atom
  trust makes the barrier an explicit rule. The realizable hidden-failure
  positions are **body-position** dependencies (a failed lemma called
  inside an `external_body` or admitted body), which the producer's
  assertion covers: assuming a spec of an unchecked body assumes the body
  wholesale, calls included. The **spec-position** counterexample (a
  failed predicate in `requires-dependencies`) is currently unrealizable
  in emitted data: Verus spec functions are structurally status-less, and
  the audit found `requires-dependencies`/`ensures-dependencies` empty in
  every real artifact (a probe-verus categorization defect, filed in its
  rollout issue).
- **probe-leanblueprint**: synthesizes statuses **only** on
  `language: "blueprint"` node atoms (hard guard), never on code atoms;
  its ladder is failure-first and trust-sticky (`failed` > `unverified`/
  unknown/missing > `trusted` > `verified` > `transitively-verified`), so
  `trusted` caps a mixed trusted+verified binding. Two of its three
  trusted sources are *claims, not attestations* (`declared` — a human
  `\leanok`; `upstream-proved` — the renderer's word about a dependency
  not in the extract). Its node atoms carry node→code dependencies only;
  no code atom depends on a node atom.

**Resolution**: whole-atom trust is adopted for **code atoms**. The
statement/spec-position hole is a shared producer-level limitation and
remains **open** until producers ship detection. The `kernel-taint` marker
gives it an expression channel — a statement-taint demotion is
graph-inexpressible under whole-atom trust and flows to the hub as a
blocker seed — but marker *capability* is not statement-taint *detection*:
probe-lean's walk stops at trusted declarations today, and its contract
release in the Decision 7 gate table guarantees only the marker.
Statement-taint detection (a type-closure check, with direct and multi-hop
regression fixtures and a caller pinned non-transitive) is tracked in the
probe-lean rollout issue; until it lands, this ADR records the shape as a
known assurance limitation, not as remediated.

The fallback trigger is deliberately narrower than the plan's original
wording ("if the audit surfaces a real counterexample"): the mechanical
body-trust/closure-trust split (separate traversal of spec/type
dependencies) is the **documented fallback**, to be built only if a real,
artifact-realizable case emerges **that markers cannot express**. The
audit's counterexample is marker-expressible, which is why
detection-plus-marker — not the split — is the chosen remediation.

## Decision 5: The assurance contract is stated over code atoms

The P23 assurance contract — including the whole-closure trusted-boundary
reading — quantifies over **code atoms**. Synthetic
`language: "blueprint"` statuses are **presentation aggregates**, never
trust-boundary assertions. This is a scoping of the claim, not a
laundering fix: blueprint emits no code→node dependencies, so a blueprint
aggregate can shield only other presentation atoms, and `probe summary`
excludes blueprint-language atoms from every partition (Decision 9). The
containment is **structural per blueprint's emitted graph shape** (its own
SCHEMA.md documents the no-code→node property, with the same escape
clause) and is not re-validated for arbitrary merged input; an input that
fabricates trusted atoms defeats status semantics regardless of
`language`, which is why the enrichment boundary predicate stays
blueprint-unaware.

A marker-free counterexample makes scoping (not marker propagation) the
necessary resolution: a graph-contaminated `verified` theorem carries no
marker (its negative finding is graph-expressible), so a node binding it
together with a trusted axiom synthesizes an unmarked `trusted` no marker
propagation could qualify.

**Blueprint synthesis rule** (its rollout): `derive_synthetic_verification`
propagates contributors' `status-origin` markers — a binding over marked
evidence never synthesizes an unmarked `trusted`/`verified`. Marker
precedence on the synthetic status: a node carries a marker iff any
contributing decl carries one; when contributors carry different values,
use `"translation"` if any contributor has `"translation"`, otherwise
`"kernel-taint"`. This is a deliberately **lossy presentation convention**,
not an assurance ordering — the two values are different dimensions
(origin vs. known hidden taint), enrichment blocks identically on either,
and blueprint atoms appear in no summary partition, so the only
information lost is that kernel taint *also* contributed to a
translation-marked aggregate (see the aggregate reading in Decision 2).
The marker-free case above is legal precisely because blueprint statuses
sit outside the code-atom assurance contract, and is pinned by test in the
blueprint rollout.

## Decision 6: Projections are views — rejected at recomputation boundaries

`transitively-verified` describes the **original dependency graph**; a
projection is a *view*, and deleting edges must not improve assurance.
Enriching a projection upgrades labels (`f [verified] → bad [unverified]`
projected at depth 0 leaves `f` with no deps).

- `probe project` writes the distinct schema **`probe/projected-atoms`**
  (the `projection` envelope field stays as metadata).
- `probe merge` and `probe enrich` **error** on projected inputs.
  Rejection, not warn-and-skip: skipping is not a defined operation (it
  preserves exactly the stale labels re-enrichment exists to repair).
- **Legacy detection**: every pre-ADR projection carries
  `schema: "probe/merged-atoms"` plus a `projection` field, so the
  rejection predicate is *new schema string OR presence of a `projection`
  field*, applied at the envelope boundary before metadata is discarded.
- Authority checking lives at the **envelope boundary** (`cmd_merge`,
  `merge_atom_files`, `cmd_enrich`, the raw staging primitive); the
  bare-map library API is documented as unable to check authority.
- On an authoritative input, `probe project` **recomputes enrichment on
  the full input graph before trimming** — otherwise a stale embedded
  producer label is frozen into a view this very rejection rule makes
  unrepairable. Labels are never recomputed from the trimmed view;
  already-projected inputs stay readable with labels untouched.
- Projections remain **readable** by `probe summary` and `probe project`
  (views with inherited labels are legitimate to read); only recomputation
  paths refuse them.
- Projection BFS stays **dependency-only**: it does not traverse
  correspondence records (the reachability the old fabricated edges
  provided is not reproduced; regeneration covers affected consumers).

## Decision 7: Per-producer version gate

Merge and enrich **error** on any atoms envelope whose `tool.name` is in
the following table with `tool.version` below the producer's contract
threshold. `probe summary` and `probe project` run the version-gate
component at their load boundaries too (an unmarked pre-contract
`verified` is indistinguishable from local evidence, and project re-stamps
its output at the current hub version, concealing origins). The
projection-rejection predicate does not travel with the gate to read-only
consumers.

**Reserved contract-release thresholds** (constants, compiled into the
shared validator; each producer's contract release must ship with exactly
its reserved version):

| `tool.name` | Threshold | Contract release content |
|---|---|---|
| `probe-lean` | **0.16.0** | `kernel-taint` marker |
| `probe-aeneas` | **0.21.0** | `translation` marker + raw staging + authoritative `Vec<Mapping>` |
| `probe-leanblueprint` | **0.11.0** | authority validation + marker propagation + composed-`inputs` emit |
| `probe-vcvio` | **0.2.0** | input version gate before re-emission |
| `probe` | **0.5.0 ≤ v < 1.0.0**, and reject `tool.command: "merge-atoms"` at any version | this ADR's hub release |

The `probe` entry is an **interval, not a floor**: probe-verus's legacy
`merge-atoms` composer hard-codes `tool.name: "probe"` while stamping
probe-verus's own package version (currently 8.x), so pre-contract output
would appear newer than any hub 0.x threshold. The command check catches
hypothetical 0.x-era outputs of that composer (`merge-atoms` is a command
the hub never shipped). The 1.0.0 ceiling is a reserved constant; crossing
it is a coordinated major release that re-bumps composers anyway.

Tool names not in the table pass at any version: probe-verus and
probe-rust **extracts** carry no imported or graph-inexpressible evidence,
so there is nothing for re-enrichment to launder (probe-verus's stale
embedded-enrichment labels are repaired by every hub recomputation path;
its dependency bump is conformance, not authority). There is **no
schema-version bump in any producer** — the gate keys on
`tool.name`/`tool.version`, which every envelope already carries; a
uniform "reject below schema 3.1" gate was rejected because it forces
releases and re-extraction on unaffected producers.

The gate does **not** recurse into merged inputs (`InputProvenance`
carries only `schema` + `source`). The table supplies the induction base
case instead: fresh extracts are gated directly, pre-threshold composed
artifacts are rejected at the outer envelope, and a post-threshold
composed artifact was produced by a tool that itself validates authority
(Decision 8), so "composer output is built only from contract-compliant
inputs" holds by induction with no provenance change.

Two scope statements make the induction honest. (a) It quantifies over the
**audited producer population**: unknown `tool.name`s pass at any version
by design — an openness trade-off, not an oversight — and a new composer
joins the table via its own rollout under Decision 8's obligation. (b) One
legacy exception is **accepted rather than gated**: a pre-fix probe-verus
`--skip-atomize` run can consume a projection (whose provenance it erases)
and emit an ordinary, ungated `probe-verus/extract` with unrepairably
truncated dependencies. No artifact of this class is known to exist, a
gate entry would reject every ordinary Verus extract to catch it, and the
Decision 10 regeneration policy covers any suspect file — so the guarantee
is stated net of this documented exception, not unconditionally.

## Decision 8: The composer rule

**Any producer that (i) runs enrichment over foreign atoms, (ii) re-emits
foreign verification evidence under its own tool identity, or
(iii) constructs authoritative verification output over a
previously-emitted dependency graph, must validate that input's authority
first** — the same shared validator (projection rejection in both formats
+ version gate) at every envelope boundary.

Current instances, one per shape: (i) probe-leanblueprint (enriches a
foreign atom base with the hub's bare-map API and re-emits under its own
identity); (ii) probe-vcvio (re-emits probe-lean extracts, statuses
included, under `probe-vcvio/extract` — re-stamping conceals the
evidence's origin from a gate keyed on the original producer's name);
(iii) probe-verus `--skip-atomize` (combines a cached, unvalidated atom
graph with fresh verification results — a projection fed through the cache
flag yields an unrepairable truncated extract). All three are fixed in
their rollouts; probe-verus's `merge-atoms` is retired or reimplemented as
a caller of the hub's validating merge, stamping its own accurate tool
identity.

## Decision 9: Consumer contracts (`probe summary`)

Summary must not present imported evidence as a locally established
result:

- New fourth list **`imported-verified`** (names only, deterministic
  order). Membership: `status-origin == "translation"` **AND** status ∈
  {`verified`, `transitively-verified`}. The conjunction is required
  because the marker rides on every copied status (Aeneas also copies
  `trusted`/`failed`); a translated non-verified status appears in **no**
  verified list.
- `status-origin: "kernel-taint"` atoms stay in the ordinary partitions —
  that evidence is local; the marker only makes it non-promotable.
- **Blueprint exclusion**: verification partitions are computed over code
  atoms — `language: "blueprint"` atoms appear in no verified list and are
  excluded from the `depended_upon` set used for entrypoint
  classification, so presentation-layer binding edges cannot alter code
  entrypoints (one checked theorem must report as one verified lemma, not
  two).
- The contract is enforceable only behind the version gate (Decision 7),
  which is why summary runs it.

Provenance is defined as a **deduplicated source inventory** with
**structural** composed-shape detection
([P9](../engineering/properties.md#p9-provenance-is-preserved)) — the old
`probe/merged-` prefix check silently discarded the Rust+Lean inventory of
`probe-aeneas/extract` envelopes.

## Decision 10: Migration and regeneration policy

Pre-contract artifacts are **regenerated, never repaired in place**:

- Merged files are derived artifacts — regenerate from original extracts
  and mappings after upgrading. "Original extracts" is qualified:
  pre-marker producer extracts are themselves affected artifacts and are
  **re-extracted** with the marker-capable releases.
- The version gate makes the prerequisite checkable at the file boundary
  instead of relying on release ordering: pre-threshold extracts, hub
  merges, Aeneas extracts, blueprint and vcvio outputs are rejected
  outright.
- Legacy projections are rejected at the boundary and regenerated if still
  needed; no edge-stripping or repair tooling.
- **Mapping corrections and withdrawals** follow the same policy:
  attachment is set-union and never removes a record, so a corrected or
  retracted mapping takes effect only by regenerating from extracts + the
  corrected mappings file.
- Hub PRs may land before producer releases; the gate enforces the
  ordering mechanically (a reserved threshold rejects every existing
  artifact until the producer ships under it). Thresholds are compiled in
  before the gate lands — composers link the validator through their
  pinned hub dependency, so a later hub-side lowering would strand them.

## Deferred (with rationale)

- **`verified-by-translation` derived status** — future ADR; also where
  staleness anchors on correspondence records belong. The interim
  `status-origin` rules close the laundering paths without it.
- **General extension union through conflicts** (review issue 4) —
  separate decision; only the correspondence-record union is carved out.
- **Body-trust/closure-trust split** — fallback design only (Decision 4).
- **Staleness diagnostics** — warning-only, no soundness content; design
  recorded in the fix plan.
- **Recursive gating of merged inputs / provenance `tool` entries** —
  unnecessary given the induction in Decision 7.
- **Tool-identity verification framework** — the one masquerading shape
  (Verus `merge-atoms`) is closed by the interval + command check + its
  retirement.

## Consequences

- Merged output labels are always enrichment-consistent; the algebra
  ([P4](../engineering/properties.md#p4-merge-associativity-on-the-carrier)/[P5](../engineering/properties.md#p5-merge-identity-exact-on-the-carrier))
  holds on the carrier of normalized, enrichment-consistent maps.
- `dependencies` is never modified by mappings; cross-language linkage is
  data (`maps-to`/`mapped-from`), styled and filtered by consumers.
- Consumers of the old injected edges (probegraph UI) must migrate to
  reading correspondence records.
- Every existing merged artifact, pre-marker producer extract, and legacy
  projection is rejected by the gate and must be regenerated.
- Downstream rollout (one issue per repo): probe-lean, probe-aeneas,
  probe-leanblueprint, probe-vcvio, probe-verus, probegraph UI.
  probe-rust needs no changes.
