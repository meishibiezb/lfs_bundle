use crate::core::models::{BranchInfo, CommitInfo};
use crate::core::process::run_command;
use anyhow::{Context, Result};
use std::path::Path;

pub fn is_git_repo_path(path: &Path) -> bool {
    path.join(".git").exists()
}

pub fn split_short_id(commit: &str) -> String {
    commit.chars().take(8).collect()
}

pub fn load_commits(repo: &Path, limit: usize) -> Result<Vec<CommitInfo>> {
    let limit_string = limit.to_string();
    let output = run_command(
        "git",
        &[
            "log",
            "--max-count",
            limit_string.as_str(),
            "--pretty=format:%H%x1f%an%x1f%ad%x1f%s",
            "--date=iso-strict",
        ],
        Some(repo),
    )?;

    let stdout = String::from_utf8(output.stdout).context("git log output was not valid utf-8")?;

    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            let id = parts.next()?.to_string();
            let author = parts.next()?.to_string();
            let timestamp = parts.next()?.to_string();
            let summary = parts.next()?.to_string();
            Some(CommitInfo {
                short_id: split_short_id(&id),
                id,
                summary,
                author,
                timestamp,
            })
        })
        .collect())
}

pub fn load_branches(repo: &Path) -> Result<Vec<BranchInfo>> {
    let output = run_command(
        "git",
        &["for-each-ref", "--format=%(refname:short)%x1f%(objectname)", "refs/heads"],
        Some(repo),
    )?;
    let stdout = String::from_utf8(output.stdout).context("git branch output was not valid utf-8")?;

    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            Some(BranchInfo {
                name: parts.next()?.to_string(),
                head: parts.next()?.to_string(),
            })
        })
        .collect())
}

pub fn current_head(repo: &Path, branch: &str) -> Result<String> {
    let output = run_command("git", &["rev-parse", branch], Some(repo))?;
    Ok(String::from_utf8(output.stdout)
        .context("git rev-parse output was not valid utf-8")?
        .trim()
        .to_string())
}
