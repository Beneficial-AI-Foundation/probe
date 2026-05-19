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

        /// Translations file for cross-language atom matching.
        ///
        /// Maps code-names between languages (e.g., Rust ↔ Lean) so that
        /// the merge can add cross-language dependency edges. See
        /// docs/translations-spec.md for the file format.
        #[arg(short, long)]
        translations: Option<PathBuf>,
    },

    /// Propagate verification status through the dependency graph.
    ///
    /// Reads a Schema 2.0 atom file, walks the dependency graph, and
    /// computes whether each verified atom is transitively verified
    /// (all transitive dependencies are also verified/trusted) or only
    /// locally-scoped verified (the atom itself is verified but some
    /// transitive dependency is not).
    ///
    /// Adds `transitive-verification-status` ("transitive" or "local")
    /// to each verified atom. The output preserves the input envelope
    /// structure exactly.
    // @kb: kb/engineering/properties.md#p23-transitive-verification-scope-is-computed-by-reverse-bfs-contamination
    PropagateVerificationStatus {
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
            translations,
        } => {
            probe::commands::merge::cmd_merge(inputs, output, translations);
        }
        Commands::PropagateVerificationStatus { input, output } => {
            probe::commands::propagate::cmd_propagate_verification_status(
                &input,
                output.as_deref(),
            );
        }
        Commands::Summary { input, output } => {
            probe::commands::summary::cmd_summary(&input, output.as_deref());
        }
    }
}
