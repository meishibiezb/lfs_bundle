use anyhow::Result;
use lfs_bundle::core::archive::build_final_archive;
use lfs_bundle::core::import::{FailureInjection, import_archive_with_injection};
use lfs_bundle::core::manifest::{BundleManifest, FileEntry, write_manifest};
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
    assert_eq!(
        fs::read_to_string(target.join("file.txt"))?.replace("\r\n", "\n"),
        "second\n"
    );
    Ok(())
}

#[test]
fn import_archive_places_lfs_objects_under_dot_git_directory() -> Result<()> {
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

    let bundle_path = temp.path().join("bundle.bundle");
    run_git(
        &source,
        &[
            "bundle",
            "create",
            bundle_path.to_str().unwrap(),
            "HEAD",
            &format!("^{start}"),
        ],
    )?;

    let oid = "3e456df59e23bd6764f7ad5a842323842b9af1220e7ab8fcc11bfad83744174b";
    let lfs_object_relative = std::path::PathBuf::from(format!(
        ".git/lfs/objects/{}/{}/{}",
        &oid[0..2],
        &oid[2..4],
        oid
    ));
    let lfs_object_source = source.join(&lfs_object_relative);
    fs::create_dir_all(lfs_object_source.parent().unwrap())?;
    fs::write(&lfs_object_source, b"fake-lfs-object")?;

    let lfs_tar = temp.path().join("lfs.tar.gz");
    lfs_bundle::core::lfs::create_lfs_tar(&lfs_tar, &source, &[lfs_object_relative.clone()])?;

    let manifest_path = temp.path().join("manifest.json");
    let manifest = BundleManifest {
        tool_version: "0.1.0".into(),
        source_repo: "source".into(),
        start_commit: start,
        end_commit: end.clone(),
        target_commit: end,
        bundle: FileEntry {
            name: "bundle.bundle".into(),
            size_bytes: fs::metadata(&bundle_path)?.len(),
            sha256: "bundle".into(),
        },
        lfs: FileEntry {
            name: "lfs.tar.gz".into(),
            size_bytes: fs::metadata(&lfs_tar)?.len(),
            sha256: "lfs".into(),
        },
        lfs_object_count: 1,
        created_at: "2026-03-26T00:00:00Z".into(),
    };
    write_manifest(&manifest_path, &manifest)?;

    let archive = temp.path().join("package.zip");
    build_final_archive(&archive, &bundle_path, &lfs_tar, &manifest_path)?;

    let request = ImportRequest {
        repo_path: target.clone(),
        branch: "master".into(),
        archive_path: archive,
        safe_mode: true,
    };

    import_archive_with_injection(&request, FailureInjection::None)?;

    assert!(target.join(lfs_object_relative).exists());
    assert!(!target.join("lfs").exists());
    assert!(!target.join("objects").exists());
    assert!(!target.join(&oid[0..2]).exists());
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
