use std::path::PathBuf;

use super::{Session, SessionError};

pub struct SessionInit {
    pub shell: PathBuf,
}

impl SessionInit {
    pub fn boot(self) -> Result<Session, SessionError> {
        Ok(Session { shell: self.shell })
    }
}
