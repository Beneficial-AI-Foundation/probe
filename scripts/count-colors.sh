#!/usr/bin/env bash
# Count atoms per verification color, following docs/verification-statuses.md.
#
# Works with any Schema 2.0 atoms file: probe-verus/extract, probe-aeneas/extract,
# probe-lean/extract, probe-rust/extract, or a merged probe/merged-atoms file
# (which may mix pipelines). Coloring is per-atom, so a merged file is handled the
# same as a single-tool file.
#
# Atoms not shown in VeriLib are dropped before counting: external-crate stubs
# (code-path == "") and atoms flagged is-hidden / is-ignored / is-extraction-artifact.
#
# A browse-only project (no verification framework) carries no verification
# information at all — no statuses, specs, or translations on any shown atom. This
# is detected structurally (not from the schema string, so a merged file of only
# pure-Rust atoms is handled too): every shown atom is White, with no counts.
#
# Otherwise colors follow the two tables in docs/verification-statuses.md:
#
#   Implementations (kind: "exec") — does it *verify*?
#     Grey   unspecified impl (Verus is-disabled:true / Aeneas not translated)
#     Yellow Aeneas: translated but unspecified
#     Blue   specified, not yet proven
#     Light Green  specified + "verified"   Dark Green  specified + "transitively-verified"
#   `specified` is checked BEFORE status: an unspecified impl is Grey/Yellow even if
#   its verification-status happens to be "verified" — Green requires being specified.
#
#   Specifications & proofs (Lean atoms + Verus proof/spec) — is it *proved*?
#     White  no verification-status     Blue  "unverified"
#     Light Green  "verified"          Dark Green  "transitively-verified"
#
#   Both tables: "trusted" -> Purple and "failed" -> Red take precedence.
#
# Each atom is assigned exactly one color by a single if/elif chain, so the buckets
# partition the shown atoms by construction. The invariant that keeps "specified"
# (Blue/Green) disjoint from "unspecified" (Grey) is has-spec => not-disabled (P24).
# The subtotal checks below are a backstop against a color escaping the buckets;
# the two diagnostic warnings (dangling translation, unknown status) surface data
# that the color tables cannot represent faithfully.
# @kb: kb/engineering/properties.md#p24-a-specified-atom-is-in-analysis-scope
#
# Usage: scripts/count-colors.sh <input.json>

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <input.json>" >&2
    exit 1
fi

INPUT="$1"

if [ ! -f "$INPUT" ]; then
    echo "Error: file not found: $INPUT" >&2
    exit 1
fi

