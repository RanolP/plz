use std::path::PathBuf;

use clap::Parser;

/// Suggest the command you may want, for ergonomics
#[derive(Debug, Parser)]
pub enum PlzArgument {
    Boot(BootArgument),
}

/// Boot the plz session with shell provided
#[derive(Debug, Parser)]
pub struct BootArgument {
    /// The shell file
    pub shell: PathBuf,
}
