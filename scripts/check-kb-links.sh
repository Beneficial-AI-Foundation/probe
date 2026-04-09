#!/usr/bin/env bash
# Validates cross-references between markdown files in kb/.
# Checks: (1) relative file links resolve, (2) heading anchors exist in targets.
# Run from repo root: ./scripts/check-kb-links.sh

set -euo pipefail

KB_DIR="kb"
REPO_ROOT=$(pwd)

if [ ! -d "$KB_DIR" ]; then
    echo "Error: $KB_DIR directory not found. Run from repo root."
    exit 1
fi

ERRFILE=$(mktemp)
trap 'rm -f "$ERRFILE"' EXIT

# GitHub-compatible heading slug: lowercase, strip non-alnum except spaces/hyphens,
# spaces to hyphens, collapse runs. Also strip markdown link syntax from heading text.
slugify() {
    echo "$1" \
        | sed -E 's/\[([^]]*)\]\([^)]*\)/\1/g' \
        | tr '[:upper:]' '[:lower:]' \
        | sed 's/[^a-z0-9 _-]//g' \
        | sed 's/ /-/g' \
        | sed 's/--*/-/g'
}

heading_exists() {
    local file="$1" fragment="$2"
    while IFS= read -r heading; do
        local slug
        slug=$(slugify "$heading")
        if [ "$slug" = "$fragment" ]; then
            return 0
        fi
    done < <(grep -E '^#{1,6} ' "$file" 2>/dev/null | sed 's/^#\{1,6\} //')
    return 1
}

echo "Checking KB cross-references..."
echo

while IFS= read -r src_file; do
    src_dir=$(dirname "$src_file")

    while IFS= read -r link; do
        [[ -z "$link" ]] && continue
        [[ "$link" =~ ^https?:// ]] && continue
        [[ "$link" =~ ^mailto: ]] && continue

        # Split path and fragment
        if [[ "$link" == *"#"* ]]; then
            path="${link%%#*}"
            fragment="${link#*#}"
        else
            path="$link"
            fragment=""
        fi

        # Same-file anchor
        if [ -z "$path" ] && [ -n "$fragment" ]; then
            if ! heading_exists "$src_file" "$fragment"; then
                echo "  ERROR: $src_file -> #$fragment (anchor not found in same file)" >> "$ERRFILE"
            fi
            continue
        fi

        # Resolve relative path
        target=$(cd "$src_dir" && realpath -m "$path" 2>/dev/null || echo "")
        if [ -z "$target" ]; then
            echo "  ERROR: $src_file -> $path (cannot resolve)" >> "$ERRFILE"
            continue
        fi

        # Skip cross-repo links (targets outside the repo root)
        case "$target" in
            "$REPO_ROOT"/*) ;;
            *) continue ;;
        esac

        if [ ! -e "$target" ]; then
            echo "  ERROR: $src_file -> $path (file not found)" >> "$ERRFILE"
            continue
        fi

        # Check heading anchor if present
        if [ -n "$fragment" ] && [ -f "$target" ]; then
            if ! heading_exists "$target" "$fragment"; then
                echo "  ERROR: $src_file -> $path#$fragment (anchor not found)" >> "$ERRFILE"
            fi
        fi

    done < <(grep -oP '\[[^\]!][^\]]*\]\(\K[^)]+' "$src_file" 2>/dev/null || true)

done < <(find "$KB_DIR" -name '*.md' -type f | sort)

if [ -s "$ERRFILE" ]; then
    cat "$ERRFILE"
    count=$(wc -l < "$ERRFILE")
    echo
    echo "Found $count error(s)."
    exit 1
else
    echo "All KB links OK."
    exit 0
fi
