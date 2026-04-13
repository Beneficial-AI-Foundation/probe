#!/usr/bin/env python3
"""Generate a markdown verification report from a probe extract JSON file.

Works with probe-verus, probe-aeneas, and probe-lean extract JSON.

Usage:
    python scripts/summarize_extract.py <input> [OPTIONS]

Arguments:
    input                Path to an extract JSON file, or a repo directory
                         containing .verilib/probes/

Options:
    -o, --output PATH                  Output markdown file (default: stdout)
    --package-summary PATH             Markdown file prepended to the report
    --package-assumptions PATH         Markdown file appended to the report

Examples:
    python scripts/summarize_extract.py path/to/extract.json -o summary.md
    python scripts/summarize_extract.py path/to/extract.json  # stdout
    python scripts/summarize_extract.py path/to/repo          # auto-discover from .verilib/probes/
    python scripts/summarize_extract.py path/to/extract.json \\
        --package-summary summary.md \\
        --package-assumptions assumptions.md \\
        -o report.md
"""

import argparse
import json
import sys
from pathlib import Path

# Maps tool-specific trusted-reason values to common display labels.
TRUST_LABELS = {
    "admit": "axiom",
    "external-body": "external",
    "assume-specification": "assumed spec",
    "axiom": "axiom",
    "external": "external",
    "auto-generated": "auto-generated",
}

# Tool-specific configuration keyed by detected tool family.
TOOL_CONFIG = {
    "verus": {
        "verifier_name": "Verus",
        "axiom_reasons": ("admit",),
        "external_reasons": ("external-body", "assume-specification"),
        "auto_generated_reasons": (),
        "axiom_description": "Functions using `admit()` — the solver accepts the proof without checking.",
        "lemma_kinds": ("proof",),
        "remaining_kinds": ("exec",),
        "remaining_label": "Rust",
    },
    "lean": {
        "verifier_name": "Lean",
        "axiom_reasons": ("axiom",),
        "external_reasons": ("external",),
        "auto_generated_reasons": ("auto-generated",),
        "axiom_description": "Axioms — propositions assumed without proof.",
        "lemma_kinds": ("theorem",),
        "remaining_kinds": ("def", "abbrev", "opaque", "instance", "class", "structure", "inductive"),
        "remaining_label": "Lean",
    },
    "aeneas": {
        "verifier_name": "Lean (via Aeneas)",
        "axiom_reasons": ("axiom",),
        "external_reasons": ("external",),
        "auto_generated_reasons": ("auto-generated",),
        "axiom_description": "Axioms — propositions assumed without proof.",
        "lemma_kinds": ("theorem",),
        "remaining_kinds": ("def", "abbrev", "opaque", "instance"),
        "remaining_label": "Lean",
    },
    "rust": {
        "verifier_name": "probe-rust",
        "axiom_reasons": (),
        "external_reasons": (),
        "auto_generated_reasons": (),
        "axiom_description": "",
        "lemma_kinds": (),
        "remaining_kinds": ("exec",),
        "remaining_label": "Rust",
    },
}


def detect_tool(extract: dict) -> str:
    schema = extract.get("schema", "")
    tool_name = extract.get("tool", {}).get("name", "")
    for key in ("aeneas", "verus", "lean", "rust"):
        if key in schema or key in tool_name:
            return key
    return "verus"


