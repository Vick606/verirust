//! VeriRust — deterministic compile, run, and verify for Rust submissions.
//!
//! The binary in `main.rs` is a thin wrapper around this library. Keeping the
//! logic here means integration tests in `tests/` can drive the exact same
//! code paths the CLI uses, without spawning a process.

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

pub mod compiler;
pub mod runner;
pub mod verifier;

pub use verifier::Verdict;

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

    /// Path to the file containing expected output.
    #[arg(long, value_name = "FILE")]
    pub tests: PathBuf,

    /// Timeout for the compiled binary, in seconds.
    #[arg(long, value_name = "SECONDS", default_value_t = 5)]
    pub timeout: u64,
}

/// Errors that occur before verification starts or that prevent it entirely.
///
/// A rejected submission is *not* an error — it is a successful
/// verification with a negative result, represented by `Verdict::Rejected`.
#[derive(Debug)]
pub enum Error {
    /// A path given on the command line does not point to a file.
    MissingFile { flag: &'static str, path: PathBuf },

    /// An I/O operation failed. `context` says which one.
    Io {
        context: &'static str,
        source: std::io::Error,
    },

    /// rustc ran and rejected the source, or could not be spawned.
    CompileFailed { stderr: String },

    /// The compiled binary exceeded the timeout.
    Timeout { seconds: u64 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingFile { flag, path } => {
                write!(f, "{flag} does not point to a file: {}", path.display())
            }
            Error::Io { context, source } => write!(f, "{context}: {source}"),
            Error::CompileFailed { stderr } => write!(f, "compilation failed:\n{stderr}"),
            Error::Timeout { seconds } => write!(f, "program exceeded {seconds}s timeout"),
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
///
/// Returns `Ok(Verdict::Rejected { .. })` for compile failures, timeouts,
/// non-zero exits, and output mismatches. Returns `Err` only when the
/// verifier itself could not run.
pub fn run(args: Args) -> Result<Verdict, Error> {
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

    let expected = std::fs::read_to_string(&args.tests).map_err(|e| Error::Io {
        context: "reading tests file",
        source: e,
    })?;

    let compiled = match compiler::compile(&args.source) {
        Ok(c) => c,
        Err(Error::CompileFailed { stderr }) => {
            return Ok(Verdict::Rejected {
                reason: format!("compilation failed:\n{stderr}"),
            });
        }
        Err(e) => return Err(e),
    };

    let output = match runner::run(compiled.binary(), Duration::from_secs(args.timeout)) {
        Ok(o) => o,
        Err(Error::Timeout { seconds }) => {
            return Ok(Verdict::Rejected {
                reason: format!("program exceeded {seconds}s timeout"),
            });
        }
        Err(e) => return Err(e),
    };

    Ok(verifier::verify(&output, &expected))
}