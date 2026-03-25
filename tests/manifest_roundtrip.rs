use lfs_bundle::core::manifest::{BundleManifest, FileEntry};
use lfs_bundle::core::models::PackageRequest;

#[test]
fn package_request_keeps_selected_commit_range() {
    let req = PackageRequest {
        repo_path: "D:/repo".into(),
        start_commit: "abc123".into(),
        end_commit: "def456".into(),
        output_archive: "D:/out/pkg.zip".into(),
        safe_mode: true,
    };

    assert_eq!(req.start_commit, "abc123");
    assert_eq!(req.end_commit, "def456");
}

#[test]
fn manifest_roundtrips_through_json() {
    let manifest = BundleManifest {
        tool_version: "0.1.0".into(),
        source_repo: "demo".into(),
        start_commit: "abc".into(),
        end_commit: "def".into(),
        target_commit: "def".into(),
        bundle: FileEntry { name: "bundle.bundle".into(), size_bytes: 10, sha256: "1".into() },
        lfs: FileEntry { name: "lfs.tar.gz".into(), size_bytes: 20, sha256: "2".into() },
        lfs_object_count: 1,
        created_at: "2026-03-25T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let reparsed: BundleManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(reparsed.end_commit, "def");
}