def load_extract(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def get_val(atom: dict, key: str, default=None):
    return atom.get(key, default)


def filtered_ids(data: dict, predicate) -> list[str]:
    """Return sorted probe-ids where predicate(atom) is True."""
    return sorted(pid for pid, atom in data.items() if predicate(atom))


def bullet_list(ids: list[str], annotation_fn=None) -> str:
    if not ids:
        return "None\n"
    lines = []
    for pid in ids:
        if annotation_fn:
            ann = annotation_fn(pid)
            lines.append(f"- `{pid}` {ann}" if ann else f"- `{pid}`")
        else:
            lines.append(f"- `{pid}`")
    return "\n".join(lines) + "\n"


def resolve_primary_spec(data: dict, atom: dict) -> str | None:
    """Resolve primary-spec for an atom.

    Aeneas: follow translation-name -> lean atom -> primary-spec.
    Lean: read primary-spec directly from the atom.
    """
    translation = atom.get("translation-name")
    if translation is not None:
        lean_atom = data.get(translation)
        if lean_atom is not None:
            ps = lean_atom.get("primary-spec")
            if ps is not None:
                return ps
    return atom.get("primary-spec")


def trust_label(reason: str | None) -> str:
    if reason is None:
        return "unknown"
    return TRUST_LABELS.get(reason, reason)


# ---------------------------------------------------------------------------
# Shared report sections (used by both Lean and non-Lean reports)
# ---------------------------------------------------------------------------

def _trust_base_section(out, data, cfg):
    """Section 3: Trust base — axioms, external functions, auto-generated."""
    all_axiom_reasons = cfg["axiom_reasons"]
    all_external_reasons = cfg["external_reasons"]
    all_auto_generated_reasons = cfg.get("auto_generated_reasons", ())

    out.append("## 3. Trust base\n")

    # 3a. Axioms
    axioms = filtered_ids(
        data,
        lambda a: get_val(a, "trusted-reason") in all_axiom_reasons,
    )
    out.append(f"### 3a. Properties assumed to hold ({len(axioms)} axioms)\n")
    if cfg["axiom_description"]:
        out.append(f"{cfg['axiom_description']}\n")
    out.append(bullet_list(axioms))

    # 3b. External functions
    external_trusted = filtered_ids(
        data,
        lambda a: get_val(a, "trusted-reason") in all_external_reasons,
    )
    out.append(
        f"### 3b. External functions assumed correct w.r.t. their specs ({len(external_trusted)})\n"
    )
    out.append(
        bullet_list(
            external_trusted,
            annotation_fn=lambda pid: f"({trust_label(get_val(data[pid], 'trusted-reason'))})",
        )
    )

    # 3c. Auto-generated declarations (Lean and Aeneas)
    if all_auto_generated_reasons:
        auto_gen = filtered_ids(
            data,
            lambda a: get_val(a, "trusted-reason") in all_auto_generated_reasons,
        )
        if auto_gen:
            out.append(
                f"### 3c. Auto-generated declarations ({len(auto_gen)})\n"
            )
            out.append(
                "Kernel-synthesized declarations without source location (congruence lemmas, eliminators, etc.).\n"
            )
            out.append(bullet_list(auto_gen))


def _unverified_section(out, data):
    """Section 4: Unverified and failed functions."""
    failed = filtered_ids(
        data, lambda a: get_val(a, "verification-status") == "failed"
    )
    unverified = filtered_ids(
        data, lambda a: get_val(a, "verification-status") == "unverified"
    )
    combined = len(failed) + len(unverified)
    out.append(f"## 4. Unverified and failed functions ({combined})\n")
    if combined == 0:
        out.append("None\n")
    else:
        if failed:
            out.append(
                bullet_list(failed, annotation_fn=lambda _pid: "[FAILED]")
            )
        if unverified:
            out.append(bullet_list(unverified))
    return combined


def _lemmas_section(out, data, cfg):
    """Section 6: Verified lemmas."""
    lemma_kinds = cfg["lemma_kinds"]
    lemmas = filtered_ids(
        data,
        lambda a: get_val(a, "kind") in lemma_kinds
        and get_val(a, "verification-status") == "verified",
    )
    out.append(f"## 6. Verified lemmas ({len(lemmas)})\n")
    if lemmas:
        out.append(bullet_list(lemmas))
    else:
        out.append("None\n")
    return lemmas


# ---------------------------------------------------------------------------
# Lean-specific report generation
# ---------------------------------------------------------------------------

def _generate_lean_report(out, data, cfg):
    """Generate sections 1, 2, 5, 7, and footer for Lean projects.

    Lean has no public/private API distinction, so:
    - Section 1 lists all verified definitions
    - Section 2 lists all trusted definitions
    - Section 5 is empty (all captured in section 1)
    - Section 7 is empty (no public API concept)
    """
    remaining_kinds = cfg["remaining_kinds"]
    remaining_label = cfg["remaining_label"]

    def spec_annotation(pid: str) -> str:
        spec = resolve_primary_spec(data, data[pid])
        if spec is None:
            return ""
        return f"(spec: `{spec}`)"

    # --- 1. Verified definitions ---
    verified_defs = filtered_ids(
        data,
        lambda a: get_val(a, "kind") in remaining_kinds
        and get_val(a, "verification-status") == "verified",
    )
    out.append(f"## 1. Verified definitions ({len(verified_defs)})\n")
    out.append(bullet_list(verified_defs, annotation_fn=spec_annotation))

    # --- 2. Trusted definitions ---
    trusted_defs = filtered_ids(
        data,
        lambda a: get_val(a, "kind") in remaining_kinds
        and get_val(a, "verification-status") == "trusted",
    )
    out.append(f"## 2. Trusted definitions ({len(trusted_defs)})\n")

    def trusted_annotation(pid: str) -> str:
        parts = [f"({trust_label(get_val(data[pid], 'trusted-reason'))})"]
        s = spec_annotation(pid)
        if s:
            parts.append(s)
        return " ".join(parts)

    out.append(bullet_list(trusted_defs, annotation_fn=trusted_annotation))

    # --- 3. Trust base (shared) ---
    _trust_base_section(out, data, cfg)

    # --- 4. Unverified and failed (shared) ---
    combined = _unverified_section(out, data)

    # --- 5. Verified remaining (empty for Lean) ---
    out.append(f"## 5. Verified remaining {remaining_label} functions (0)\n")
    out.append("All verified definitions are listed in section 1.\n")

    # --- 6. Lemmas (shared) ---
    lemmas = _lemmas_section(out, data, cfg)

    # --- 7. Out-of-scope (not applicable for Lean) ---
    out.append("## 7. Out-of-scope functions (0)\n")
    out.append("Not applicable — Lean does not have a public/private API distinction.\n")

    # --- Accounting footer ---
    out.append("---\n")
    out.append("## Verification accounting\n")
    out.append("| Category | Count |")
    out.append("|----------|------:|")
    out.append(f"| Verified definitions | {len(verified_defs)} |")
    out.append(f"| Trusted definitions | {len(trusted_defs)} |")
    out.append(f"| Verified lemmas | {len(lemmas)} |")
    out.append(f"| Unverified / failed | {combined} |")
    total = len(verified_defs) + len(trusted_defs) + len(lemmas) + combined
    out.append(f"| **Total** | **{total}** |")
    out.append("")


# ---------------------------------------------------------------------------
# Non-Lean report generation (Verus, Aeneas, Rust)
# ---------------------------------------------------------------------------

def _generate_non_lean_report(out, data, cfg, tool):
    """Generate sections 1, 2, 5, 7, and footer for tools with public API."""
    verifier = cfg["verifier_name"]
    remaining_kinds = cfg["remaining_kinds"]
    remaining_label = cfg["remaining_label"]
    show_specs = tool in ("lean", "aeneas")

    def spec_annotation(pid: str) -> str:
        spec = resolve_primary_spec(data, data[pid])
        if spec is None:
            return ""
        return f"(spec: `{spec}`)"

    # --- 1. Verified public API ---
    verified_pub = filtered_ids(
        data,
        lambda a: get_val(a, "is-public-api") is True
        and get_val(a, "verification-status") == "verified",
    )
    out.append(f"## 1. Verified public API functions ({len(verified_pub)})\n")
    out.append(bullet_list(
        verified_pub,
        annotation_fn=spec_annotation if show_specs else None,
    ))

    # --- 2. Trusted public API ---
    trusted_pub = filtered_ids(
        data,
        lambda a: get_val(a, "is-public-api") is True
        and get_val(a, "verification-status") == "trusted",
    )
    out.append(f"## 2. Trusted public API functions ({len(trusted_pub)})\n")

    def trusted_annotation(pid: str) -> str:
        parts = [f"({trust_label(get_val(data[pid], 'trusted-reason'))})"]
        if show_specs:
            s = spec_annotation(pid)
            if s:
                parts.append(s)
        return " ".join(parts)

    out.append(bullet_list(trusted_pub, annotation_fn=trusted_annotation))

    # --- 3. Trust base (shared) ---
    _trust_base_section(out, data, cfg)

    # --- 4. Unverified and failed (shared) ---
    _unverified_section(out, data)

    # --- 5. Verified remaining functions ---
    verified_remaining = filtered_ids(
        data,
        lambda a: get_val(a, "kind") in remaining_kinds
        and get_val(a, "verification-status") == "verified"
        and get_val(a, "is-public-api") is not True,
    )
    out.append(
        f"## 5. Verified remaining {remaining_label} functions ({len(verified_remaining)})\n"
    )
    out.append(bullet_list(
        verified_remaining,
        annotation_fn=spec_annotation if show_specs else None,
    ))

    # --- 6. Lemmas (shared) ---
    _lemmas_section(out, data, cfg)

    # --- 7. Out-of-scope public API ---
    oos_pub = filtered_ids(
        data,
        lambda a: get_val(a, "is-public-api") is True
        and get_val(a, "verification-status") is None,
    )

    def oos_reason(pid: str) -> str:
        atom = data[pid]
        if get_val(atom, "is-cfg-gated") is True:
            return "(cfg-gated)"
        if get_val(atom, "is-external") is True:
            return "(external)"
        if get_val(atom, "has-body") is False:
            return "(bodyless)"
        return "(other)"

    out.append(f"## 7. Out-of-scope public API functions ({len(oos_pub)})\n")
    out.append(
        f"Public API functions that {verifier} did not process.\n"
    )
    out.append(bullet_list(oos_pub, annotation_fn=oos_reason))

    # --- Accounting footer ---
    out.append("---\n")
    out.append("## Public API accounting\n")
    out.append("| Category | Count |")
    out.append("|----------|------:|")
    out.append(f"| Verified public API | {len(verified_pub)} |")
    out.append(f"| Trusted public API | {len(trusted_pub)} |")
    out.append(f"| Out-of-scope public API | {len(oos_pub)} |")
    total_pub = len(verified_pub) + len(trusted_pub) + len(oos_pub)
    out.append(f"| **Total public API** | **{total_pub}** |")
    out.append("")


# ---------------------------------------------------------------------------
# Main entry point
# ---------------------------------------------------------------------------

def generate_report(extract: dict) -> str:
    source = extract.get("source", {})
    pkg_name = source.get("package", "unknown")
    pkg_version = source.get("package-version", "unknown")
    data = extract.get("data", {})

    tool = detect_tool(extract)
    cfg = TOOL_CONFIG[tool]

    out = []
    out.append(f"# Verification report: {pkg_name} {pkg_version}\n")

    if tool == "lean":
        _generate_lean_report(out, data, cfg)
    else:
        _generate_non_lean_report(out, data, cfg, tool)

    return "\n".join(out)


def find_extract_json(repo_dir: Path) -> Path:
    """Find the main extract JSON inside a repo's .verilib/probes/ directory.

    The main extract is identified by having both "source" and "data" top-level
    keys, distinguishing it from derived files (summaries, atoms, specs, etc.).
    """
    probes_dir = repo_dir / ".verilib" / "probes"
    if not probes_dir.is_dir():
        print(f"Error: {probes_dir} does not exist or is not a directory.", file=sys.stderr)
        sys.exit(1)

    candidates = []
    for json_file in sorted(probes_dir.glob("*.json")):
        try:
            with open(json_file) as f:
                obj = json.load(f)
            schema = obj.get("schema", "")
            if schema.endswith("/extract"):
                candidates.append(json_file)
        except (json.JSONDecodeError, OSError):
            continue

    if not candidates:
        print(f"Error: no extract JSON found in {probes_dir}.", file=sys.stderr)
        sys.exit(1)
    if len(candidates) > 1:
        names = ", ".join(c.name for c in candidates)
        print(
            f"Error: multiple extract JSONs found in {probes_dir}: {names}\n"
            f"Pass the specific file path instead.",
            file=sys.stderr,
        )
        sys.exit(1)

    return candidates[0]


def load_package_summary(repo_dir: Path) -> str:
    """Load .verilib/package-summary.md or return an empty section."""
    summary_path = repo_dir / ".verilib" / "package-summary.md"
    if summary_path.is_file():
        return summary_path.read_text().rstrip("\n") + "\n"
    return "# Package Summary\n\n_No package summary available._\n"


def load_markdown_file(path: Path) -> str | None:
    """Load a markdown file and return its content, or None if not found."""
    if not path.is_file():
        print(f"Warning: {path} not found, skipping.", file=sys.stderr)
        return None
    return path.read_text().rstrip("\n") + "\n"


def main():
    parser = argparse.ArgumentParser(
        description="Generate a markdown verification report from a probe extract JSON."
    )
    parser.add_argument(
        "input",
        help="Path to an extract JSON file, or a repo directory containing .verilib/probes/",
    )
    parser.add_argument("-o", "--output", help="Output markdown file (default: stdout)")
    parser.add_argument(
        "--package-summary",
        help="Path to a markdown file with the package summary (prepended to the report)",
    )
    parser.add_argument(
        "--package-assumptions",
        help="Path to a markdown file with trust assumptions (appended to the report)",
    )
    args = parser.parse_args()

    input_path = Path(args.input)
    package_summary = None

    if args.package_summary:
        package_summary = load_markdown_file(Path(args.package_summary))
    elif input_path.is_dir():
        package_summary = load_package_summary(input_path)

    if input_path.is_dir():
        json_path = find_extract_json(input_path)
        print(f"Using extract: {json_path}", file=sys.stderr)
    else:
        json_path = input_path

    extract = load_extract(str(json_path))
    report = generate_report(extract)

    if package_summary is not None:
        report = package_summary + "\n---\n\n" + report

    if args.package_assumptions:
        assumptions = load_markdown_file(Path(args.package_assumptions))
        if assumptions is not None:
            report = report + "\n---\n\n" + assumptions

    if args.output:
        Path(args.output).write_text(report)
        print(f"Wrote report to {args.output}", file=sys.stderr)
    else:
        print(report)


if __name__ == "__main__":
    main()
