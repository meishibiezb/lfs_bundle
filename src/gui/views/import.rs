use crate::core::models::ImportRequest;
use crate::gui::picker;
use egui::Ui;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct ImportViewState {
    pub archive_path: String,
    pub repo_path: String,
    pub branch: String,
    pub safe_mode: bool,
}

impl ImportViewState {
    pub fn apply_archive_path_from_picker(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.archive_path = path.to_string_lossy().to_string();
        }
    }

    pub fn apply_repo_path_from_picker(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.repo_path = path.to_string_lossy().to_string();
        }
    }

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
    ui.group(|ui| {
        ui.label("Archive path");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.archive_path);
            if ui.button("Browse...").clicked() {
                state.apply_archive_path_from_picker(picker::pick_archive_file());
            }
        });

        ui.label("Repository");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.repo_path);
            if ui.button("Browse...").clicked() {
                state.apply_repo_path_from_picker(picker::pick_repo_dir());
            }
        });

        ui.label("Branch");
        ui.text_edit_singleline(&mut state.branch);
        ui.checkbox(&mut state.safe_mode, "Safe mode");
    });

    ui.separator();
    ui.heading("Validation");
    if state.to_request().is_some() {
        ui.colored_label(egui::Color32::LIGHT_GREEN, "Ready to import");
    } else {
        ui.colored_label(
            egui::Color32::YELLOW,
            "Fill archive, repo, and branch first",
        );
    }
}
