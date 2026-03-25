use crate::core::models::ImportRequest;
use egui::Ui;

#[derive(Clone, Debug, Default)]
pub struct ImportViewState {
    pub archive_path: String,
    pub repo_path: String,
    pub branch: String,
    pub safe_mode: bool,
}

impl ImportViewState {
    pub fn to_request(&self) -> Option<ImportRequest> {
        if self.archive_path.is_empty() || self.repo_path.is_empty() || self.branch.is_empty() {
            return None;
        }
        Some(ImportRequest {
            repo_path: self.repo_path.clone().into(),
            branch: self.branch.clone(),
            archive_path: self.archive_path.clone().into(),
            safe_mode: self.safe_mode,
        })
    }
}

pub fn render(ui: &mut Ui, state: &mut ImportViewState) {
    ui.heading("Import");
    ui.label("Archive path");
    ui.text_edit_singleline(&mut state.archive_path);
    ui.label("Repository");
    ui.text_edit_singleline(&mut state.repo_path);
    ui.label("Branch");
    ui.text_edit_singleline(&mut state.branch);
    ui.checkbox(&mut state.safe_mode, "Safe mode");
}
