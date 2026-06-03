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
    /// Takes two or more Schema 2.0 files (atoms, specs, or proofs from
    /// probe-verus, probe-lean, etc.) and produces a merged file. The schema
    /// category is auto-detected from the inputs; all inputs must be the same
    /// category. For atoms, stubs are replaced by real entries (first-wins on
    /// conflict). For specs and proofs, last-wins on conflict.
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
        /// the merge can add cross-language dependency edges. See
        /// docs/mappings-spec.md for the file format.
        #[arg(short, long)]
        mappings: Option<PathBuf>,
    },

    /// Project a subgraph from a merged atom file using mapping seeds.
    ///
    /// Reads a Schema 2.0 atom file and a mappings file, uses all mapping
    /// endpoints (from + to) as seeds, then expands via BFS: forward
    /// (callees) and backward (callers) with separate depth controls.
    /// Outputs a trimmed atom file containing only the projected subgraph.
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

        /// Also emit a focus-set JSON for scip-callgraph ?focus= param.
        #[arg(long)]
        emit_focus: bool,
    },

    /// Enrich verification status through the dependency graph.
    ///
    /// Reads a Schema 2.0 atom file, walks the dependency graph, and
    /// upgrades `verification-status` from "verified" to
    /// "transitively-verified" on atoms whose entire transitive dependency
    /// closure is verified or trusted. Atoms that remain "verified" are only
    /// locally verified (some transitive dep is not verified/trusted).
    ///
    /// The output preserves the input envelope structure exactly.
    // @kb: kb/engineering/properties.md#p23-transitive-verification-is-computed-by-reverse-bfs-contamination
    Enrich {
        /// Input atom file (Schema 2.0).
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Summarize verified atoms: entrypoints, functions, and lemmas.
    ///
    /// Reads a Schema 2.0 atom file and partitions all atoms with
    /// "verification-status": "verified" into three lists:
    ///
    /// Entrypoints — verified, non-stub, non-test, Rust `exec` atoms whose
    /// code-name never appears in any non-test atom's dependency list.
    ///
    /// Verified functions — remaining verified Rust `exec` atoms.
    ///
    /// Verified lemmas — verified Verus `proof`/`spec` atoms.
    ///
    /// Output is a Schema 2.0 envelope with schema "probe/summary".
    Summary {
        /// Input atom file (Schema 2.0).
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
