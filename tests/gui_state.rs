use lfs_bundle::core::models::CommitTreeNode;
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
        range_valid: Some(true),
        ..Default::default()
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
        ..Default::default()
    };

    let request = state.to_request().expect("request");
    assert_eq!(request.archive_path.to_string_lossy(), "D:/archive.zip");
    assert_eq!(request.repo_path.to_string_lossy(), "D:/repo");
    assert_eq!(request.branch, "master");
    assert!(request.safe_mode);
}

#[test]
fn completed_operation_is_recorded_in_history_view_model() {
    let mut app = BundleStudioApp::default();
    app.record_operation("imported package into master");
    assert_eq!(app.recent_ops.len(), 1);
}

#[test]
fn pack_state_accepts_repo_path_from_picker_result() {
    let mut state = PackViewState::default();
    state.apply_repo_path_from_picker(Some("D:/repo".into()));
    assert_eq!(state.repo_path, "D:/repo");
}

#[test]
fn import_state_ignores_cancelled_picker_result() {
    let mut state = ImportViewState::default();
    state.archive_path = "existing.zip".into();
    state.apply_archive_path_from_picker(None);
    assert_eq!(state.archive_path, "existing.zip");
}

#[test]
fn clicking_commit_then_set_start_updates_start_commit() {
    let mut state = PackViewState {
        commits: vec![CommitTreeNode {
            id: "1234567890abcdef".into(),
            short_id: "12345678".into(),
            summary: "demo".into(),
            author: "u".into(),
            timestamp: "t".into(),
            parents: vec![],
            graph_prefix: "*".into(),
        }],
        highlighted_commit: Some(0),
        ..Default::default()
    };

    state.set_highlighted_as_start();
    assert_eq!(state.start_commit, "1234567890abcdef");
}

#[test]
fn invalid_commit_range_blocks_request_generation() {
    let state = PackViewState {
        repo_path: "D:/repo".into(),
        start_commit: "end".into(),
        end_commit: "start".into(),
        output_archive: "D:/out/pkg.zip".into(),
        safe_mode: true,
        range_valid: Some(false),
        ..Default::default()
    };

    assert!(state.to_request().is_none());
}

#[test]
fn same_start_and_end_commit_blocks_request_generation() {
    let state = PackViewState {
        repo_path: "D:/repo".into(),
        start_commit: "same".into(),
        end_commit: "same".into(),
        output_archive: "D:/out/pkg.zip".into(),
        safe_mode: true,
        range_valid: Some(true),
        ..Default::default()
    };

    assert!(state.to_request().is_none());
}

#[test]
fn pack_success_status_is_not_error() {
    let mut state = PackViewState::default();
    state.set_status_success("ok");
    assert!(!state.status_is_error);
}

#[test]
fn import_success_status_is_not_error() {
    let mut state = ImportViewState::default();
    state.set_status_success("ok");
    assert!(!state.status_is_error);
}
