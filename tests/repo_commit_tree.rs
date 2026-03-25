use anyhow::Result;
use lfs_bundle::core::repo::{is_valid_commit_range, load_branch_commit_tree};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn load_branch_commit_tree_returns_current_branch_commits() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo)?;

    run_git(&repo, &["init", "."])?;
    run_git(&repo, &["config", "user.name", "Test User"])?;
    run_git(&repo, &["config", "user.email", "test@example.com"])?;

    fs::write(repo.join("file.txt"), "first\n")?;
    run_git(&repo, &["add", "."])?;
    run_git(&repo, &["commit", "-m", "first"])?;

    fs::write(repo.join("file.txt"), "second\n")?;
    run_git(&repo, &["add", "."])?;
    run_git(&repo, &["commit", "-m", "second"])?;

    let nodes = load_branch_commit_tree(&repo, "master", 50)?;
    assert!(nodes.len() >= 2);
    assert!(!nodes[0].id.is_empty());
    assert!(!nodes[0].summary.is_empty());

    Ok(())
}

#[test]
fn commit_range_validation_detects_invalid_order() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo)?;

    run_git(&repo, &["init", "."])?;
    run_git(&repo, &["config", "user.name", "Test User"])?;
    run_git(&repo, &["config", "user.email", "test@example.com"])?;

    fs::write(repo.join("file.txt"), "first\n")?;
    run_git(&repo, &["add", "."])?;
    run_git(&repo, &["commit", "-m", "first"])?;
    let start = git_stdout(&repo, &["rev-parse", "HEAD"])?;

    fs::write(repo.join("file.txt"), "second\n")?;
    run_git(&repo, &["add", "."])?;
    run_git(&repo, &["commit", "-m", "second"])?;
    let end = git_stdout(&repo, &["rev-parse", "HEAD"])?;

    assert!(is_valid_commit_range(&repo, &start, &end)?);
    assert!(!is_valid_commit_range(&repo, &end, &start)?);

    Ok(())
}

fn run_git(repo: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git").args(args).current_dir(repo).status()?;
    anyhow::ensure!(status.success(), "git command failed: {:?}", args);
    Ok(())
}

fn git_stdout(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    anyhow::ensure!(output.status.success(), "git command failed: {:?}", args);
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
