#!/usr/bin/env bash
# Count atoms per colour, split by the two visual channels of the scheme.
#
# Works with probe-aeneas/extract, probe-verus/extract, and probe-lean/extract
# JSON. Auto-detects the pipeline from the schema field.
#
# See docs/atoms_roles_statuses.md for the scheme. Two channels:
#
#   Colour BAR  — Rust `exec` atoms (language "rust", kind "exec").
#     Verification status: does the implementation meet its spec?
#     Pure function of (is-disabled, verification-status):
#       Grey        is-disabled: true              (out of verification scope)
#       White       no verification-status         (tracked, no spec yet)
#       Red         "failed"
#       Yellow      "unverified"                   (sorry / assume)
#       Light Green "verified"
#       Dark Green  "transitively-verified"
#       Purple      "trusted"                      (Rust atoms only)
#     These seven partition the exec total (relies on P24: status => not-disabled).
#
#   Colour DOT  — verification artifacts: Verus "spec"/"proof" (language
#     "verus" per KB P20) and every Lean atom (language "lean"). Selected by
#     kind (spec/proof) or language (lean).
#     Checking status: does the tool (lake build / cargo verus verify) accept it?
#     Pure function of verification-status:
#       Red    "failed"        (does not check)
#       Yellow "unverified"    (checks with a sorry / assume warning)
#       Green  otherwise       (verified / transitively-verified / trusted /
#                               none — accepted by the tool)
#
# External-crate stubs (code-path == "") are excluded from both channels.
# @kb: kb/engineering/properties.md#p24-a-status-bearing-atom-is-in-analysis-scope
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
  .schema as $schema |
  ($schema | if startswith("probe-aeneas") then "aeneas"
             elif startswith("probe-verus") then "verus"
             elif startswith("probe-lean")  then "lean"
             else "unknown" end) as $pipeline |

  [.data[] | select(.["code-path"] != "")] as $atoms |

  # --- Colour BAR: Rust exec atoms -------------------------------------------
  ([$atoms[] | select(.language == "rust" and .kind == "exec") | {
     disabled: (.["is-disabled"] == true),
     status:   (.["verification-status"] // null)
   }]) as $exec |

  # --- Colour DOT: verification artifacts ------------------------------------
  # Verus spec/proof and every Lean atom. Keyed on kind, not language: per KB
  # P20 a Verus spec/proof has language "verus" (only exec is "rust"), so we
  # select spec/proof by kind and thus stay correct regardless of that tag.
  ([$atoms[] | select(
     (.kind == "spec" or .kind == "proof") or
     (.language == "lean")
   ) | (.["verification-status"] // null)]) as $art |

  {
    pipeline:    $pipeline,

    grey:        [$exec[] | select(.disabled)] | length,
    white:       [$exec[] | select(.disabled | not) | select(.status == null)] | length,
    red:         [$exec[] | select(.disabled | not) | select(.status == "failed")] | length,
    yellow:      [$exec[] | select(.disabled | not) | select(.status == "unverified")] | length,
    light_green: [$exec[] | select(.disabled | not) | select(.status == "verified")] | length,
    dark_green:  [$exec[] | select(.disabled | not) | select(.status == "transitively-verified")] | length,
    purple:      [$exec[] | select(.disabled | not) | select(.status == "trusted")] | length,
    exec_total:  ($exec | length),

    dot_red:     [$art[] | select(. == "failed")] | length,
    dot_yellow:  [$art[] | select(. == "unverified")] | length,
    dot_green:   [$art[] | select(. != "failed" and . != "unverified")] | length,
    art_total:   ($art | length)
  } |

  (.grey + .white + .red + .yellow + .light_green + .dark_green + .purple) as $bar_cover |
  (.dot_red + .dot_yellow + .dot_green) as $dot_cover |
  (.exec_total - .grey) as $tracked |
  (.light_green + .dark_green) as $verified |
  (.light_green + .dark_green + .purple) as $verified_trusted |

  "Pipeline: \(.pipeline)",
  "",
  "Colour BAR — Rust exec atoms (verification status)",
  "# | Color       | Count",
  "--|-------------|------",
  "1 | Grey        | \(.grey)",
  "2 | White       | \(.white)",
  "3 | Red         | \(.red)",
  "4 | Yellow      | \(.yellow)",
  "5 | Light Green | \(.light_green)",
  "6 | Dark Green  | \(.dark_green)",
  "7 | Purple      | \(.purple)",
  "--|-------------|------",
  "  | Total       | \(.exec_total)",
  "",
  "  Tracked  (total - grey):            \($tracked)",
  "  Verified (light + dark green):      \($verified)",
  "  Verified + trusted (+ purple):      \($verified_trusted)",
  (if $tracked > 0 then
    "  (Verified + trusted) / tracked:     \($verified_trusted) / \($tracked)"
  else empty end),
  "",
  "Colour DOT — verification artifacts (checking status)",
  "# | Color  | Count",
  "--|--------|------",
  "1 | Red    | \(.dot_red)",
  "2 | Yellow | \(.dot_yellow)",
  "3 | Green  | \(.dot_green)",
  "--|--------|------",
  "  | Total  | \(.art_total)",
  (if $bar_cover != .exec_total then
    "  WARNING: bar colours (\($bar_cover)) != exec total (\(.exec_total))"
  else empty end),
  (if $dot_cover != .art_total then
    "  WARNING: dot colours (\($dot_cover)) != artifact total (\(.art_total))"
  else empty end)
' "$INPUT"
