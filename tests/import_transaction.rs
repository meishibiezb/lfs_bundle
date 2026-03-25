use anyhow::Result;
use lfs_bundle::core::import::{import_archive_with_injection, FailureInjection};
use lfs_bundle::core::models::{ImportRequest, PackageRequest};
use lfs_bundle::core::pack::package_repository;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn lfs_import_failure_restores_original_head() -> Result<()> {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    fs::create_dir_all(&source)?;

    run_git(&source, &["init", "."])?;
    run_git(&source, &["config", "user.name", "Test User"])?;
    run_git(&source, &["config", "user.email", "test@example.com"])?;

    fs::write(source.join("file.txt"), "first\n")?;
    run_git(&source, &["add", "."])?;
    run_git(&source, &["commit", "-m", "first"])?;
    let start = git_stdout(&source, &["rev-parse", "HEAD"])?;

    copy_dir_all(&source, &target)?;
    let original_head = git_stdout(&target, &["rev-parse", "master"])?;

    fs::write(source.join("file.txt"), "second\n")?;
    run_git(&source, &["add", "."])?;
    run_git(&source, &["commit", "-m", "second"])?;
    let end = git_stdout(&source, &["rev-parse", "HEAD"])?;

    let archive = temp.path().join("package.zip");
    package_repository(&PackageRequest {
        repo_path: source.clone(),
        start_commit: start,
        end_commit: end.clone(),
        output_archive: archive.clone(),
        safe_mode: true,
    })?;

    let request = ImportRequest {
        repo_path: target.clone(),
        branch: "master".into(),
        archive_path: archive,
        safe_mode: true,
    };

    let result = import_archive_with_injection(&request, FailureInjection::FailAfterBundleApplied);
    assert!(result.is_err());

    let rolled_back_head = git_stdout(&target, &["rev-parse", "master"])?;
    assert_eq!(rolled_back_head, original_head);
    Ok(())
}

#[test]
fn import_archive_updates_existing_branch_when_bundle_and_lfs_succeed() -> Result<()> {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    fs::create_dir_all(&source)?;

    run_git(&source, &["init", "."])?;
    run_git(&source, &["config", "user.name", "Test User"])?;
    run_git(&source, &["config", "user.email", "test@example.com"])?;

    fs::write(source.join("file.txt"), "first\n")?;
    run_git(&source, &["add", "."])?;
    run_git(&source, &["commit", "-m", "first"])?;
    let start = git_stdout(&source, &["rev-parse", "HEAD"])?;

    copy_dir_all(&source, &target)?;

    fs::write(source.join("file.txt"), "second\n")?;
    run_git(&source, &["add", "."])?;
    run_git(&source, &["commit", "-m", "second"])?;
    let end = git_stdout(&source, &["rev-parse", "HEAD"])?;

    let archive = temp.path().join("package.zip");
    package_repository(&PackageRequest {
        repo_path: source.clone(),
        start_commit: start,
        end_commit: end.clone(),
        output_archive: archive.clone(),
        safe_mode: true,
    })?;

    let request = ImportRequest {
        repo_path: target.clone(),
        branch: "master".into(),
        archive_path: archive,
        safe_mode: true,
    };

    import_archive_with_injection(&request, FailureInjection::None)?;

    let imported_head = git_stdout(&target, &["rev-parse", "master"])?;
    assert_eq!(imported_head, end);
    Ok(())
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
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
