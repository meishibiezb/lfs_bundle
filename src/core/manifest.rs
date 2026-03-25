use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleManifest {
    pub tool_version: String,
    pub source_repo: String,
    pub start_commit: String,
    pub end_commit: String,
    pub target_commit: String,
    pub bundle: FileEntry,
    pub lfs: FileEntry,
    pub lfs_object_count: usize,
    pub created_at: String,
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("failed to open file for hashing: {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];

    loop {
        let bytes = file.read(&mut buf)?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buf[..bytes]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn write_manifest(path: &Path, manifest: &BundleManifest) -> Result<()> {
    let json = serde_json::to_vec_pretty(manifest)?;
    fs::write(path, json).with_context(|| format!("failed to write manifest: {}", path.display()))?;
    Ok(())
}

pub fn read_manifest(path: &Path) -> Result<BundleManifest> {
    let bytes = fs::read(path).with_context(|| format!("failed to read manifest: {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes)?)
}
