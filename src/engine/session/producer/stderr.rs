use std::{
    io::{Read, Write},
    process::ChildStderr,
    sync::mpsc::Sender,
};

use crate::engine::session::event::{SessionEvent, SessionEventProduceError};

pub struct StderrProducer(pub ChildStderr);

impl StderrProducer {
    pub fn boot(mut self, sender: Sender<SessionEvent>) -> Result<(), SessionEventProduceError> {
        // let mut stderr = std::io::stderr().lock();
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
