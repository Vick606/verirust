//! Compiles a Rust source file into a standalone binary in a temp directory.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use crate::Error;

/// A successfully compiled program.
///
/// The temp directory holding the binary is kept alive by this value.
/// Dropping it deletes the directory and the binary inside.
pub struct Compiled {
    // Underscore prevents an "unused field" warning. The field exists
    // only so its Drop impl runs when `Compiled` drops.
    _dir: TempDir,
    binary: PathBuf,
}

impl Compiled {
    /// Path to the compiled binary. Valid for as long as `self` is alive.
    pub fn binary(&self) -> &Path {
        &self.binary
    }
}

/// Compile `source` with `rustc`, writing the binary into a fresh temp directory.
///
/// Returns `Error::CompileFailed` with rustc's stderr if the source does not
/// compile. Returns `Error::Io` if the temp directory cannot be created or
/// `rustc` cannot be spawned.
pub fn compile(source: &Path) -> Result<Compiled, Error> {
    let dir = TempDir::new().map_err(|e| Error::Io {
        context: "creating temp directory",
        source: e,
    })?;

    let binary = dir.path().join("program");

    let output = Command::new("rustc")
        .arg("--edition")
        .arg("2024")
        .arg(source)
        .arg("-o")
        .arg(&binary)
        .output()
        .map_err(|e| Error::Io {
            context: "spawning rustc",
            source: e,
        })?;

    if !output.status.success() {
        return Err(Error::CompileFailed {
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    Ok(Compiled {
        _dir: dir,
        binary,
    })
}