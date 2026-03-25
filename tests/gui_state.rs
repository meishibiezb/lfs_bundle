use lfs_bundle::gui::app::{AppTab, BundleStudioApp};

#[test]
fn default_gui_tab_is_packaging() {
    let app = BundleStudioApp::default();
    assert_eq!(app.active_tab(), AppTab::Packaging);
}
