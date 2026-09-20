//! VeriRust — deterministic compile, run, and verify for Rust submissions.
//!
//! The binary in `main.rs` is a thin wrapper around this library. Keeping the
//! logic here means integration tests in `tests/` can drive the exact same
//! code paths the CLI uses, without spawning a process.

use std::fmt;
use std::path::PathBuf;

use clap::Parser;

pub mod compiler;

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

/// Errors that can occur while running the verifier.
#[derive(Debug)]
pub enum Error {
    /// A path given on the command line does not point to a file.
    MissingFile { flag: &'static str, path: PathBuf },

    /// An I/O operation failed. `context` says which one.
    Io {
        context: &'static str,
        source: std::io::Error,
    },

    /// rustc ran and rejected the source. `stderr` is its diagnostic output.
    CompileFailed { stderr: String },
}

impl Error {
    /// Process exit code for this error.
    ///
    /// 1 = compilation failed. In the final product this becomes a
    ///     `REJECTED` verdict with exit code 1, matching grep's convention.
    /// 2 = the verifier could not run at all (bad args, missing file,
    ///     toolchain unavailable). Matches clap's convention.
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::CompileFailed { .. } => 1,
            _ => 2,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingFile { flag, path } => {
                write!(f, "{flag} does not point to a file: {}", path.display())
            }
            Error::Io { context, source } => {
                write!(f, "{context}: {source}")
            }
            Error::CompileFailed { stderr } => {
                write!(f, "compilation failed:\n{stderr}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

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

    // Held for the duration of `run`. Dropping it deletes the temp directory.
    // Execution and verdict reporting arrive in the next step.
    let _compiled = compiler::compile(&args.source)?;
    println!("compiled successfully");

    Ok(())
}