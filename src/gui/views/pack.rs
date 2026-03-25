use crate::core::models::PackageRequest;
use egui::Ui;

#[derive(Clone, Debug, Default)]
pub struct PackViewState {
    pub repo_path: String,
    pub start_commit: String,
    pub end_commit: String,
    pub output_archive: String,
    pub safe_mode: bool,
}

impl PackViewState {
    pub fn to_request(&self) -> Option<PackageRequest> {
        if self.repo_path.is_empty() || self.start_commit.is_empty() || self.end_commit.is_empty() || self.output_archive.is_empty() {
            return None;
        }
        Some(PackageRequest {
            repo_path: self.repo_path.clone().into(),
            start_commit: self.start_commit.clone(),
            end_commit: self.end_commit.clone(),
            output_archive: self.output_archive.clone().into(),
            safe_mode: self.safe_mode,
        })
    }
}

pub fn render(ui: &mut Ui, state: &mut PackViewState) {
    ui.heading("Packaging");
    ui.label("Repository");
    ui.text_edit_singleline(&mut state.repo_path);
    ui.label("Start commit");
    ui.text_edit_singleline(&mut state.start_commit);
    ui.label("End commit");
    ui.text_edit_singleline(&mut state.end_commit);
    ui.label("Output archive");
    ui.text_edit_singleline(&mut state.output_archive);
    ui.checkbox(&mut state.safe_mode, "Safe mode");
}
