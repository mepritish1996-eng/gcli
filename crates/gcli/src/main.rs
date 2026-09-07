//! `gcli` — the binary entry point. Owns argv parsing (via `clap`, which
//! already gives us real POSIX behavior — `--help`/`--version` §8, short+
//! long flags §2, `--` end-of-options §4 — for free instead of hand-rolling
//! a parser) and output plumbing. All real verb logic lives in `gcli-core`;
//! this crate stays a thin shell, same separation as `gcli-core`'s own doc
//! comment describes.
//!
//! Spec: `docs/UNIX_grammar.md` in this repo's root.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use gcli_core::verbs::info;
use gcli_core::GcliError;

/// gcli — the universal GIS CLI.
#[derive(Parser)]
#[command(name = "gcli", version, about, long_about = None)]
struct Cli {
    /// Machine-readable output (§9: separate contract from the
    /// human-readable default — never mixed, always this flag to opt in).
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect a dataset's real metadata — format, CRS, extent, feature count.
    Info {
        /// Input file. Omit (or pass `-`) to read from stdin (§6 piping).
        path: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Info { path } => match info::run(path.as_deref()) {
            Ok(result) => {
                if cli.json {
                    println!("{}", serde_json::to_string(&result).expect("InfoResult is always serializable"));
                } else {
                    print!("{result}");
                }
                ExitCode::SUCCESS
            }
            Err(err) => report_error(&err, cli.json),
        },
    }
}

/// One real, shared error-reporting path (§16) — every verb's failure goes
/// through this, never a per-verb ad hoc `eprintln!`.
fn report_error(err: &GcliError, json: bool) -> ExitCode {
    if json {
        eprintln!("{}", err.to_json_line());
    } else {
        eprintln!("{err}");
    }
    err.process_exit_code()
}
