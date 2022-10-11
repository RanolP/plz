use std::{
    io::{Read, Write},
    process::ChildStderr,
    sync::mpsc::Sender,
};

use nix::unistd;

use crate::engine::{
    faketty,
    session::{SessionError, SessionEvent},
};

pub struct StderrProducer;

impl StderrProducer {
    pub fn boot(mut self, sender: Sender<SessionEvent>) -> Result<(), SessionError> {
        let stderr = faketty::dup(2)?;
        let pty = unsafe { faketty::forkpty() }?;
        if let unistd::ForkResult::Parent { child } = pty.fork_result {
            faketty::copyfd(pty.master, stderr);
            faketty::copyexit(child);
        }
        let mut stderr = FromRaw
        let mut buffer = [0; 1024];
        loop {
            let read = self.0.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            sender.send(SessionEvent::Stderr(buffer[..read].to_vec()))?;
            // stderr.write(&buffer[..read])?;
        }
        Ok(())
    }
}
