use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use thiserror::Error;

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

pub struct Session {
    child: Child,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
}

impl Session {
    pub fn run(mut self) -> Result<(), SessionError> {
        let mut child_stdout = self.child.stdout.take().unwrap();
        let mut stdout = std::io::stdout();

        let mut buf = [0; 1024];
        while let Ok(read) = child_stdout.read(&mut buf) {
            if read == 0 {
                break;
            }
            stdout.write(&buf[..read])?;
            stdout.flush()?;
        }
        Ok(())
    }
}
