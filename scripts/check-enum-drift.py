#!/usr/bin/env python3
"""Enum drift guard for the live reference docs.

The status/kind/language enum value sets are defined once, in
kb/engineering/schema.md. Reference docs must link there instead of
restating the lists ("never re-enumerate an enum in docs/") — restated
copies rot silently when the KB evolves (docs-report C1/C2).

This check flags any line in the tracked live docs surface (README.md +
docs/, excluding docs/archive/) that contains THRESHOLD or more distinct
values of the same enum: single mentions are legitimate, enumerations
are the drift hazard. kb/ itself is out of scope — its normative files
restate enum subsets by design and are kept consistent by the KB
auditors.

Deliberate enumerations (e.g. the UI-owned colour mapping in
docs/ui-views.md) opt out with an inline `<!-- enum-ok -->` marker.

The value lists below are fixed strings by design (keep it dumb); update
them in lockstep with kb/engineering/schema.md §§ Core fields /
Kind values / Common optional fields.

Run from the repo root: ./scripts/check-enum-drift.py [--self-test]
"""

import re
import subprocess
import sys

THRESHOLD = 3

ENUMS = {
    # kb/engineering/schema.md § Common optional fields (verification-status)
    "verification-status": [
        "transitively-verified",
        "unverified",
        "verified",
        "failed",
        "trusted",
    ],
    # kb/engineering/schema.md § Core fields (language)
    "language": ["rust", "verus", "lean", "blueprint"],
    # kb/engineering/schema.md § Kind values
    "kind": [
        "exec",
        "proof",
        "spec",
        "def",
        "theorem",
        "abbrev",
        "class",
        "structure",
        "inductive",
        "instance",
        "axiom",
        "opaque",
        "quot",
        "blueprint-definition",
        "blueprint-theorem",
    ],
}

# Field names removed from the schema; any live-doc occurrence is rot.
DEAD_VALUES = ["is-disabled"]

# Tokens that contain enum words without being enum usage, removed
# before counting: tool names, toolchain names, schema-string patterns
# like `probe-(rust|lean|verus|aeneas|leanblueprint)/(atoms|...)`.
COMPOSITES = re.compile(
    r"probe-\([^)\n]*\)[^\s`]*"
    r"|probe-(rust|verus|lean|leanblueprint|aeneas|vcvio)"
    r"|(rust|verus)-analyzer"
    r"|rust-toolchain"
)

OPT_OUT = "<!-- enum-ok -->"


def value_pattern(value: str) -> re.Pattern:
    # Hyphen counts as a word character here so that `verified` does not
    # match inside `transitively-verified` (longest-first consumption
    # also removes the longer value before shorter ones are counted).
    return re.compile(r"(?<![\w-])" + re.escape(value) + r"(?![\w-])")


def enum_hits(line: str) -> list[tuple[str, list[str]]]:
    """Return (enum-name, found-values) for enums with >= THRESHOLD
    distinct values on the line, plus any dead value, after stripping
    composites."""
    stripped = COMPOSITES.sub("~", line)
    hits = []
    for name, values in ENUMS.items():
        found = []
        remaining = stripped
        for value in sorted(values, key=len, reverse=True):
            pat = value_pattern(value)
            if pat.search(remaining):
                found.append(value)
                remaining = pat.sub("~", remaining)
        if len(found) >= THRESHOLD:
            hits.append((name, sorted(found)))
    for dead in DEAD_VALUES:
        if value_pattern(dead).search(stripped):
            hits.append(("dead value", [dead]))
    return hits


def scanned_files() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "README.md", "docs/*.md", "docs/**/*.md"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return sorted(
        f for f in set(out.splitlines()) if f and not f.startswith("docs/archive/")
    )


def scan() -> int:
    errors = []
    for path in scanned_files():
        with open(path, encoding="utf-8") as fh:
            for lineno, line in enumerate(fh, 1):
                if OPT_OUT in line:
                    continue
                for enum_name, found in enum_hits(line):
                    errors.append(
                        f"  ERROR: {path}:{lineno} restates {enum_name} "
                        f"values {', '.join(found)}"
                    )
    if errors:
        print("Enum drift found (enums are defined in kb/engineering/schema.md;")
        print("link there instead of restating, or mark deliberate lines with")
        print(f"{OPT_OUT}):")
        print("\n".join(errors))
        print(f"\nFound {len(errors)} error(s).")
        return 1
    print("No enum drift in the live docs.")
    return 0


def self_test() -> int:
    # The exact restatement lines from docs-report C1/C2 (git history)
    # must trip the detector; ordinary tool-name prose must not.
    must_flag = [
        # C1: docs/ui-views.md before PR #67
        'Every atom carries a `language` field (`"rust"`, `"lean"`, or `"verus"`).',
        "green = verified, red = failed, grey = unverified, blue = unknown.",
        "| **Kind values** | `exec`, `proof`, `spec` (Verus) | `def`, `theorem`, ...",
        # C2: docs/probes-overview-slides.md before PR #69
        "- kinds: def/abbrev/thm/... for lean; exec/spec/proof for verus",
        "the `is-disabled` marker",
    ]
    must_pass = [
        "merge output from probe-rust, probe-lean, and probe-verus",
        "`probe-(rust|lean|verus|aeneas|leanblueprint)/(specs|proofs|stubs|verification-report)`",
        "upgrade `verified` to `transitively-verified` (see the KB)",
        "def check_completeness(atoms, rendered_nodes, rendered_edges):",
        "red = `failed`, yellow = `unverified`, light green = `verified`, <!-- enum-ok -->",
    ]
    failures = []
    for line in must_flag:
        if not enum_hits(line):
            failures.append(f"  should flag but passed: {line}")
    for line in must_pass:
        if OPT_OUT not in line and enum_hits(line):
            failures.append(f"  should pass but flagged: {line}")
    if failures:
        print("Self-test FAILED:")
        print("\n".join(failures))
        return 1
    print("Self-test OK.")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        sys.exit(self_test())
    sys.exit(scan())
