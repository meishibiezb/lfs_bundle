use crate::core::archive::build_final_archive;
use crate::core::lfs::{create_lfs_tar, parse_lfs_output};
use crate::core::manifest::{sha256_file, write_manifest, BundleManifest, FileEntry};
use crate::core::models::{PackageRequest, PackageSummary};
use crate::core::process::run_command;
use crate::core::repo::is_git_repo_path;
use crate::core::temp::new_session_dir;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn package_repository(req: &PackageRequest) -> Result<PackageSummary> {
    let repo_path = &req.repo_path;
    if !is_git_repo_path(repo_path) {
        anyhow::bail!("not a git repository: {}", repo_path.display());
    }

    if let Some(parent) = req.output_archive.parent() {
        fs::create_dir_all(parent)?;
    }

    let session = new_session_dir("lfs-bundle-")?;
    let bundle_path = session.path().join("bundle.bundle");
    let lfs_path = session.path().join("lfs.tar.gz");
    let manifest_path = session.path().join("manifest.json");

    let temp_ref = format!("refs/lfs-bundle-temp/{}", timestamp_string());
    run_command("git", &["update-ref", temp_ref.as_str(), req.end_commit.as_str()], Some(repo_path))?;

    let bundle_result = run_command(
        "git",
        &[
            "bundle",
            "create",
            bundle_path.to_str().context("invalid bundle path")?,
            temp_ref.as_str(),
            &format!("^{}", req.start_commit),
        ],
        Some(repo_path),
    );

    let delete_ref_result = run_command("git", &["update-ref", "-d", temp_ref.as_str()], Some(repo_path));

    bundle_result?;
    delete_ref_result?;

    let lfs_relative_paths = match run_command(
        "git",
        &["lfs", "ls-files", "--long", req.start_commit.as_str(), req.end_commit.as_str()],
        Some(repo_path),
    ) {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).context("git lfs output was not utf-8")?;
            parse_lfs_output(&stdout)
        }
        Err(_) => Vec::new(),
    };

    create_lfs_tar(&lfs_path, repo_path, &lfs_relative_paths)?;

    let range = format!("{}..{}", req.start_commit, req.end_commit);
    let commit_count = count_commits_in_range(repo_path, &range)?;
    let manifest = BundleManifest {
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        source_repo: repo_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("repository")
            .to_string(),
        start_commit: req.start_commit.clone(),
        end_commit: req.end_commit.clone(),
        target_commit: req.end_commit.clone(),
        bundle: file_entry("bundle.bundle", &bundle_path)?,
        lfs: file_entry("lfs.tar.gz", &lfs_path)?,
        lfs_object_count: lfs_relative_paths.len(),
        created_at: timestamp_string(),
    };
    write_manifest(&manifest_path, &manifest)?;

    build_final_archive(&req.output_archive, &bundle_path, &lfs_path, &manifest_path)?;

    Ok(PackageSummary {
        commit_count,
        lfs_object_count: lfs_relative_paths.len(),
        bundle_name: "bundle.bundle".into(),
        lfs_name: "lfs.tar.gz".into(),
    })
}

fn count_commits_in_range(repo: &Path, range: &str) -> Result<usize> {
    let output = run_command("git", &["rev-list", "--count", range], Some(repo))?;
    let count = String::from_utf8(output.stdout)
        .context("git rev-list output was not utf-8")?
        .trim()
        .parse()?;
    Ok(count)
}

fn file_entry(name: &str, path: &Path) -> Result<FileEntry> {
    let metadata = fs::metadata(path)?;
    Ok(FileEntry {
        name: name.to_string(),
        size_bytes: metadata.len(),
        sha256: sha256_file(path)?,
    })
}

fn timestamp_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("{seconds}")
}
