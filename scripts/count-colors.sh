#!/usr/bin/env bash
# Count Rust project functions per verification color in a probe-aeneas/extract JSON.
#
# Scopes to project functions only (code-path != ""), excluding external crate
# stubs. Grey count includes test functions.
#
# Grey, White, and Light Cyan are disjoint (unspecified functions).
# Dark Blue is cumulative: all functions with translation + specs (superset of
# Light Green, Dark Green, and Purple).
# Sanity check: grey + white + light_cyan + dark_blue = total.
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
  .data as $d |
  [.data | to_entries[] | select(
    .value.language == "rust" and .value["code-path"] != ""
  )] |
  {
    grey:        [.[] | select(.value["is-disabled"] == true)] | length,
    white:       [.[] | select(.value["is-disabled"] == false and
                                .value["translation-name"] == null)] | length,
    light_cyan:  [.[] | select(
                   .value["translation-name"] != null and
                   ($d[.value["translation-name"]]["primary-spec"] // null) == null and
                   (.value["verification-status"] // null) != "trusted"
                 )] | length,
    light_blue:  0,
    dark_blue:   [.[] | select(
                   .value["translation-name"] != null and
                   ($d[.value["translation-name"]]["primary-spec"] // null) != null
                 )] | length,
    light_green: [.[] | select(.value["verification-status"] == "verified")] | length,
    dark_green:  [.[] | select(.value["verification-status"] == "transitively-verified")] | length,
    purple:      [.[] | select(.value["verification-status"] == "trusted")] | length,
    total:       length
  } |
  (.grey + .white + .light_cyan + .dark_blue) as $cover |
  (.light_green + .dark_green + .purple) as $verified |
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
    "  WARNING: grey+white+cyan+blue (\($cover)) != total (\(.total))"
  else empty end),
  (if $verified != .dark_blue then
    "  WARNING: green+purple (\($verified)) != dark_blue (\(.dark_blue))"
  else empty end)
' "$INPUT"
