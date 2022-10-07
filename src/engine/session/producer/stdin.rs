use std::{
    io::{stdin, Read, Write},
    process::ChildStdin,
    sync::mpsc::Sender,
};

use crate::engine::session::event::{SessionEvent, SessionEventProduceError};

pub struct StdinProducer(pub ChildStdin);

impl StdinProducer {
    pub fn boot(mut self, sender: Sender<SessionEvent>) -> Result<(), SessionEventProduceError> {
        let mut stdin = stdin().lock();
        let mut buffer = [0; 1024];
        loop {
            let read = stdin.read(&mut buffer)?;
            sender.send(SessionEvent::Stdin(buffer[..read].to_vec()))?;
            self.0.write(&buffer[..read])?;
        }
    }
}
