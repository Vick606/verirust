use std::process::ExitCode;

use clap::Parser;

use verirust::{Args, Verdict};

fn main() -> ExitCode {
    let args = Args::parse();

    match verirust::run(args) {
        Ok(Verdict::Accepted) => {
            println!("VERIFY_RESULT: ACCEPTED");
            ExitCode::SUCCESS
        }
        Ok(Verdict::Rejected { reason }) => {
            eprintln!("{reason}");
            println!("VERIFY_RESULT: REJECTED");
            ExitCode::from(1)
        }
        Err(err) => {
            eprintln!("verirust: {err}");
            ExitCode::from(2)
        }
    }
}