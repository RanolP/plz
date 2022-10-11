use std::{
    ffi::{CString, NulError},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
    process::{Command, Stdio},
};

use nix::unistd;
use thiserror::Error;

use super::Session;

pub struct SessionInit {}

#[derive(Debug, Error)]
pub enum SessionBootstrapError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("*nix api call error: {0}")]
    Nix(#[from] nix::Error),

    #[error("NULL from shell path")]
    Nul(#[from] NulError),
}

impl SessionInit {
    pub fn boot(self) -> Result<(), SessionBootstrapError> {
        let args: [CString; 0] = [];
        unistd::execvp(
            &CString::new(self.shell.as_os_str().as_bytes().to_vec())?,
            &args,
        )?;
        Ok(())
    }
}
