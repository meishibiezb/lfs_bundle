use lfs_bundle::gui::app::{AppTab, BundleStudioApp};
use lfs_bundle::gui::views::import::ImportViewState;
use lfs_bundle::gui::views::pack::PackViewState;

#[test]
fn default_gui_tab_is_packaging() {
    let app = BundleStudioApp::default();
    assert_eq!(app.active_tab(), AppTab::Packaging);
}

#[test]
fn pack_form_builds_safe_mode_request_from_selected_commits() {
    let state = PackViewState {
        repo_path: "D:/repo".into(),
        start_commit: "abc123".into(),
        end_commit: "def456".into(),
        output_archive: "D:/out/pkg.zip".into(),
        safe_mode: true,
    };

    let request = state.to_request().expect("request");
    assert_eq!(request.repo_path.to_string_lossy(), "D:/repo");
    assert_eq!(request.start_commit, "abc123");
    assert_eq!(request.end_commit, "def456");
    assert_eq!(request.output_archive.to_string_lossy(), "D:/out/pkg.zip");
    assert!(request.safe_mode);
}

#[test]
fn import_form_builds_request_for_existing_branch() {
    let state = ImportViewState {
        archive_path: "D:/archive.zip".into(),
        repo_path: "D:/repo".into(),
        branch: "master".into(),
        safe_mode: true,
    };

    let request = state.to_request().expect("request");
    assert_eq!(request.archive_path.to_string_lossy(), "D:/archive.zip");
    assert_eq!(request.repo_path.to_string_lossy(), "D:/repo");
    assert_eq!(request.branch, "master");
    assert!(request.safe_mode);
}