jq -r '
  (.schema // "") as $schema |
  .data as $d |

  # Shown atoms: drop external-crate stubs and hidden/ignored/artifact atoms.
  [ $d | to_entries[] | .value | select(
      (.["code-path"] // "") != "" and
      (.["is-hidden"] // false) != true and
      (.["is-ignored"] // false) != true and
      (.["is-extraction-artifact"] // false) != true
  ) ] as $shown |
  (($d | length) - ($shown | length)) as $dropped |

  # A verification-framework extract (probe-verus/-aeneas/-lean) always uses the
  # two tables — its spec-less execs are Grey, not White — even if nothing is
  # specified yet.
  ( $schema | (startswith("probe-verus") or startswith("probe-aeneas") or startswith("probe-lean")) ) as $is_framework |

  # Browse-only: no verification framework AND no shown atom carries any
  # verification information (a pure-Rust extract, or a merged file with none).
  ( ($is_framework | not)
    and ( ( $shown | any(
              (.["verification-status"] // null) != null
              or ((.["primary-spec"] // "") != "")
              or (((.["specs"] // []) | length) > 0)
              or ((.["translation-name"] // null) != null)
          ) ) | not )
  ) as $browse_only |

  if $browse_only then
    "Schema: \($schema)   (browse-only — no verification information)",
    "",
    "All \($shown | length) shown atoms are White; no verification colors apply.",
    "(dropped \($dropped) not-shown atoms)"
  else
    # Assign exactly one color per shown atom (see docs/verification-statuses.md).
    ( $shown | map(
        (.["verification-status"] // null) as $vs |
        (.kind == "exec") as $is_impl |
        # specified: a Verus inline primary-spec, or the chosen primary-spec on the
        # Aeneas translation. The translation is looked up in the full atom map (not
        # just $shown): a hidden/ignored translation still carries the spec that the
        # Rust function verifies against. `specs` (a generic dependency signal) is
        # intentionally NOT used — only a chosen primary-spec makes it specified.
        (
          ((.["primary-spec"] // "") != "")
          or
          ( (.["translation-name"] // null) as $tn |
            $tn != null and (($d[$tn]["primary-spec"] // "") != "") )
        ) as $specified |
        ((.["translation-name"] // null) != null) as $translated |
        {
          group: (if $is_impl then "impl" else "spec" end),
          color: (
            if   $vs == "trusted" then "purple"
            elif $vs == "failed"  then "red"
            elif $is_impl then
              # specified first: Green/Blue require a spec (doc Table 1).
              (if $specified then
                 (if   $vs == "transitively-verified" then "dark_green"
                  elif $vs == "verified"              then "light_green"
                  else "blue" end)
               elif $translated then "yellow"
               else "grey" end)
            else
              (if   $vs == "transitively-verified" then "dark_green"
               elif $vs == "verified"              then "light_green"
               elif $vs == "unverified"            then "blue"
               else "white" end)
            end
          )
        }
      )
    ) as $atoms |

    [ $atoms[] | select(.group == "impl") ] as $impl |
    [ $atoms[] | select(.group == "spec") ] as $spec |

    # Diagnostics: data the color tables cannot represent faithfully.
    ( [ $shown[] | select(
          (.["translation-name"] // null) as $tn | $tn != null and ($d[$tn] == null)
        ) ] | length ) as $dangling |
    ( [ $shown[] | select(
          (.["verification-status"] // null) as $v |
          $v != null and (($v | IN("verified","transitively-verified","unverified","failed","trusted")) | not)
        ) ] | length ) as $unknown_status |

    {
      pipeline: ($schema | if   startswith("probe-verus")  then "verus"
                           elif startswith("probe-aeneas") then "aeneas"
                           elif startswith("probe-lean")   then "lean"
                           elif startswith("probe/")       then "merged"
                           else "mixed" end),
      dropped:  $dropped,
      shown:    ($shown | length),

      impl_grey:   [$impl[] | select(.color == "grey")]        | length,
      impl_yellow: [$impl[] | select(.color == "yellow")]      | length,
      impl_blue:   [$impl[] | select(.color == "blue")]        | length,
      impl_lgreen: [$impl[] | select(.color == "light_green")] | length,
      impl_dgreen: [$impl[] | select(.color == "dark_green")]  | length,
      impl_purple: [$impl[] | select(.color == "purple")]      | length,
      impl_red:    [$impl[] | select(.color == "red")]         | length,
      impl_total:  ($impl | length),

      spec_white:  [$spec[] | select(.color == "white")]       | length,
      spec_blue:   [$spec[] | select(.color == "blue")]        | length,
      spec_lgreen: [$spec[] | select(.color == "light_green")] | length,
      spec_dgreen: [$spec[] | select(.color == "dark_green")]  | length,
      spec_purple: [$spec[] | select(.color == "purple")]      | length,
      spec_red:    [$spec[] | select(.color == "red")]         | length,
      spec_total:  ($spec | length)
    } |

    (.impl_grey + .impl_yellow + .impl_blue + .impl_lgreen + .impl_dgreen + .impl_purple + .impl_red) as $impl_sum |
    (.spec_white + .spec_blue + .spec_lgreen + .spec_dgreen + .spec_purple + .spec_red) as $spec_sum |

    "Pipeline: \(.pipeline)   (shown \(.shown), dropped \(.dropped) not-shown)",
    "",
    "Implementations — does it *verify*?  (kind: exec)",
    "# | Color       | Count",
    "--|-------------|------",
    "1 | Grey        | \(.impl_grey)",
    "2 | Yellow      | \(.impl_yellow)",
    "3 | Blue        | \(.impl_blue)",
    "4 | Light Green | \(.impl_lgreen)",
    "5 | Dark Green  | \(.impl_dgreen)",
    "6 | Purple      | \(.impl_purple)",
    "7 | Red         | \(.impl_red)",
    "--|-------------|------",
    "  | Subtotal    | \(.impl_total)",
    "",
    "Specifications & proofs — is it *proved*?  (Lean atoms + Verus proof/spec)",
    "# | Color       | Count",
    "--|-------------|------",
    "1 | White       | \(.spec_white)",
    "2 | Blue        | \(.spec_blue)",
    "3 | Light Green | \(.spec_lgreen)",
    "4 | Dark Green  | \(.spec_dgreen)",
    "5 | Purple      | \(.spec_purple)",
    "6 | Red         | \(.spec_red)",
    "--|-------------|------",
    "  | Subtotal    | \(.spec_total)",
    "",
    "Combined (universal color, scoped counts summed)",
    "Color       | Impl | Spec | Total",
    "------------|------|------|------",
    "Grey        | \(.impl_grey)   |  -   | \(.impl_grey)",
    "White       |  -   | \(.spec_white)   | \(.spec_white)",
    "Yellow      | \(.impl_yellow)   |  -   | \(.impl_yellow)",
    "Blue        | \(.impl_blue)   | \(.spec_blue)   | \(.impl_blue + .spec_blue)",
    "Light Green | \(.impl_lgreen)   | \(.spec_lgreen)   | \(.impl_lgreen + .spec_lgreen)",
    "Dark Green  | \(.impl_dgreen)   | \(.spec_dgreen)   | \(.impl_dgreen + .spec_dgreen)",
    "Purple      | \(.impl_purple)   | \(.spec_purple)   | \(.impl_purple + .spec_purple)",
    "Red         | \(.impl_red)   | \(.spec_red)   | \(.impl_red + .spec_red)",
    "------------|------|------|------",
    "  | Total shown | \(.impl_total + .spec_total)",
    (if $impl_sum != .impl_total then
      "  WARNING: implementation colors (\($impl_sum)) != impl subtotal (\(.impl_total))"
     else empty end),
    (if $spec_sum != .spec_total then
      "  WARNING: spec/proof colors (\($spec_sum)) != spec subtotal (\(.spec_total))"
     else empty end),
    (if (.impl_total + .spec_total) != .shown then
      "  WARNING: impl+spec (\(.impl_total + .spec_total)) != shown (\(.shown))"
     else empty end),
    (if $dangling > 0 then
      "  WARNING: \($dangling) atom(s) have a translation-name absent from the file (dangling reference)"
     else empty end),
    (if $unknown_status > 0 then
      "  WARNING: \($unknown_status) atom(s) have an unrecognized verification-status (treated as absent)"
     else empty end)
  end
' "$INPUT"
