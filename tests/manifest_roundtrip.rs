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
