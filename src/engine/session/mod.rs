use std::{
    io::Write,
    process::Child,
    sync::mpsc::{self, Receiver},
    thread::{scope, Builder},
};

use thiserror::Error;

pub use init::{SessionBootstrapError, SessionInit};

use self::{
    event::{SessionEvent, SessionEventProduceError},
    producer::{StderrProducer, StdinProducer, StdoutProducer},
};

mod event;
mod init;
mod producer;

pub struct Session {
    child: Child,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("An error occured when producing events: {0}")]
    SessionEvent(#[from] SessionEventProduceError),
}

impl Session {
    pub fn run(mut self) -> Result<(), SessionError> {
        let (tx, rx) = mpsc::channel();

        let stdin_producer = self.child.stdin.take().map(StdinProducer);
        let stdout_producer = self.child.stdout.take().map(StdoutProducer);
        let stderr_producer = self.child.stderr.take().map(StderrProducer);

        scope(|s| -> Result<_, SessionError> {
            let tx_stdin = tx.clone();
            let stdin_producer_handle = stdin_producer
                .map(move |stdin_producer| {
                    Builder::new()
                        .name("stdin-producer".to_string())
                        .spawn_scoped(s, move || stdin_producer.boot(tx_stdin))
                })
                .transpose()?;

            let tx_stdout = tx.clone();
            let stdout_producer_handle = stdout_producer
                .map(move |stdout_producer| {
                    Builder::new()
                        .name("stdout-producer".to_string())
                        .spawn_scoped(s, move || stdout_producer.boot(tx_stdout))
                })
                .transpose()?;
            let tx_stderr = tx.clone();
            let stderr_producer_handle = stderr_producer
                .map(move |stderr_producer| {
                    Builder::new()
                        .name("stderr-producer".to_string())
                        .spawn_scoped(s, move || stderr_producer.boot(tx_stderr))
                })
                .transpose()?;

            let stdio_loop = Builder::new()
                .name("stdio-loop".to_string())
                .spawn_scoped(s, move || Session::stdio_loop(rx))?;

            if let Some(stdin_producer_handle) = stdin_producer_handle {
                stdin_producer_handle.join().unwrap()?;
            }
            if let Some(stdout_producer_handle) = stdout_producer_handle {
                stdout_producer_handle.join().unwrap()?;
            }
            if let Some(stderr_producer_handle) = stderr_producer_handle {
                stderr_producer_handle.join().unwrap()?;
            }
            stdio_loop.join().unwrap()?;
            Ok(())
        })?;

        Ok(())
    }

    fn stdio_loop(rx: Receiver<SessionEvent>) -> Result<(), SessionError> {
        let mut stdout = std::io::stdout().lock();
        let mut stderr = std::io::stderr().lock();

        let mut prompt: Option<Vec<u8>> = None;
        let mut buffer: Vec<u8> = Vec::new();

        while let Ok(e) = rx.recv() {
            match e {
                SessionEvent::Stdout(data) => {
                    if let Some(prompt) = &prompt {
                        if &data == prompt {
                            write!(stdout, ":::debug\n{:?}\n:::\n\n", buffer)?;
                            buffer.clear();
                        } else {
                            buffer.extend(&data);
                        }
                    } else {
                        prompt = Some(data.to_vec());
                    }
                    stdout.write(&data)?;
                    stdout.flush()?;
                }
                SessionEvent::Stderr(data) => {
                    stderr.write(&data)?;
                    stderr.flush()?;
                }
                SessionEvent::Stdin(data) => {}
            }
        }
        Ok(())
    }
}
