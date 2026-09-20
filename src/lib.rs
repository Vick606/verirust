//! VeriRust — deterministic compile, run, and verify for Rust submissions.
//!
//! The binary in `main.rs` is a thin wrapper around this library. Keeping the
//! logic here means integration tests in `tests/` can drive the exact same
//! code paths the CLI uses, without spawning a process.

use std::fmt;
use std::path::PathBuf;

use clap::Parser;

/// Command-line arguments for `verirust`.
#[derive(Debug, Parser)]
#[command(
    name = "verirust",
    version,
    about = "Compile a Rust solution, run it against test cases, and emit a deterministic verdict."
)]
pub struct Args {
    /// Path to the Rust source file to verify.
    #[arg(long, value_name = "FILE")]
    pub source: PathBuf,

    /// Path to the file containing test cases.
    #[arg(long, value_name = "FILE")]
    pub tests: PathBuf,
}

/// Errors that occur before verification starts.
#[derive(Debug)]
pub enum Error {
    /// A path given on the command line does not point to a file.
    MissingFile { flag: &'static str, path: PathBuf },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingFile { flag, path } => {
                write!(f, "{flag} does not point to a file: {}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {}

/// Entry point shared by the CLI and, later, integration tests.
pub fn run(args: Args) -> Result<(), Error> {
    if !args.source.is_file() {
        return Err(Error::MissingFile {
            flag: "--source",
            path: args.source,
        });
    }
    if !args.tests.is_file() {
        return Err(Error::MissingFile {
            flag: "--tests",
            path: args.tests,
        });
    }

    // Compilation, execution, and verdict reporting arrive in later steps.
    println!("verirust {}", env!("CARGO_PKG_VERSION"));
    println!("  source: {}", args.source.display());
    println!("  tests:  {}", args.tests.display());
    println!("verification not yet implemented");

    Ok(())
}