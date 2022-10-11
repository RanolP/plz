use std::{io::Read, process::ChildStdout, sync::mpsc::Sender};

use nix::unistd;

use crate::engine::{
    faketty,
    session::{SessionError, SessionEvent},
};

pub struct StdoutProducer(pub ChildStdout);

impl StdoutProducer {
    pub fn boot(mut self, sender: Sender<SessionEvent>) -> Result<(), SessionError> {
        let stdout = faketty::dup(1)?;
        unistd::dup2(stdout, 1)?;

        // let mut stderr = std::io::stderr().lock();
        let mut buffer = [0; 1024];
        loop {
            let read = self.0.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            sender.send(SessionEvent::Stdout(buffer[..read].to_vec()))?;
            // stderr.write(&buffer[..read])?;
        }
        Ok(())
    }
}
