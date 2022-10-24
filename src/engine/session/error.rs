use std::{io, sync::mpsc::SendError};

use thiserror::Error;

use super::event::SessionEvent;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),

    #[error("*nix api call error: {0}")]
    Nix(#[from] nix::Error),

    #[error("Send error: {0}")]
    Send(#[from] SendError<SessionEvent>),

    #[error("Found NUL from path: {0}")]
    PathNul(#[from] std::ffi::NulError),

    #[error("Failed to spawn child")]
    SpawnChild,
}
