use std::{io, sync::mpsc::SendError};

use thiserror::Error;

#[derive(Debug)]
pub enum SessionEvent {
    Stdin(Vec<u8>),
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

#[derive(Debug, Error)]
pub enum SessionEventProduceError {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Send error: {0}")]
    Send(#[from] SendError<SessionEvent>)
}
