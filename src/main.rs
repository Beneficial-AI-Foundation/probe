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
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Merge { inputs, output } => {
            probe::commands::merge::cmd_merge(inputs, output);
        }
    }
}
