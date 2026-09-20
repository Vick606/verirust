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