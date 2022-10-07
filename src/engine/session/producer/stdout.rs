use std::{
    io::{Read, Write},
    process::ChildStdout,
    sync::mpsc::Sender,
};

use crate::engine::session::event::{SessionEvent, SessionEventProduceError};

pub struct StdoutProducer(pub ChildStdout);

impl StdoutProducer {
    pub fn boot(mut self, sender: Sender<SessionEvent>) -> Result<(), SessionEventProduceError> {
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
