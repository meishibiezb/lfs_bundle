use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageRequest {
    pub repo_path: PathBuf,
    pub start_commit: String,
    pub end_commit: String,
    pub output_archive: PathBuf,
    pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportRequest {
    pub repo_path: PathBuf,
    pub branch: String,
    pub archive_path: PathBuf,
    pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub summary: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitTreeNode {
    pub id: String,
    pub short_id: String,
    pub summary: String,
    pub author: String,
    pub timestamp: String,
    pub parents: Vec<String>,
    pub graph_prefix: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchInfo {
    pub name: String,
    pub head: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageSummary {
    pub commit_count: usize,
    pub lfs_object_count: usize,
    pub bundle_name: String,
    pub lfs_name: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunMode {
    Cli,
    Gui,
}
