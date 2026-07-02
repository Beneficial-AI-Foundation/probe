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
#   Implementation  — Rust `exec`, and an Aeneas Lean translation `def` (a `def`
#                     that is some exec's translation-name target). A translation
#                     inherits its Rust function's verify-status, so a translation
#                     that merely compiles is NOT counted Green unless the function
#                     it implements is verified against a spec.
#   Spec definition — a Verus `spec fn` (kind: "spec"): a stated condition, no
#                     proof obligation of its own -> Blue.
#   Proof / theorem — Verus `proof fn`, Lean `theorem`: colored by proof status.
#   Definition      — a non-translation `def`/`abbrev`/`opaque`/..., or a type
#                     declaration (`structure`/`inductive`/`class`): White, unless
#                     a `def` body carries a sorry (-> Orange) or it is trusted.
#
# Colors (status): Grey unspecified impl; Yellow Aeneas translated-but-unspecified;
#   Orange "unverified" (a Verus assume() / Lean sorry); Light Green "verified";
#   Dark Green "transitively-verified"; Purple "trusted"; Red "failed"; White nothing
#   to grade; Blue a Verus spec definition. "trusted"->Purple and "failed"->Red take
#   precedence.
#
# A browse-only project (no verification framework, no verification information on
# any shown atom) is reported as all White, with no counts.
# @kb: kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope
#
# Usage: scripts/count-colors.sh <input.json> [--per-atom]
#   --per-atom  emit one JSON object per shown atom: {id, language, group, kind, color}
#               (for VeriLib node coloring) instead of the human tables.

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
  # Implementation color (verify axis) for an exec-like value $v.
  def impl_color($v):
    ($v["verification-status"] // null) as $vs |
    (
      (($v["primary-spec"] // "") != "")
      or ( ($v["translation-name"] // null) as $tn |
           $tn != null and ((.[$tn]["primary-spec"] // "") != "") )
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

  # Proof/theorem color (proved axis) for status $vs.
  # The final "white" is a fallback for a proof/theorem with no verification-status
  # (rare, e.g. verification skipped); the Proofs table in the doc lists only the
  # graded rows. It keeps the partition total correct if such an atom ever appears.
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
          ) ) | not )
  ) as $browse_only |

  # Map: translation-target id -> the impl color of the exec that owns it.
  # Lets a Lean translation def inherit its Rust function verify-status.
  ( reduce ($shown[] | select(.value.kind == "exec")) as $e ({};
      ($e.value["translation-name"] // null) as $tn |
      if $tn != null then .[$tn] = ($d | impl_color($e.value)) else . end
    ) ) as $xlate |

  if $browse_only then
    "Schema: \($schema)   (browse-only — no verification information)",
    "",
    "All \($shown | length) shown atoms are White; no verification colors apply.",
    "(dropped \($dropped) not-shown atoms)"
  else
    # Classify every shown atom: {id, lang, kind, group, color}.
    [ $shown[]
      | .key as $id | .value as $v |
        ($v.kind // "") as $k |
        ($v.language // "?") as $lang |
        ($v["verification-status"] // null) as $vs |
        ($xlate[$id] // null) as $inherited |
        ( if   $k == "exec" then {group: "impl", color: ($d | impl_color($v))}
          elif ($k == "def" and $inherited != null) then {group: "impl", color: $inherited}
          elif $k == "spec" then {group: "spec", color: (if $vs == "failed" then "red" elif $vs == "trusted" then "purple" else "blue" end)}
          elif ($k == "proof" or $k == "theorem") then {group: "proof", color: proof_color($vs)}
          else {group: "def", color: def_color($vs)} end )
        + {id: $id, language: $lang, kind: $k}
    ] as $atoms |

    if $mode == "per-atom" then
      $atoms[] | {id, language, group, kind, color} | tojson
    else
      ($schema | if   startswith("probe-verus")  then "verus"
                 elif startswith("probe-aeneas") then "aeneas"
                 elif startswith("probe-lean")   then "lean"
                 elif startswith("probe/")       then "merged"
                 else "mixed" end) as $pipeline |

      # counters
      def cnt($g; $lang; $c): [ $atoms[] | select(.group==$g and .language==$lang and .color==$c) ] | length;
      def langs($g): [ $atoms[] | select(.group==$g) | .language ] | unique;
      def sub($g; $lang): [ $atoms[] | select(.group==$g and .language==$lang) ] | length;
      def tot($c): [ $atoms[] | select(.color==$c) ] | length;

      # generic row renderer: pads columns loosely (markdown-ish)
      def row($g; $lang; $cols): ($lang + "  | " + ([ $cols[] as $c | (cnt($g;$lang;$c)|tostring) ] | join("  | ")) + "  | " + (sub($g;$lang)|tostring));

      ["grey","yellow","orange","light_green","dark_green","purple","red"] as $impl_cols |
      ["blue","purple","red"] as $spec_cols |
      ["orange","light_green","dark_green","purple","red","white"] as $proof_cols |
      ["white","orange","purple","red"] as $def_cols |

      "Pipeline: \($pipeline)   (shown \($shown|length), dropped \($dropped) not-shown)",
      "",
      "Implementations — does it *verify against its spec*?   (color = verify status)",
      "lang  | Grey | Yellow | Orange | LtGreen | DkGreen | Purple | Red | Subtotal",
      ( langs("impl")[] as $l | (sub("impl";$l) > 0 | if . then row("impl";$l;$impl_cols) else empty end) ),
      "",
      "Specifications (stated conditions) — Verus spec fn, not proved   (Blue)",
      "lang  | Blue | Purple | Red | Subtotal",
      ( langs("spec")[] as $l | (sub("spec";$l) > 0 | if . then row("spec";$l;$spec_cols) else empty end) ),
      "",
      "Proofs & theorem-specs — is it *proved*?",
      "lang  | Orange | LtGreen | DkGreen | Purple | Red | White | Subtotal",
      ( langs("proof")[] as $l | (sub("proof";$l) > 0 | if . then row("proof";$l;$proof_cols) else empty end) ),
      "",
      "Definitions & type declarations   (White; Orange if a def has a sorry)",
      "lang  | White | Orange | Purple | Red | Subtotal",
      ( langs("def")[] as $l | (sub("def";$l) > 0 | if . then row("def";$l;$def_cols) else empty end) ),
      "",
      "Combined (universal color, all groups/languages)",
      "Color       | Count",
      "------------|------",
      ( ["grey","white","yellow","blue","orange","light_green","dark_green","purple","red"][] as $c
        | (tot($c) > 0 | if . then ($c + " | " + (tot($c)|tostring)) else empty end) ),
      "------------|------",
      "Total shown | \($atoms|length)",
      ( ([$atoms[]|select(.group=="impl")]|length)  as $impl_n |
        ([$atoms[]|select(.group=="spec")]|length)  as $spec_n |
        ([$atoms[]|select(.group=="proof")]|length) as $proof_n |
        ([$atoms[]|select(.group=="def")]|length)   as $def_n |
        if ($impl_n + $spec_n + $proof_n + $def_n) != ($shown|length) then
          "  WARNING: impl+spec+proof+def (\($impl_n+$spec_n+$proof_n+$def_n)) != shown (\($shown|length))"
        else empty end ),
      ( [ $shown[] | select(.value["translation-name"] != null and ($d[.value["translation-name"]] == null)) ] | length ) as $dangling |
      (if $dangling > 0 then "  WARNING: \($dangling) atom(s) have a translation-name absent from the file" else empty end),
      ( [ $shown[] | .value["verification-status"] // null | select(. != null and ((. | IN("verified","transitively-verified","unverified","failed","trusted")) | not)) ] | length ) as $unknown |
      (if $unknown > 0 then "  WARNING: \($unknown) atom(s) have an unrecognized verification-status" else empty end)
    end
  end
' "$INPUT"
