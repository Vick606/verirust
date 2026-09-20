use std::process::ExitCode;

use clap::Parser;

use verirust::Args;

fn main() -> ExitCode {
    let args = Args::parse();

    match verirust::run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("verirust: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}