use std::{path::PathBuf, process::{Command, Stdio}};

use thiserror::Error;

use super::Session;

pub struct SessionInit {
    pub shell: PathBuf,
}

#[derive(Debug, Error)]
pub enum SessionBootstrapError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
}

impl SessionInit {
    pub fn boot(self) -> Result<Session, SessionBootstrapError> {
        let child = Command::new(self.shell)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        Ok(Session { child })
    }
}
