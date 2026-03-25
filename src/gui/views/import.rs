use crate::core::models::ImportRequest;
use crate::gui::i18n::tr;
use crate::gui::picker;
use egui::Ui;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct ImportViewState {
    pub archive_path: String,
    pub repo_path: String,
    pub branch: String,
    pub safe_mode: bool,
    pub status_message: Option<String>,
    pub status_is_error: bool,
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

    pub fn set_status_success(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_is_error = false;
    }

    pub fn set_status_error(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_is_error = true;
    }
}

pub fn render(ui: &mut Ui, state: &mut ImportViewState) -> Option<ImportRequest> {
    let mut import_request = None;

    ui.heading(tr("import.heading"));
    ui.group(|ui| {
        ui.label(tr("label.archive_path"));
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.archive_path);
            if ui.button(tr("btn.browse")).clicked() {
                state.apply_archive_path_from_picker(picker::pick_archive_file());
            }
        });

        ui.label(tr("label.repository"));
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.repo_path);
            if ui.button(tr("btn.browse")).clicked() {
                state.apply_repo_path_from_picker(picker::pick_repo_dir());
            }
        });

        ui.label(tr("label.branch"));
        ui.text_edit_singleline(&mut state.branch);
        ui.checkbox(&mut state.safe_mode, tr("label.safe_mode"));

        if let Some(message) = &state.status_message {
            let color = if state.status_is_error {
                egui::Color32::RED
            } else {
                egui::Color32::LIGHT_GREEN
            };
            ui.colored_label(color, message);
        }
    });

    ui.separator();
    ui.heading(tr("validation.heading"));
    if let Some(request) = state.to_request() {
        ui.colored_label(egui::Color32::LIGHT_GREEN, tr("status.ready_import"));
        if ui.button(tr("btn.import")).clicked() {
            import_request = Some(request);
        }
    } else {
        ui.colored_label(egui::Color32::YELLOW, tr("status.fill_required_import"));
    }

    import_request
}
