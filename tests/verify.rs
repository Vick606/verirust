//! End-to-end tests for the verirust library API.

use std::path::PathBuf;
use std::time::Duration;

use verirust::{Error, Verdict, compiler, runner, verifier};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn accepts_matching_fixture() {
    let compiled = compiler::compile(&fixture("hello.rs")).unwrap();
    let output = runner::run(compiled.binary(), Duration::from_secs(5)).unwrap();
    assert_eq!(
        verifier::verify(&output, "hello from verirust\n"),
        Verdict::Accepted
    );
}

#[test]
fn rejects_wrong_output() {
    let compiled = compiler::compile(&fixture("hello.rs")).unwrap();
    let output = runner::run(compiled.binary(), Duration::from_secs(5)).unwrap();
    assert!(matches!(
        verifier::verify(&output, "goodbye\n"),
        Verdict::Rejected { .. }
    ));
}

#[test]
fn rejects_compile_error() {
    assert!(matches!(
        compiler::compile(&fixture("syntax_error.rs")),
        Err(Error::CompileFailed { .. })
    ));
}

#[test]
fn rejects_nonzero_exit() {
    let compiled = compiler::compile(&fixture("nonzero.rs")).unwrap();
    let output = runner::run(compiled.binary(), Duration::from_secs(5)).unwrap();
    assert!(matches!(
        verifier::verify(&output, ""),
        Verdict::Rejected { .. }
    ));
}

#[test]
fn kills_hung_process() {
    let compiled = compiler::compile(&fixture("hang.rs")).unwrap();
    assert!(matches!(
        runner::run(compiled.binary(), Duration::from_millis(500)),
        Err(Error::Timeout { .. })
    ));
}
