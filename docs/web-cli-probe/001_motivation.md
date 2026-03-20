# Web, CLI, Probe - Motivation

## Relationship between Web, CLI, Probe

* Users can interact with VeriLib through two interfaces  - Web and CLI

* The probe scripts contains functionality that is needed by CLI and Web to support user requests. So hopefully, Web can be decoupled from CLI eventually. 

* The probe scripts are essentially parsers and analyzers. They don't change the source code. They generate output files that contain the parsing and analysis results, which are then used by CLI or Web. They do not filter or delete results unless it is clear that those results will not be used in any situation. 

* The current outputs are: stubs.json, atoms.json, specs.json, proofs.json. We also have spec certificates in .verilib/specs. We can add information to the outputs over time, but hopefully in a way that does not break existing schema. We should make the schema version explicit in the JSON outputs in anticipation of these breaking changes.

## Probe

* Contains installers for tool dependencies, such as SCIP, Verus-Analyzer and Verus

* Language-dependent

* Subcommands for a variety of applications
    * Projects like VeriLib-CLI may use subcommands like `probe-verus atomize` to get atoms in repo and output to JSON
    * Projects like Dalek-Verus may use special subcommands like `probe-verus tracked_csv` to get additional atom properties and output to CSV
    * Projects like Dalek-Lean may use special subcommands like `probe-lean tracked_csv` to get additional atom properties and output to CSV

## CLI

* Language-independent

* Could combine multiple probes

* Only installs itself. Additional steps needed to install `probe-verus` or `probe-lean` and their dependencies. Checks for these dependencies before calling them within the CLI scripts, and give installation instructions if they are missing.

## Explore

* Creates views

* e.g. callgraphs, structure, crates


## Atoms

* In the beginning, there are only .md or .tex stub files. No need to install Lean or Verus till later.

* when calling `probe-XXX stubify`, it can produce `stubs.json` from a variety of sources, like the .md files, latex files, rust files, lean files.

* When do we call `verilib-cli create` to create the first stubs from the existing atoms?

* When do we need to graduate a stub to an actual Lean atom or Rust atom? 
    * Think about spec files in Claude Code.
    * Can have several links in one stub file.
    * No need to graduate the stub. Just update link. Stubs are part of a write-up, not part of function.
    * If user chooses to have separate files for stubs for AI reasons, that's their choice. AI doesn't need separate files.

* Atom types `[Rust, Lean]`

* No such thing as creating stubs from atoms. That is a view.


## Views

* when calling `probe-XXX viewify`, it can produce `views.json` from a variety of sources, like the stub files, latex files, rust files, lean files.

* Progress for Rust, Lean, Latex

* Templates for views

* Already can get basic views from atoms, by labeling each atom as visible or invisible.

## File format

* Schema Version
* Compute Hashes
* List-types: `[Rust, Lean, Latex]`
* Rust:
* Lean:
* Latex:

## Plan

* Update probe-verus to have schema version
    * Read schema version from .verilib/config.json
    * Add probes needed in .verilib/config.json
* One line installers
    * ask user to run uv, 
    * it will list things that the installer will do
    * ask user for permission before proceeding
    * If installer fails, where to go for more information
* Update probe-verus to have compute hashes
    * e.g. if atoms.json is needed before specs.json
    * check if atoms.json is up-to-date with latest source
    * just use git hashes, and have option to recompute if missing
* Update probe-verus to have sublists for atoms, specs, proofs
    * no more stubs.json
    * perhaps also no more specs.json and proofs.json
* Update probe-verus to create simple views
* Create probe-XXX to create complex views
    * it can call other probes to form a merged atoms.json
    * e.g. probe-aeneas calls probe-rust, probe-lean
* Make Curve25519-Dalek-Lean-Verify call probe-lean instead of Utils
* Make sure "data" folder in probe-verus is now ".verilib"
    * ask user if they want to gitignore the generated files
    * or just have an "data" folder and put everything there



