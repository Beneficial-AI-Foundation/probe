use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "probe-extract-check")]
#[command(about = "Validate probe extract JSON against source code")]
struct Cli {
    /// Path to the extract JSON file.
    json: PathBuf,

    /// Path to the source project root.
    ///
    /// If omitted, only structural checks are run (no source validation).
    #[arg(short, long)]
    project: Option<PathBuf>,

    /// Exit with 0 even if there are warnings (still fail on errors).
    #[arg(long)]
    allow_warnings: bool,
}

fn main() {
    let cli = Cli::parse();

    let envelope = match probe_extract_check::load_extract_json(&cli.json) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(2);
        }
    };

    let report = probe_extract_check::check_all(&envelope, cli.project.as_deref());
    report.print_summary();

    if !report.is_ok() {
        process::exit(1);
    }
    if !cli.allow_warnings && report.warning_count() > 0 {
        process::exit(1);
    }
}
