//! Compares program output to expected output and produces a verdict.

use crate::runner::RunOutput;

/// Outcome of verifying a submission.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    Rejected { reason: String },
}

/// Compare `actual` to `expected`.
///
/// The submission is accepted iff it exits with code 0 and its stdout
/// matches `expected` after stripping trailing newlines from both sides.
/// The newline tolerance handles the common case where an editor strips
/// a trailing newline from the expected-output file.
pub fn verify(actual: &RunOutput, expected: &str) -> Verdict {
    if actual.exit_code != Some(0) {
        let code = match actual.exit_code {
            Some(c) => format!("code {c}"),
            None => "unknown (signal)".to_string(),
        };
        return Verdict::Rejected {
            reason: format!("program exited with {code}"),
        };
    }

    let actual_trimmed = actual.stdout.trim_end_matches(['\n', '\r']);
    let expected_trimmed = expected.trim_end_matches(['\n', '\r']);

    if actual_trimmed == expected_trimmed {
        Verdict::Accepted
    } else {
        Verdict::Rejected {
            reason: format!(
                "stdout mismatch\nexpected:\n{expected_trimmed}\nactual:\n{actual_trimmed}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out(stdout: &str, exit_code: Option<i32>) -> RunOutput {
        RunOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code,
        }
    }

    #[test]
    fn accepts_matching_output() {
        assert_eq!(verify(&out("hello\n", Some(0)), "hello\n"), Verdict::Accepted);
    }

    #[test]
    fn accepts_trailing_newline_difference() {
        assert_eq!(verify(&out("hello\n", Some(0)), "hello"), Verdict::Accepted);
    }

    #[test]
    fn rejects_stdout_mismatch() {
        match verify(&out("goodbye\n", Some(0)), "hello\n") {
            Verdict::Rejected { reason } => assert!(reason.contains("mismatch")),
            Verdict::Accepted => panic!("expected rejection"),
        }
    }

    #[test]
    fn rejects_nonzero_exit() {
        match verify(&out("hello\n", Some(1)), "hello\n") {
            Verdict::Rejected { reason } => assert!(reason.contains("code 1")),
            Verdict::Accepted => panic!("expected rejection"),
        }
    }

    #[test]
    fn rejects_signal_terminated() {
        match verify(&out("", None), "") {
            Verdict::Rejected { reason } => assert!(reason.contains("signal")),
            Verdict::Accepted => panic!("expected rejection"),
        }
    }
}