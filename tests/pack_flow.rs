use anyhow::Result;
use lfs_bundle::core::archive::extract_archive;
use lfs_bundle::core::models::PackageRequest;
use lfs_bundle::core::pack::package_repository;
use lfs_bundle::core::repo::split_short_id;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn split_short_id_truncates_long_commit_ids() {
    assert_eq!(split_short_id("1234567890abcdef"), "12345678");
}

#[test]
fn package_workflow_creates_bundle_lfs_and_manifest_archive() -> Result<()> {
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

    let output_archive = temp.path().join("package.zip");
    let request = PackageRequest {
        repo_path: repo.clone(),
        start_commit: start,
        end_commit: end,
        output_archive: output_archive.clone(),
        safe_mode: true,
    };

    let _summary = package_repository(&request)?;
    assert!(output_archive.exists());

    let extracted = temp.path().join("extracted");
    extract_archive(&output_archive, &extracted)?;

    assert!(extracted.join("bundle.bundle").exists());
    assert!(extracted.join("lfs.tar.gz").exists());
    assert!(extracted.join("manifest.json").exists());

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
