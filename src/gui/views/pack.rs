use crate::core::models::PackageRequest;
use crate::gui::picker;
use egui::Ui;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct PackViewState {
    pub repo_path: String,
    pub start_commit: String,
    pub end_commit: String,
    pub output_archive: String,
    pub safe_mode: bool,
}

impl PackViewState {
    pub fn apply_repo_path_from_picker(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.repo_path = path.to_string_lossy().to_string();
        }
    }

    pub fn apply_output_archive_from_picker(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.output_archive = path.to_string_lossy().to_string();
        }
    }

    pub fn to_request(&self) -> Option<PackageRequest> {
        if self.repo_path.is_empty()
            || self.start_commit.is_empty()
            || self.end_commit.is_empty()
            || self.output_archive.is_empty()
        {
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
    ui.group(|ui| {
        ui.label("Repository");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.repo_path);
            if ui.button("Browse...").clicked() {
                state.apply_repo_path_from_picker(picker::pick_repo_dir());
            }
        });

        ui.label("Start commit");
        ui.text_edit_singleline(&mut state.start_commit);
        ui.label("End commit");
        ui.text_edit_singleline(&mut state.end_commit);

        ui.label("Output archive");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.output_archive);
            if ui.button("Browse...").clicked() {
                state.apply_output_archive_from_picker(picker::pick_output_archive_file());
            }
        });

        ui.checkbox(&mut state.safe_mode, "Safe mode");
    });

    ui.separator();
    ui.heading("Preview");
    if let Some(request) = state.to_request() {
        ui.label(format!("{} -> {}", request.start_commit, request.end_commit));
        ui.label(format!("Output: {}", request.output_archive.display()));
    } else {
        ui.colored_label(
            egui::Color32::YELLOW,
            "Fill repository, commit range, and output path",
        );
    }
}
