use clap::Parser;
use miette::IntoDiagnostic;
use plz::PlzArgument;

fn main() -> miette::Result<()> {
    let argument = PlzArgument::parse();
    match argument {
        PlzArgument::Boot(boot) => {
            let session = plz::engine::SessionInit { shell: boot.shell }
                .boot()
                .into_diagnostic()?;
            session.run().into_diagnostic()?;
        }
    }
    Ok(())
}
