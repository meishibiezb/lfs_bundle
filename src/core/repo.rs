use crate::core::models::{BranchInfo, CommitInfo, CommitTreeNode};
use crate::core::process::run_command;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

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

pub fn load_branch_commit_tree(repo: &Path, branch: &str, limit: usize) -> Result<Vec<CommitTreeNode>> {
    let limit_string = limit.to_string();
    let output = run_command(
        "git",
        &[
            "log",
            branch,
            "--max-count",
            limit_string.as_str(),
            "--pretty=format:%H%x1f%P%x1f%an%x1f%ad%x1f%s",
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
            let parent_text = parts.next()?.to_string();
            let author = parts.next()?.to_string();
            let timestamp = parts.next()?.to_string();
            let summary = parts.next()?.to_string();

            let parents = if parent_text.trim().is_empty() {
                Vec::new()
            } else {
                parent_text
                    .split_whitespace()
                    .map(|parent| parent.to_string())
                    .collect()
            };

            Some(CommitTreeNode {
                short_id: split_short_id(&id),
                id,
                summary,
                author,
                timestamp,
                parents,
                graph_prefix: "*".to_string(),
            })
        })
        .collect())
}

pub fn is_valid_commit_range(repo: &Path, start: &str, end: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", start, end])
        .current_dir(repo)
        .status()
        .context("failed to run git merge-base --is-ancestor")?;

    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(other) => bail!("git merge-base returned unexpected status code: {other}"),
        None => bail!("git merge-base terminated without an exit code"),
    }
}

pub fn load_branches(repo: &Path) -> Result<Vec<BranchInfo>> {
    let output = run_command(
        "git",
        &[
            "for-each-ref",
            "--format=%(refname:short)%x1f%(objectname)",
            "refs/heads",
        ],
        Some(repo),
    )?;
    let stdout =
        String::from_utf8(output.stdout).context("git branch output was not valid utf-8")?;

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
