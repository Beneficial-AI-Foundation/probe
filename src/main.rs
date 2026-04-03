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

    /// Partition verified atoms into entrypoints and verified dependencies.
    ///
    /// Reads a Schema 2.0 atom file and partitions all atoms with
    /// "verification-status": "verified" into two lists:
    ///
    /// Entrypoints — verified, non-stub, non-test, Rust `exec` atoms whose
    /// code-name never appears in any atom's dependency list.
    ///
    /// Verified dependencies — all remaining verified atoms (stubs, specs,
    /// proofs, test functions, and depended-upon exec functions).
    ///
    /// Output is a Schema 2.0 envelope with schema "probe/query".
    Query {
        /// Input atom file (Schema 2.0).
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path (defaults to stdout).
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
        Commands::Query { input, output } => {
            probe::commands::query::cmd_query(&input, output.as_deref());
        }
    }
}
