#!/usr/bin/env bash
# Count atoms per verification color, following docs/verification-statuses.md.
#
# Works with any Schema 2.0 atoms file: probe-verus/extract, probe-aeneas/extract,
# probe-lean/extract, probe-rust/extract, or a merged probe/merged-atoms file
# (which may mix pipelines). Coloring is per-atom.
#
# Color = verification *status*; role (impl / spec / proof / definition) is a
# separate axis carried by `kind` (+ links) and reported as a scoped count group
# and per-language breakdown. VeriLib is expected to render color = status and
# shape = role, so this script also offers a per-atom mode (see --per-atom).
#
# Atoms not shown in VeriLib are dropped before counting: external-crate stubs
# (code-path == "") and atoms flagged is-hidden / is-ignored / is-extraction-artifact.
#
# Roles (see docs/verification-statuses.md):
#   Implementation  — Rust `exec`, and a Lean `def` standing for a Rust function:
#                     a translation-name target, a `def` with a *documented*
#                     primary-spec, or an Aeneas-generated `def` (rust-source).
#                     A Lean def is graded by its OWN documented primary-spec
#                     theorem (the same Lean spec that probe-aeneas propagates to
#                     the Rust exec), not by borrowing the exec's color; a def
#                     with no documented spec is Yellow. So a translation that
#                     merely compiles is NOT counted Green unless it is verified
#                     against a spec.
#                     A primary-spec link is *documented* when the spec theorem
#                     carries a spec attribute (primary_spec / progress / pspec /
#                     step) or follows the `<def>_spec` naming convention.
#                     probe-lean also emits primary-spec by sole-spec inference
#                     (the def is referenced by exactly one theorem); that signal
#                     is too loose to make the def an implementation, so it is
#                     ignored here and the def stays in the Definitions group.
#                     A generic Lean project carries no pairing evidence at all,
#                     so its Implementations group is empty by construction.
#   Spec definition — a Verus `spec fn` (kind: "spec"): a stated condition, no
#                     proof obligation of its own -> Blue.
#   Proof / theorem — Verus `proof fn`, Lean `theorem`: colored by proof status.
#                     A proof with no status is White (not matched to a
#                     verification result — extraction bug, see probe-verus#33).
#   Definition      — a non-implementation `def`/`abbrev`/`opaque`/..., or a type
#                     declaration (`structure`/`inductive`/`class`): White, unless
#                     a `def` body carries a sorry (-> Orange) or it is trusted.
#
# Colors (status): Grey unspecified impl; Yellow translated/generated-but-unspecified;
#   Orange "unverified" (a Verus assume() / Lean sorry); Light Green "verified";
#   Dark Green "transitively-verified"; Purple "trusted"; Red "failed"; White nothing
#   to grade; Blue a Verus spec definition. "trusted"->Purple and "failed"->Red take
#   precedence.
#
# A browse-only project (no verification framework, no verification information on
# any shown atom) is reported as all White, with no counts.
# @kb: kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope
# @kb: kb/engineering/properties.md#p25-a-graded-atom-is-in-analysis-scope
#
# The tables count each function once: a Lean def whose Rust exec is itself
# shown repeats the exec's verdict, so it is left out of the tables (reported
# as a footnote). A probe-lean-only extract has no exec atoms, so there the
# Lean stand-ins carry the implementation counts. Empty groups print no table.
#
# Usage: scripts/count-colors.sh <input.json> [--per-atom]
#   --per-atom  emit one JSON object per shown atom: {id, language, group, kind, color}
#               (for VeriLib node coloring) instead of the human tables. Emits
#               BOTH atoms of an exec/translation pair (VeriLib paints both
#               nodes); the table dedupe applies to tables mode only.

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <input.json> [--per-atom]" >&2
    exit 1
fi

INPUT="$1"
MODE="tables"
if [ "${2:-}" = "--per-atom" ]; then
    MODE="per-atom"
fi

if [ ! -f "$INPUT" ]; then
    echo "Error: file not found: $INPUT" >&2
    exit 1
fi

