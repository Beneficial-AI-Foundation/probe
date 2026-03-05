use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "probe")]
#[command(
    author,
    version,
    about = "Cross-tool atom operations for probe-* verification tools"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Merge atom files from multiple probe tools into a single file.
    ///
    /// Takes two or more Schema 2.0 atom files (from probe-verus, probe-lean, etc.)
    /// and produces a merged file with schema "probe/merged-atoms". Stubs are replaced
    /// by real atoms when the same code-name appears in multiple inputs.
    MergeAtoms {
        /// Input atom files (at least 2 required).
        #[arg(required = true, num_args = 2..)]
        inputs: Vec<PathBuf>,

        /// Output file path.
        #[arg(short, long, default_value = "merged_atoms.json")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::MergeAtoms { inputs, output } => {
            probe::commands::merge_atoms::cmd_merge_atoms(inputs, output);
        }
    }
}
