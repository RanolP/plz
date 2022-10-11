use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Child, ExitStatus},
    sync::mpsc::{self, Receiver},
    thread::{scope, Builder},
};

pub use error::SessionError;
pub use init::{SessionBootstrapError, SessionInit};

use self::{
    event::SessionEvent,
    producer::{StderrProducer, StdinProducer, StdoutProducer},
};

mod error;
mod event;
mod init;
mod producer;

pub struct Session {
    pub shell: PathBuf,
}

impl Session {
    pub fn run(mut self) -> Result<ExitStatus, SessionError> {
        let (tx, rx) = mpsc::channel();

        let stdin_producer = self.child.stdin.take().map(StdinProducer);
        let stdout_producer = self.child.stdout.take().map(StdoutProducer);
        let stderr_producer = self.child.stderr.take().map(StderrProducer);

        let exit_status = scope(|s| -> Result<_, SessionError> {
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

            let tx_check_die = tx.clone();
            let check_die = Builder::new().name("check-die".to_string()).spawn_scoped(
                s,
                move || -> Result<_, SessionError> {
                    let exit_status = self.child.wait()?;
                    tx_check_die
                        .send(SessionEvent::Die)
                        .map_err(SessionError::Send)?;
                    Ok(exit_status)
                },
            )?;

            if let Some(stdin_producer_handle) = stdin_producer_handle {
                stdin_producer_handle.join().unwrap()?;
            }
            if let Some(stdout_producer_handle) = stdout_producer_handle {
                stdout_producer_handle.join().unwrap()?;
            }
            if let Some(stderr_producer_handle) = stderr_producer_handle {
                stderr_producer_handle.join().unwrap()?;
            }
            let exit_status = check_die.join().unwrap()?;
            stdio_loop.join().unwrap()?;

            Ok(exit_status)
        })?;

        Ok(exit_status)
    }

    fn stdio_loop(rx: Receiver<SessionEvent>) -> Result<(), SessionError> {
        let mut stdout = std::io::stdout().lock();
        let mut stderr = std::io::stderr().lock();

        let mut buffer: Vec<u8> = Vec::new();

        let mut file = File::create("debug.log")?;

        while let Ok(e) = rx.recv() {
            match e {
                SessionEvent::Stdout(data) => {
                    buffer.extend(&data);
                    stdout.write(&data)?;
                    stdout.flush()?;
                }
                SessionEvent::Stderr(data) => {
                    buffer.extend(&data);
                    stderr.write(&data)?;
                    stderr.flush()?;
                }
                SessionEvent::Enter => {
                    file.write(b":::debug (buffer)\n")?;
                    file.write(&buffer)?;
                    buffer.clear();
                    file.write(b"\n:::\n\n")?;
                }
                SessionEvent::Die => break,
            }
        }
        Ok(())
    }
}
