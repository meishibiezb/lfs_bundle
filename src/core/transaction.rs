use crate::core::process::run_command;
use crate::core::repo::current_head;
use anyhow::Result;
use std::path::PathBuf;

pub struct ImportTransaction {
    pub original_head: String,
    pub branch: String,
    pub repo_path: PathBuf,
}

impl ImportTransaction {
    pub fn begin(repo_path: PathBuf, branch: String) -> Result<Self> {
        let original_head = current_head(&repo_path, &branch)?;
        Ok(Self {
            original_head,
            branch,
            repo_path,
        })
    }

    pub fn rollback(&self) -> Result<()> {
        let branch_ref = format!("refs/heads/{}", self.branch);
        run_command(
            "git",
            &[
                "update-ref",
                branch_ref.as_str(),
                self.original_head.as_str(),
            ],
            Some(&self.repo_path),
        )?;
        Ok(())
    }
}
