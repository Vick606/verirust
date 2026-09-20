//! Runs a compiled binary and captures its output.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::Error;

/// Result of one run of a compiled binary.
#[derive(Debug)]
pub struct RunOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Run `binary` with no arguments and no stdin.
///
/// Kills the process and returns `Error::Timeout` if it runs longer than
/// `timeout`.
///
/// Known limitation: if the child writes more than the OS pipe buffer
/// (64 KiB on Linux) before exiting, it blocks on write and we time out.
/// Fine for small verification test cases; see README limitations.
pub fn run(binary: &Path, timeout: Duration) -> Result<RunOutput, Error> {
    let mut child = Command::new(binary)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Io {
            context: "spawning compiled binary",
            source: e,
        })?;

    let start = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| Error::Io {
            context: "waiting for compiled binary",
            source: e,
        })? {
            Some(status) => break status,
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Error::Timeout {
                    seconds: timeout.as_secs(),
                });
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    };

    let stdout = read_pipe(child.stdout.take(), "reading stdout")?;
    let stderr = read_pipe(child.stderr.take(), "reading stderr")?;

    Ok(RunOutput {
        stdout,
        stderr,
        exit_code: status.code(),
    })
}

fn read_pipe(pipe: Option<impl Read>, context: &'static str) -> Result<String, Error> {
    let mut bytes = Vec::new();
    if let Some(mut p) = pipe {
        p.read_to_end(&mut bytes)
            .map_err(|e| Error::Io { context, source: e })?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}