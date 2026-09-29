// @kb: kb/tools/probe-merge.md — CLI entry point
// @kb: kb/product/spec.md

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "probe")]
#[command(
    author,
    version,
    about = "Cross-tool operations for probe-* verification tools"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Merge data files from multiple probe tools into a single file.
    ///
    /// Takes two or more Schema 3.0 files (atoms, specs, or proofs from
    /// probe-verus, probe-lean, etc.) and produces a merged file. The schema
    /// category is auto-detected from the inputs; all inputs must be the same
    /// category. For atoms, stubs are replaced by real entries (first-wins on
    /// conflict). For specs and proofs, last-wins on conflict.
    ///
    /// Atoms inputs must pass authority validation (ADR-006): projections
    /// (probe/projected-atoms, or the legacy probe/merged-atoms form with a
    /// projection field) and pre-contract envelopes (per-producer
    /// tool.version gate) are rejected — regenerate them instead.
    Merge {
        /// Input files (at least 2 required).
        #[arg(required = true, num_args = 2..)]
        inputs: Vec<PathBuf>,

        /// Output file path.
        #[arg(short, long, default_value = "merged.json")]
        output: PathBuf,

        /// Mappings file for cross-language atom matching.
        ///
        /// Maps code-names between languages (e.g., Rust ↔ Lean) so that
        /// the merge can attach maps-to/mapped-from correspondence records
        /// (never dependency edges). See kb/engineering/schema.md
        /// § Mappings file format.
        #[arg(short, long)]
        mappings: Option<PathBuf>,
    },

    /// Project a subgraph from a merged atom file using mapping seeds.
    ///
    /// Reads a Schema 3.0 atom file and a mappings file, uses all mapping
    /// endpoints (from + to) as seeds, then expands via BFS: forward
    /// (callees) and backward (callers) with separate depth controls.
    /// Outputs a trimmed atom file containing only the projected subgraph,
    /// under the probe/projected-atoms schema — a view that probe merge and
    /// probe enrich reject (ADR-006). The input must pass the per-producer
    /// version gate; already-projected inputs stay readable.
    // @kb: kb/tools/probe-project.md
    Project {
        /// Input atom file (merged or single-tool).
        #[arg(required = true)]
        input: PathBuf,

        /// Mappings file — seeds are all `from` and `to` code-names.
        #[arg(short, long, required = true)]
        mappings: PathBuf,

        /// Forward BFS depth: follow callees from seeds (default = 2).
        #[arg(long, default_value = "2")]
        forward_depth: usize,

        /// Reverse BFS depth: follow callers of seeds (default = 0).
        #[arg(long, default_value = "0")]
        reverse_depth: usize,

        /// Output file path.
        #[arg(short, long, default_value = "projected.json")]
        output: PathBuf,

        /// Also emit a focus-set JSON for probegraph ?focus= param.
        #[arg(long)]
        emit_focus: bool,
    },

    /// Recompute verification labels through the dependency graph.
    ///
    /// Reads a Schema 3.0 atom file, normalizes code-names (P8), and sets
    /// every "verified"/"transitively-verified" label fresh (P23): an atom
    /// is "transitively-verified" iff no seed — an explicit
    /// failed/unverified atom, or any status-origin-bearing atom — is
    /// reachable along a dependency path that doesn't pass through a
    /// trusted boundary. Atoms that come out "verified" are only locally
    /// verified (such a seed is reachable, or the atom is itself a seed);
    /// stale "transitively-verified" labels are downgraded.
    ///
    /// The input must pass authority validation (ADR-006): projections in
    /// either format and pre-contract envelopes are rejected.
    ///
    /// The output preserves the input envelope structure exactly, up to
    /// normalized code-names.
    // @kb: kb/engineering/properties.md#p23-transitive-verification
    Enrich {
        /// Input atom file (Schema 3.0).
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Summarize verified atoms: entrypoints, functions, lemmas, imported.
    ///
    /// Reads a Schema 3.0 atom file and partitions all verified code atoms
    /// (language "blueprint" excluded) into four lists:
    ///
    /// Entrypoints — locally verified, non-stub, non-test, Rust `exec`
    /// atoms whose code-name never appears in any non-test code atom's
    /// dependency list.
    ///
    /// Verified functions — remaining locally verified Rust `exec` atoms.
    ///
    /// Verified lemmas — locally verified non-(Rust exec) code atoms.
    ///
    /// Imported verified — atoms whose verified status carries
    /// "status-origin": "translation" (evidence copied from another
    /// language); never presented as a local result.
    ///
    /// The input must pass the per-producer version gate (ADR-006): a
    /// pre-contract verified status is indistinguishable from local
    /// evidence. Projections stay readable.
    ///
    /// Output is a Schema 3.0 envelope with schema "probe/summary".
    Summary {
        /// Input atom file (Schema 3.0).
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path (defaults to summary_<package>_<version>.json).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Merge {
            inputs,
            output,
            mappings,
        } => {
            probe::commands::merge::cmd_merge(inputs, output, mappings);
        }
        Commands::Project {
            input,
            mappings,
            forward_depth,
            reverse_depth,
            output,
            emit_focus,
        } => {
            probe::commands::project::cmd_project(
                input,
                mappings,
                forward_depth,
                reverse_depth,
                output,
                emit_focus,
            );
        }
        Commands::Enrich { input, output } => {
            probe::commands::propagate::cmd_enrich(&input, output.as_deref());
        }
        Commands::Summary { input, output } => {
            probe::commands::summary::cmd_summary(&input, output.as_deref());
        }
    }
}
