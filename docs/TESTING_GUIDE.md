# Testing Guide for Probe Consumers

How to verify that your visualization of probe data is correct.

As a consumer, your responsibility is to test that your visualization
faithfully represents the probe JSON (**data fidelity**). You do not
need to verify that the JSON itself is correct -- that is handled by the
test suites in each probe repository (see
[How the probes test extraction correctness](#how-the-probes-test-extraction-correctness)
below).

## Testing your visualization (data fidelity)

These tests verify that your rendered graph accurately represents the
probe output. They only need the JSON file -- no access to the original
source code.

### Schema validation

Before rendering, validate the input JSON against the
[JSON Schema](https://github.com/Beneficial-AI-Foundation/probe/blob/main/schemas/atom-envelope.schema.json)
provided in the probe repo. See the
[probe README](https://github.com/Beneficial-AI-Foundation/probe#json-schema)
for examples in Rust, Python, and CLI.

```bash
pip install jsonschema
python -m jsonschema -i output.json atom-envelope.schema.json
```

### Node and edge completeness

Every non-stub atom in `data` should appear as a node in your
visualization, and every `dependencies` entry (where both endpoints are
non-stubs) should appear as an edge.

```python
import json

def load_atoms(path):
    with open(path) as f:
        return json.load(f)["data"]

def non_stub(atom):
    return atom["code-path"] != ""

def check_completeness(atoms, rendered_nodes, rendered_edges):
    """
    rendered_nodes: set of code-names shown as nodes
    rendered_edges: set of (source, target) tuples shown as edges
    """
    missing_nodes = []
    missing_edges = []

    for code_name, atom in atoms.items():
        if not non_stub(atom):
            continue
        if code_name not in rendered_nodes:
            missing_nodes.append(code_name)
        for dep in atom["dependencies"]:
            dep_atom = atoms.get(dep)
            if dep_atom and non_stub(dep_atom):
                if (code_name, dep) not in rendered_edges:
                    missing_edges.append((code_name, dep))

    return missing_nodes, missing_edges
```

### Attribute accuracy

Spot-check that rendered labels, file paths, and line numbers match the
JSON fields:

```python
def check_attributes(atoms, get_rendered):
    """
    get_rendered(code_name) -> dict with keys 'label', 'file', 'kind'
    as shown in the visualization, or None if not rendered.
    """
    mismatches = []
    for code_name, atom in atoms.items():
        if not non_stub(atom):
            continue
        rendered = get_rendered(code_name)
        if rendered is None:
            continue
        if rendered["label"] != atom["display-name"]:
            mismatches.append((code_name, "display-name", atom["display-name"], rendered["label"]))
        if rendered["file"] != atom["code-path"]:
            mismatches.append((code_name, "code-path", atom["code-path"], rendered["file"]))
    return mismatches
```

### Playwright example

As the visualization runs in a browser, a Playwright test can extract
the rendered graph from the DOM and compare it against the JSON:

```python
from playwright.sync_api import sync_playwright

def test_graph_matches_json(probe_json_path, app_url):
    atoms = load_atoms(probe_json_path)

    with sync_playwright() as p:
        page = p.chromium.launch().new_page()
        page.goto(app_url)
        page.wait_for_selector("[data-testid='graph-ready']")

        rendered_nodes = set(
            el.get_attribute("data-code-name")
            for el in page.query_selector_all("[data-testid='graph-node']")
        )
        rendered_edges = set()
        for el in page.query_selector_all("[data-testid='graph-edge']"):
            src = el.get_attribute("data-source")
            tgt = el.get_attribute("data-target")
            rendered_edges.add((src, tgt))

        missing_nodes, missing_edges = check_completeness(atoms, rendered_nodes, rendered_edges)
        assert not missing_nodes, f"Missing nodes: {missing_nodes[:5]}"
        assert not missing_edges, f"Missing edges: {missing_edges[:5]}"
```

This assumes your visualization attaches `data-code-name`, `data-source`,
and `data-target` attributes to DOM elements -- adapt the selectors to
match your implementation.

## How the probes test extraction correctness

You do not need to verify that the probe JSON accurately reflects the
source code -- each probe repository has its own test suite that covers
this. Here is what each repo tests:

### probe (this repo)

Run with `cargo test` in the probe repo.

- **Schema validation** (7 tests) -- envelopes for each tool
  (`probe-<x>/extract`, merged atoms) are validated against the
  [JSON Schema](https://github.com/Beneficial-AI-Foundation/probe/blob/main/schemas/atom-envelope.schema.json).
  A negative test confirms that missing required fields are rejected.
- **Merge integration** (11 tests) -- runs the `probe merge` binary on
  fixture files and checks that stubs are replaced by real atoms,
  cross-project dependency edges are preserved, provenance is recorded,
  and category mismatches (e.g. mixing atoms and specs) are rejected.
  Covers atoms, specs, and proofs merging with last-wins semantics.
- **Merge unit** (15 tests) -- exercises merge logic in isolation:
  stub replacement, real-vs-real conflict resolution, trailing-dot
  normalization, cross-language merge, translation-based edge creation,
  and recursive provenance flattening.

### Individual probes

Each probe repository has its own test suite covering extraction
correctness. See each probe's `TESTING.md` for full test inventory,
run instructions, and CI details.

| Probe | Test command | Details |
|-------|-------------|---------|
| probe-rust | `cargo test` | [TESTING.md](https://github.com/Beneficial-AI-Foundation/probe-rust/blob/main/TESTING.md) |
| probe-verus | `cargo test` | [TESTING.md](https://github.com/Beneficial-AI-Foundation/probe-verus/blob/main/TESTING.md) |
| probe-aeneas | `cargo test` | [TESTING.md](https://github.com/Beneficial-AI-Foundation/probe-aeneas/blob/main/TESTING.md) |
| probe-lean | `lake build tests && .lake/build/bin/tests` | [TESTING.md](https://github.com/Beneficial-AI-Foundation/probe-lean/blob/main/TESTING.md) |

If you suspect an issue with the extracted data, open an issue in the
relevant probe repository.
