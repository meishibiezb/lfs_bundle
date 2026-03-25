use lfs_bundle::core::archive::{build_final_archive, inspect_archive};
use lfs_bundle::core::manifest::{write_manifest, BundleManifest, FileEntry};
use lfs_bundle::core::models::PackageRequest;
use tempfile::tempdir;

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

#[test]
fn extracted_archive_manifest_matches_expected_commits() {
    let temp = tempdir().unwrap();
    let bundle = temp.path().join("bundle.bundle");
    let lfs = temp.path().join("lfs.tar.gz");
    let manifest_path = temp.path().join("manifest.json");
    let archive = temp.path().join("archive.zip");

    std::fs::write(&bundle, b"bundle").unwrap();
    std::fs::write(&lfs, b"lfs").unwrap();

    let manifest = BundleManifest {
        tool_version: "0.1.0".into(),
        source_repo: "demo".into(),
        start_commit: "abc".into(),
        end_commit: "def".into(),
        target_commit: "def".into(),
        bundle: FileEntry { name: "bundle.bundle".into(), size_bytes: 6, sha256: "1".into() },
        lfs: FileEntry { name: "lfs.tar.gz".into(), size_bytes: 3, sha256: "2".into() },
        lfs_object_count: 0,
        created_at: "2026-03-25T00:00:00Z".into(),
    };
    write_manifest(&manifest_path, &manifest).unwrap();
    build_final_archive(&archive, &bundle, &lfs, &manifest_path).unwrap();

    let inspected = inspect_archive(&archive).unwrap();
    assert_eq!(inspected.start_commit, "abc");
    assert_eq!(inspected.end_commit, "def");
}
