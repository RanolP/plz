use std::{
    io::{self, Read, Write},
    process::ChildStdin,
    sync::mpsc::Sender,
};

use nix::unistd;

use crate::engine::{
    faketty,
    session::{SessionError, SessionEvent},
};

pub struct StdinProducer(pub ChildStdin);

impl StdinProducer {
    pub fn boot(mut self, sender: Sender<SessionEvent>) -> Result<(), SessionError> {
        let stdin = faketty::dup(0)?;
        unistd::dup2(stdin, 0)?;
        let pty = unsafe { faketty::forkpty() }?;
        if let unistd::ForkResult::Parent { child } = pty.fork_result {
            faketty::copyfd(pty.master, 1);
            faketty::copyexit(child);
        }

        let mut stdin = io::stdin().lock();
        let mut buffer = [0; 1024];
        loop {
            let read = stdin.read(&mut buffer)?;
            if buffer[..read].iter().any(|byte| *byte == b'\n') {
                sender.send(SessionEvent::Enter)?;
            }
            self.0.write(&buffer[..read])?;
        }
    }
}
