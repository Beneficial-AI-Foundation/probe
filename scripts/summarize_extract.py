#!/usr/bin/env python3
"""Generate a markdown verification report from a probe extract JSON file.

Works with probe-verus and probe-aeneas extract JSON.

Usage:
    python scripts/summarize_extract.py path/to/extract.json -o summary.md
    python scripts/summarize_extract.py path/to/extract.json  # stdout
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
}

# Tool-specific configuration keyed by detected tool family.
TOOL_CONFIG = {
    "verus": {
        "verifier_name": "Verus",
        "axiom_reasons": ("admit",),
        "external_reasons": ("external-body", "assume-specification"),
        "axiom_description": "Functions using `admit()` — the solver accepts the proof without checking.",
        "lemma_kinds": ("proof",),
        "remaining_kinds": ("exec",),
        "remaining_label": "Rust",
    },
    "lean": {
        "verifier_name": "Lean",
        "axiom_reasons": ("axiom",),
        "external_reasons": ("external",),
        "axiom_description": "Axioms — propositions assumed without proof.",
        "lemma_kinds": ("theorem",),
        "remaining_kinds": ("def", "abbrev", "opaque"),
        "remaining_label": "Lean",
    },
    "aeneas": {
        "verifier_name": "Lean (via Aeneas)",
        "axiom_reasons": ("axiom",),
        "external_reasons": ("external",),
        "axiom_description": "Axioms — propositions assumed without proof.",
        "lemma_kinds": ("theorem",),
        "remaining_kinds": ("def", "abbrev", "opaque"),
        "remaining_label": "Lean",
    },
    "rust": {
        "verifier_name": "probe-rust",
        "axiom_reasons": (),
        "external_reasons": (),
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
    """Follow atom -> translation-name -> primary-spec for aeneas extracts."""
    translation = atom.get("translation-name")
    if translation is None:
        return None
    lean_atom = data.get(translation)
    if lean_atom is None:
        return None
    return lean_atom.get("primary-spec")


def trust_label(reason: str | None) -> str:
    if reason is None:
        return "unknown"
    return TRUST_LABELS.get(reason, reason)


def generate_report(extract: dict) -> str:
    source = extract.get("source", {})
    pkg_name = source.get("package", "unknown")
    pkg_version = source.get("package-version", "unknown")
    data = extract.get("data", {})

    tool = detect_tool(extract)
    cfg = TOOL_CONFIG[tool]
    verifier = cfg["verifier_name"]
    all_axiom_reasons = cfg["axiom_reasons"]
    all_external_reasons = cfg["external_reasons"]
    all_trust_reasons = all_axiom_reasons + all_external_reasons

    out = []

    show_specs = tool in ("lean", "aeneas")

    def spec_annotation(pid: str) -> str:
        spec = resolve_primary_spec(data, data[pid])
        if spec is None:
            return ""
        return f"(spec: `{spec}`)"

    # --- Header ---
    out.append(f"# Verification report: {pkg_name} {pkg_version}\n")

    # --- 1. Verified public API ---
    # jq: [.data | to_entries[] | select(.value["is-public-api"] == true and .value["verification-status"] == "verified") | .key] | sort
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
    # jq: [.data | to_entries[] | select(.value["is-public-api"] == true and .value["verification-status"] == "trusted") | .key] | sort
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

    out.append(
        bullet_list(trusted_pub, annotation_fn=trusted_annotation)
    )

    # --- 3. Trust base ---
    out.append("## 3. Trust base\n")

    # 3a. Axioms
    # jq (verus):  [.data | to_entries[] | select(.value["trusted-reason"] == "admit") | .key] | sort
    # jq (lean):   [.data | to_entries[] | select(.value["trusted-reason"] == "axiom") | .key] | sort
    axioms = filtered_ids(
        data,
        lambda a: get_val(a, "trusted-reason") in all_axiom_reasons,
    )
    out.append(f"### 3a. Properties assumed to hold ({len(axioms)} axioms)\n")
    if cfg["axiom_description"]:
        out.append(f"{cfg['axiom_description']}\n")
    out.append(bullet_list(axioms))

    # 3b. External functions
    # jq (verus):  [.data | to_entries[] | select(.value["trusted-reason"] == "external-body" or .value["trusted-reason"] == "assume-specification") | .key] | sort
    # jq (lean):   [.data | to_entries[] | select(.value["trusted-reason"] == "external") | .key] | sort
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

    # --- 4. Unverified and failed ---
    # jq: [.data | to_entries[] | select(.value["verification-status"] == "failed") | .key] | sort
    failed = filtered_ids(
        data, lambda a: get_val(a, "verification-status") == "failed"
    )
    # jq: [.data | to_entries[] | select(.value["verification-status"] == "unverified") | .key] | sort
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

    # --- 5. Verified remaining functions ---
    remaining_kinds = cfg["remaining_kinds"]
    remaining_label = cfg["remaining_label"]
    # jq (verus/rust): [.data | to_entries[] | select(.value.kind == "exec" and .value["verification-status"] == "verified" and .value["is-public-api"] != true) | .key] | sort
    # jq (lean):       [.data | to_entries[] | select((.value.kind == "def" or .value.kind == "abbrev" or .value.kind == "opaque") and .value["verification-status"] == "verified" and .value["is-public-api"] != true) | .key] | sort
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

    # --- 6. Lemmas ---
    lemma_kinds = cfg["lemma_kinds"]
    # jq (verus): [.data | to_entries[] | select(.value.kind == "proof" and .value["verification-status"] == "verified") | .key] | sort
    # jq (lean):  [.data | to_entries[] | select(.value.kind == "theorem" and .value["verification-status"] == "verified") | .key] | sort
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

    # --- 7. Out-of-scope public API ---
    # jq: [.data | to_entries[] | select(.value["is-public-api"] == true and (.value["verification-status"] == null or (has("verification-status") | not))) | .key] | sort
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

    return "\n".join(out)


def main():
    parser = argparse.ArgumentParser(
        description="Generate a markdown verification report from a probe extract JSON."
    )
    parser.add_argument("input", help="Path to the extract JSON file")
    parser.add_argument("-o", "--output", help="Output markdown file (default: stdout)")
    args = parser.parse_args()

    extract = load_extract(args.input)
    report = generate_report(extract)

    if args.output:
        Path(args.output).write_text(report)
        print(f"Wrote report to {args.output}", file=sys.stderr)
    else:
        print(report)


if __name__ == "__main__":
    main()