jq -r --arg mode "$MODE" '
  # ---- helpers -------------------------------------------------------------
  # A def-to-primary-spec link is documented when the spec theorem carries a
  # spec attribute or is named `<def>_spec`. Excludes probe-lean sole-spec
  # inference. Context (.) must be the full data map; $id is the def atom id.
  def documented_spec($id; $v):
    ($v["primary-spec"] // "") as $ps |
    $ps != "" and (
      $ps == ($id + "_spec")
      or ( ((.[$ps] // {})["attributes"] // [])
           | any(IN("primary_spec", "progress", "pspec", "step")) )
    );

  # Color from a documented primary-spec theorem: the atom is graded by the
  # status of the theorem that specifies it. $id/$v are the specified atom;
  # returns null when the atom has no documented spec. Context (.) = data map.
  def spec_status_color($id; $v):
    if documented_spec($id; $v) then
      ( (.[$v["primary-spec"]] // {})["verification-status"] // null ) as $ss |
      (if   $ss == "transitively-verified" then "dark_green"
       elif $ss == "verified"              then "light_green"
       elif $ss == "trusted"               then "purple"
       elif $ss == "failed"                then "red"
       else "orange" end)
    else null end;

  # Implementation color (verify axis) for a Rust exec value $v with id $id.
  # probe-aeneas propagates the Lean spec theorem''s verdict onto the exec''s
  # own verification-status, so the exec is colored by that status; $specified
  # (its own inline spec, or its translation''s documented spec) only decides
  # Green-eligibility. Context (.) must be the full data map.
  def impl_color($id; $v):
    ($v["verification-status"] // null) as $vs |
    (
      (($v["primary-spec"] // "") != "")
      or ( ($v["translation-name"] // null) as $tn |
           $tn != null and documented_spec($tn; (.[$tn] // {})) )
    ) as $specified |
    (($v["translation-name"] // null) != null) as $translated |
    if   $vs == "trusted" then "purple"
    elif $vs == "failed"  then "red"
    elif $specified then
      (if   $vs == "transitively-verified" then "dark_green"
       elif $vs == "verified"              then "light_green"
       else "orange" end)
    elif $translated then "yellow"
    else "grey" end;

  # Implementation color for a Lean def standing for a Rust function. It is
  # graded by its OWN documented primary-spec theorem — the same Lean spec that
  # probe-aeneas propagates to the Rust exec — not borrowed from the exec. A
  # def with no documented spec (translated or Aeneas-generated but unspecified)
  # is Yellow. Context (.) must be the full data map.
  def def_impl_color($id; $v):
    ($v["verification-status"] // null) as $vs |
    if   $vs == "trusted" then "purple"
    elif $vs == "failed"  then "red"
    else ( spec_status_color($id; $v) // "yellow" ) end;

  # Proof/theorem color (proved axis) for status $vs. The "white" fallback is
  # the documented "no status" row: the atom was never matched to a
  # verification result (extraction bug, probe-verus#33).
  def proof_color($vs):
    if   $vs == "trusted"               then "purple"
    elif $vs == "failed"                then "red"
    elif $vs == "unverified"            then "orange"
    elif $vs == "transitively-verified" then "dark_green"
    elif $vs == "verified"              then "light_green"
    else "white" end;

  # Definition/type-decl color for status $vs.
  def def_color($vs):
    if   $vs == "trusted"    then "purple"
    elif $vs == "failed"     then "red"
    elif $vs == "unverified" then "orange"
    else "white" end;
  # -------------------------------------------------------------------------

  (.schema // "") as $schema |
  .data as $d |

  [ $d | to_entries[] | select(
      (.value["code-path"] // "") != "" and
      (.value["is-hidden"] // false) != true and
      (.value["is-ignored"] // false) != true and
      (.value["is-extraction-artifact"] // false) != true
  ) ] as $shown |
  (($d | length) - ($shown | length)) as $dropped |

  ( $schema | (startswith("probe-verus") or startswith("probe-aeneas") or startswith("probe-lean")) ) as $is_framework |
  ( ($is_framework | not)
    and ( ( $shown | any(
              (.value["verification-status"] // null) != null
              or ((.value["primary-spec"] // "") != "")
              or (((.value["specs"] // []) | length) > 0)
              or ((.value["translation-name"] // null) != null)
              or ((.value["rust-source"] // null) != null)
          ) ) | not )
  ) as $browse_only |

  # Set of all translation targets (from ALL execs, shown or hidden): a Lean
  # def in this set is an implementation even if it lacks rust-source. Used for
  # group membership only — the color comes from the def''s own spec.
  ( reduce ($d | to_entries[] | select(.value.kind == "exec")) as $e ({};
      ($e.value["translation-name"] // null) as $tn |
      if $tn != null then .[$tn] = true else . end
    ) ) as $xlate_target |

  # Translation targets whose exec is itself SHOWN: their def stand-ins would
  # repeat a counted verdict, so the tables dedupe them (per-atom mode keeps
  # them). When the exec is hidden or absent, the def carries the count.
  ( reduce ($shown[] | select(.value.kind == "exec")) as $e ({};
      ($e.value["translation-name"] // null) as $tn |
      if $tn != null then .[$tn] = true else . end
    ) ) as $shown_exec_target |

  if $browse_only then
    "Schema: \($schema)   (browse-only — no verification information)",
    "",
    "All \($shown | length) shown atoms are White; no verification colors apply.",
    "(dropped \($dropped) not-shown atoms)"
  else
    # Classify every shown atom: {id, lang, kind, group, color}.
    # The if/elif/else is total, so the four groups partition the shown atoms
    # by construction.
    [ $shown[]
      | .key as $id | .value as $v |
        ($v.kind // "") as $k |
        ($v.language // "?") as $lang |
        ($v["verification-status"] // null) as $vs |
        ($d | documented_spec($id; $v)) as $specok |
        ( if   $k == "exec" then {group: "impl", color: ($d | impl_color($id; $v))}
          elif ($k == "def" and
                (($xlate_target[$id] == true)
                 or $specok
                 or (($v["rust-source"] // null) != null)))
            then {group: "impl", color: ($d | def_impl_color($id; $v))}
          elif $k == "spec" then {group: "spec", color: (if $vs == "failed" then "red" elif $vs == "trusted" then "purple" else "blue" end)}
          elif ($k == "proof" or $k == "theorem") then {group: "proof", color: proof_color($vs)}
          else {group: "def", color: def_color($vs)} end )
        + {id: $id, language: $lang, kind: $k}
      | . + {dup: (.group == "impl" and .kind == "def" and ($shown_exec_target[.id] == true))}
    ] as $atoms |

    if $mode == "per-atom" then
      $atoms[] | {id, language, group, kind, color} | tojson
    else
      ($schema | if   startswith("probe-verus")  then "verus"
                 elif startswith("probe-aeneas") then "aeneas"
                 elif startswith("probe-lean")   then "lean"
                 elif startswith("probe/")       then "merged"
                 else "mixed" end) as $pipeline |

      # Table atoms: each function counted once (dup stand-ins excluded).
      [ $atoms[] | select(.dup | not) ] as $tatoms |
      ([ $atoms[] | select(.dup) ] | length) as $deduped |

      # counters
      def cnt($g; $lang; $c): [ $tatoms[] | select(.group==$g and .language==$lang and .color==$c) ] | length;
      def langs($g): [ $tatoms[] | select(.group==$g) | .language ] | unique;
      def sub($g; $lang): [ $tatoms[] | select(.group==$g and .language==$lang) ] | length;
      def grp($g): [ $tatoms[] | select(.group==$g) ] | length;
      def tot($c): [ $tatoms[] | select(.color==$c) ] | length;

      # generic row renderer: pads columns loosely (markdown-ish)
      def row($g; $lang; $cols): ($lang + "  | " + ([ $cols[] as $c | (cnt($g;$lang;$c)|tostring) ] | join("  | ")) + "  | " + (sub($g;$lang)|tostring));

      ["grey","yellow","orange","light_green","dark_green","purple","red"] as $impl_cols |
      ["blue","purple","red"] as $spec_cols |
      ["orange","light_green","dark_green","purple","red","white"] as $proof_cols |
      ["white","orange","purple","red"] as $def_cols |

      "Pipeline: \($pipeline)   (shown \($shown|length), dropped \($dropped) not-shown)",
      "",
      ( if grp("impl") > 0 then
          "Implementations — does it *verify against its spec*?   (color = verify status)",
          "lang  | Grey | Yellow | Orange | LtGreen | DkGreen | Purple | Red | Subtotal",
          ( langs("impl")[] as $l | (sub("impl";$l) > 0 | if . then row("impl";$l;$impl_cols) else empty end) ),
          ""
        else empty end ),
      ( if grp("spec") > 0 then
          "Specifications (stated conditions) — Verus spec fn, not proved   (Blue)",
          "lang  | Blue | Purple | Red | Subtotal",
          ( langs("spec")[] as $l | (sub("spec";$l) > 0 | if . then row("spec";$l;$spec_cols) else empty end) ),
          ""
        else empty end ),
      ( if grp("proof") > 0 then
          "Proofs & theorem-specs — is it *proved*?",
          "lang  | Orange | LtGreen | DkGreen | Purple | Red | White | Subtotal",
          ( langs("proof")[] as $l | (sub("proof";$l) > 0 | if . then row("proof";$l;$proof_cols) else empty end) ),
          ""
        else empty end ),
      ( if grp("def") > 0 then
          "Definitions & type declarations   (White; Orange if a def has a sorry)",
          "lang  | White | Orange | Purple | Red | Subtotal",
          ( langs("def")[] as $l | (sub("def";$l) > 0 | if . then row("def";$l;$def_cols) else empty end) ),
          ""
        else empty end ),
      "Combined (universal color, all groups/languages)",
      "Color       | Count",
      "------------|------",
      ( ["grey","white","yellow","blue","orange","light_green","dark_green","purple","red"][] as $c
        | (tot($c) > 0 | if . then ($c + " | " + (tot($c)|tostring)) else empty end) ),
      "------------|------",
      "Total counted | \($tatoms|length)",
      (if $deduped > 0 then "(+ \($deduped) Lean stand-ins of counted execs, not double-counted; per-atom mode emits them)" else empty end),
      ( [ $shown[] | select(.value["translation-name"] != null and ($d[.value["translation-name"]] == null)) ] | length ) as $dangling_xlate |
      (if $dangling_xlate > 0 then "  WARNING: \($dangling_xlate) atom(s) have a translation-name absent from the file" else empty end),
      ( [ $shown[] | select(.value.kind == "def" and ((.value["primary-spec"] // "") != "") and ($d[.value["primary-spec"]] == null)) ] | length ) as $dangling_spec |
      (if $dangling_spec > 0 then "  WARNING: \($dangling_spec) def(s) have a primary-spec absent from the file" else empty end),
      ( [ $shown[] | .value["verification-status"] // null | select(. != null and ((. | IN("verified","transitively-verified","unverified","failed","trusted")) | not)) ] | length ) as $unknown |
      (if $unknown > 0 then "  WARNING: \($unknown) atom(s) have an unrecognized verification-status" else empty end),
      ( [ $shown[] | select(.value.kind == "proof" and ((.value["verification-status"] // null) == null)) ] | length ) as $graded_gap |
      (if $graded_gap > 0 then "  WARNING: \($graded_gap) proof atom(s) have no verification-status (probe-verus#33)" else empty end),
      ( [ $shown[] | select(
            ((.value["is-disabled"] // false) == true) and
            ( ((.value["primary-spec"] // "") != "")
              or ( (.value["translation-name"] // null) as $tn |
                   $tn != null and (($d[$tn]["primary-spec"] // "") != "") ) )
        ) ] | length ) as $p24 |
      (if $p24 > 0 then "  WARNING: \($p24) atom(s) violate P24 (disabled yet specified)" else empty end),
      ( [ $shown[] | select(
            ((.value["is-disabled"] // false) == true) and
            ((.value["verification-status"] // null) != null)
        ) ] | length ) as $p25 |
      (if $p25 > 0 then "  WARNING: \($p25) atom(s) violate P25 (disabled yet graded)" else empty end)
    end
  end
' "$INPUT"
