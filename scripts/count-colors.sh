#!/usr/bin/env bash
# Count Rust project functions per verification color.
#
# Works with probe-aeneas/extract and probe-verus/extract JSON. Auto-detects
# the pipeline from the schema field.
#
# Scopes to project functions only (code-path != ""), excluding external crate
# stubs. Grey count includes test functions.
#
# Grey, White, Light Cyan, Dark Blue, and Purple form a partition of the total.
# Dark Blue is cumulative: all specified non-trusted functions (superset of Green).
# Sanity check: grey + white + cyan + blue + purple = total.
#
# See docs/verification-statuses.md for color definitions.
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
  .data as $d |
  ($schema | if startswith("probe-aeneas") then "aeneas"
             elif startswith("probe-verus") then "verus"
             else "unknown" end) as $pipeline |

  [.data | to_entries[] | select(
    .value.language == "rust" and .value["code-path"] != ""
  )] |

  # Precompute derived flags per atom
  map(. + {
    _disabled: (.value["is-disabled"] == true),
    _trusted:  ((.value["verification-status"] // null) == "trusted"),
    _has_translation: (.value["translation-name"] != null),
    _has_spec: (
      if $pipeline == "aeneas" then
        .value["translation-name"] as $tn |
        ($tn != null) and (($d[$tn]["primary-spec"] // null) != null)
      else
        ((.value["primary-spec"] // null) as $ps | $ps != null and $ps != "")
      end
    )
  }) |

  {
    pipeline:    $pipeline,
    grey:        [.[] | select(._disabled == true)] | length,
    white:       [.[] | select(._disabled == false and ._has_spec == false and
                                ._has_translation == false and ._trusted == false)] | length,
    light_cyan:  [.[] | select(._disabled == false and ._has_translation == true and
                                ._has_spec == false and ._trusted == false)] | length,
    light_blue:  0,
    dark_blue:   [.[] | select(._has_spec == true and ._trusted == false)] | length,
    light_green: [.[] | select(.value["verification-status"] == "verified" and
                                ._has_spec == true)] | length,
    dark_green:  [.[] | select(.value["verification-status"] == "transitively-verified" and
                                ._has_spec == true)] | length,
    purple:      [.[] | select(._disabled == false and ._trusted == true)] | length,
    total:       length
  } |

  (.grey + .white + .light_cyan + .dark_blue + .purple) as $cover |
  (.light_green + .dark_green) as $verified |
  "Pipeline: \(.pipeline)",
  "",
  "# | Color       | Count",
  "--|-------------|------",
  "1 | Grey        | \(.grey)",
  "2 | White       | \(.white)",
  "3 | Light Cyan  | \(.light_cyan)",
  "4 | Light Blue  | \(.light_blue)",
  "5 | Dark Blue   | \(.dark_blue)",
  "6 | Light Green | \(.light_green)",
  "7 | Dark Green  | \(.dark_green)",
  "- | Purple      | \(.purple)",
  "--|-------------|------",
  "  | Total       | \(.total)",
  (if $cover != .total then
    "  WARNING: grey+white+cyan+blue+purple (\($cover)) != total (\(.total))"
  else empty end),
  (if $verified > .dark_blue then
    "  WARNING: green (\($verified)) > dark_blue (\(.dark_blue))"
  else empty end)
' "$INPUT"
