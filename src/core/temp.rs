use anyhow::Result;
use tempfile::{Builder, TempDir};

pub struct SessionDir {
    pub root: TempDir,
}

impl SessionDir {
    pub fn path(&self) -> &std::path::Path {
        self.root.path()
    }
}

pub fn new_session_dir(prefix: &str) -> Result<SessionDir> {
    Ok(SessionDir {
        root: Builder::new().prefix(prefix).tempdir()?,
    })
}
