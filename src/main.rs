use std::process::ExitCode;

use clap::Parser;
use miette::IntoDiagnostic;
use plz::PlzArgument;

fn main() -> miette::Result<ExitCode> {
    let argument = PlzArgument::parse();
    match argument {
        PlzArgument::Boot(boot) => {
            let session = plz::engine::SessionInit { shell: boot.shell }
                .boot()
                .into_diagnostic()?;
            let status = session.run().into_diagnostic()?;
            match status.code() {
                Some(code) => Ok(ExitCode::from(u8::try_from(code).into_diagnostic()?)),
                None => Ok(ExitCode::FAILURE),
            }
        }
        _ => Ok(ExitCode::SUCCESS),
    }
}
