use crate::core::archive::extract_archive;
use crate::core::manifest::read_manifest;
use crate::core::models::ImportRequest;
use crate::core::process::run_command;
use crate::core::repo::{current_head, is_git_repo_path};
use crate::core::temp::new_session_dir;
use crate::core::transaction::ImportTransaction;
use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureInjection {
    None,
    FailAfterBundleApplied,
}

pub fn import_archive(req: &ImportRequest) -> Result<()> {
    import_archive_with_injection(req, FailureInjection::None)
}

pub fn import_archive_with_injection(
    req: &ImportRequest,
    injection: FailureInjection,
) -> Result<()> {
    if !is_git_repo_path(&req.repo_path) {
        anyhow::bail!("not a git repository: {}", req.repo_path.display());
    }
    if !req.archive_path.exists() {
        anyhow::bail!("archive does not exist: {}", req.archive_path.display());
    }

    let session = new_session_dir("lfs-import-")?;
    let extracted = session.path().join("archive");
    extract_archive(&req.archive_path, &extracted)?;

    let bundle_path = extracted.join("bundle.bundle");
    let lfs_path = extracted.join("lfs.tar.gz");
    let manifest_path = extracted.join("manifest.json");
    let _manifest = read_manifest(&manifest_path)?;

    let transaction = ImportTransaction::begin(req.repo_path.clone(), req.branch.clone())?;
    let temp_ref = format!("refs/lfs-bundle-import/{}", unique_suffix());

    let target_commit =
        match apply_bundle_to_branch(&req.repo_path, &req.branch, &bundle_path, &temp_ref) {
            Ok(commit) => commit,
            Err(err) => {
                let _ = delete_ref(&req.repo_path, &temp_ref);
                return Err(err);
            }
        };

    if injection == FailureInjection::FailAfterBundleApplied {
        let rollback_result = transaction.rollback();
        let _ = delete_ref(&req.repo_path, &temp_ref);
        rollback_result?;
        anyhow::bail!("injected failure after bundle application");
    }

    if let Err(err) = import_lfs_objects(&req.repo_path, &lfs_path) {
        let rollback_result = transaction.rollback();
        let _ = delete_ref(&req.repo_path, &temp_ref);
        rollback_result?;
        return Err(err);
    }

    sync_checked_out_branch_worktree(&req.repo_path, &req.branch, &target_commit)?;
    delete_ref(&req.repo_path, &temp_ref)?;
    Ok(())
}

fn apply_bundle_to_branch(
    repo_path: &Path,
    branch: &str,
    bundle_path: &Path,
    temp_ref: &str,
) -> Result<String> {
    let bundle_ref = bundle_head_ref(bundle_path)?;
    let fetch_spec = format!("{}:{}", bundle_ref, temp_ref);
    run_command(
        "git",
        &[
            "fetch",
            bundle_path.to_str().context("invalid bundle path")?,
            fetch_spec.as_str(),
        ],
        Some(repo_path),
    )?;

    let target_commit = current_head(repo_path, temp_ref)?;
    let original_head = current_head(repo_path, branch)?;

    ensure_fast_forward(repo_path, &original_head, &target_commit)?;

    let branch_ref = format!("refs/heads/{}", branch);
    run_command(
        "git",
        &[
            "update-ref",
            branch_ref.as_str(),
            target_commit.as_str(),
            original_head.as_str(),
        ],
        Some(repo_path),
    )?;

    Ok(target_commit)
}

fn ensure_fast_forward(repo_path: &Path, original_head: &str, target_commit: &str) -> Result<()> {
    run_command(
        "git",
        &["merge-base", "--is-ancestor", original_head, target_commit],
        Some(repo_path),
    )?;
    Ok(())
}

fn bundle_head_ref(bundle_path: &Path) -> Result<String> {
    let output = run_command(
        "git",
        &[
            "bundle",
            "list-heads",
            bundle_path.to_str().context("invalid bundle path")?,
        ],
        None,
    )?;
    let stdout = String::from_utf8(output.stdout).context("bundle head output was not utf-8")?;
    let first_line = stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .context("bundle did not contain any heads")?;
    let mut parts = first_line.split_whitespace();
    let _sha = parts.next().context("bundle head missing sha")?;
    Ok(parts.next().unwrap_or("HEAD").to_string())
}

fn import_lfs_objects(repo_path: &Path, lfs_archive: &Path) -> Result<()> {
    let session = new_session_dir("lfs-objects-")?;
    let decoder = GzDecoder::new(fs::File::open(lfs_archive)?);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(session.path())?;

    copy_tree(session.path(), repo_path)?;
    Ok(())
}

fn copy_tree(from: &Path, into_repo: &Path) -> Result<()> {
    copy_tree_from_root(from, from, into_repo)
}

fn copy_tree_from_root(root: &Path, current: &Path, into_repo: &Path) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root)?;
        let target = into_repo.join(relative);
        if path.is_dir() {
            fs::create_dir_all(&target)?;
            copy_tree_from_root(root, &path, into_repo)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn delete_ref(repo_path: &Path, temp_ref: &str) -> Result<()> {
    run_command("git", &["update-ref", "-d", temp_ref], Some(repo_path))?;
    Ok(())
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn sync_checked_out_branch_worktree(
    repo_path: &Path,
    branch: &str,
    target_commit: &str,
) -> Result<()> {
    if !is_branch_checked_out(repo_path, branch)? {
        return Ok(());
    }

    run_command("git", &["reset", "--hard", target_commit], Some(repo_path))?;
    Ok(())
}

fn is_branch_checked_out(repo_path: &Path, branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
        .current_dir(repo_path)
        .output()
        .context("failed to run git symbolic-ref --short HEAD")?;

    match output.status.code() {
        Some(0) => {
            let current = String::from_utf8(output.stdout)
                .context("git symbolic-ref output was not utf-8")?
                .trim()
                .to_string();
            Ok(current == branch)
        }
        Some(1) => Ok(false),
        Some(other) => bail!(
            "git symbolic-ref failed with status {other}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        None => bail!("git symbolic-ref terminated without an exit code"),
    }
}
