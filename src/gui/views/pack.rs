use crate::core::models::{CommitTreeNode, PackageRequest};
use crate::core::repo::{is_valid_commit_range, load_branch_commit_tree};
use crate::gui::i18n::tr;
use crate::gui::picker;
use anyhow::Result;
use egui::{ScrollArea, Ui};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct PackViewState {
    pub repo_path: String,
    pub branch: String,
    pub commits: Vec<CommitTreeNode>,
    pub highlighted_commit: Option<usize>,
    pub start_commit: String,
    pub end_commit: String,
    pub output_archive: String,
    pub safe_mode: bool,
    pub range_valid: Option<bool>,
    pub status_message: Option<String>,
}

impl Default for PackViewState {
    fn default() -> Self {
        Self {
            repo_path: String::new(),
            branch: "master".to_string(),
            commits: Vec::new(),
            highlighted_commit: None,
            start_commit: String::new(),
            end_commit: String::new(),
            output_archive: String::new(),
            safe_mode: true,
            range_valid: None,
            status_message: None,
        }
    }
}

impl PackViewState {
    pub fn apply_repo_path_from_picker(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.repo_path = path.to_string_lossy().to_string();
            self.commits.clear();
            self.highlighted_commit = None;
            self.start_commit.clear();
            self.end_commit.clear();
            self.range_valid = None;
        }
    }

    pub fn apply_output_archive_from_picker(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.output_archive = path.to_string_lossy().to_string();
        }
    }

    pub fn set_highlighted_as_start(&mut self) {
        if let Some(index) = self.highlighted_commit {
            if let Some(node) = self.commits.get(index) {
                self.start_commit = node.id.clone();
            }
        }
    }

    pub fn set_highlighted_as_end(&mut self) {
        if let Some(index) = self.highlighted_commit {
            if let Some(node) = self.commits.get(index) {
                self.end_commit = node.id.clone();
            }
        }
    }

    pub fn reload_commit_tree(&mut self) -> Result<()> {
        let repo_path = PathBuf::from(&self.repo_path);
        let commits = load_branch_commit_tree(&repo_path, &self.branch, 200)?;
        self.commits = commits;
        self.highlighted_commit = None;
        Ok(())
    }

    pub fn refresh_range_validity(&mut self) -> Result<()> {
        if self.start_commit.is_empty() || self.end_commit.is_empty() {
            self.range_valid = None;
            return Ok(());
        }

        let repo_path = Path::new(&self.repo_path);
        self.range_valid = Some(is_valid_commit_range(
            repo_path,
            &self.start_commit,
            &self.end_commit,
        )?);
        Ok(())
    }

    pub fn to_request(&self) -> Option<PackageRequest> {
        if self.repo_path.is_empty()
            || self.start_commit.is_empty()
            || self.end_commit.is_empty()
            || self.output_archive.is_empty()
            || self.range_valid != Some(true)
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
    ui.heading(tr("pack.heading"));
    ui.group(|ui| {
        ui.label(tr("label.repository"));
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.repo_path);
            if ui.button(tr("btn.browse")).clicked() {
                state.apply_repo_path_from_picker(picker::pick_repo_dir());
            }
        });

        ui.label(tr("label.branch"));
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.branch);
            if ui.button(tr("btn.load_commits")).clicked() {
                match state.reload_commit_tree() {
                    Ok(()) => state.status_message = None,
                    Err(err) => {
                        state.status_message =
                            Some(format!("{}: {err:#}", tr("status.load_commit_failed")))
                    }
                }
            }
        });

        ui.separator();
        ui.label(tr("pack.commit_tree"));
        ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            for (index, node) in state.commits.iter().enumerate() {
                let selected = state.highlighted_commit == Some(index);
                let label = format!("{} {} {}", node.graph_prefix, node.short_id, node.summary);
                if ui.selectable_label(selected, label).clicked() {
                    state.highlighted_commit = Some(index);
                }
            }
        });

        ui.horizontal(|ui| {
            if ui.button(tr("btn.set_start")).clicked() {
                state.set_highlighted_as_start();
                match state.refresh_range_validity() {
                    Ok(()) => state.status_message = None,
                    Err(err) => {
                        state.status_message =
                            Some(format!("{}: {err:#}", tr("status.validate_failed")))
                    }
                }
            }
            if ui.button(tr("btn.set_end")).clicked() {
                state.set_highlighted_as_end();
                match state.refresh_range_validity() {
                    Ok(()) => state.status_message = None,
                    Err(err) => {
                        state.status_message =
                            Some(format!("{}: {err:#}", tr("status.validate_failed")))
                    }
                }
            }
        });

        ui.label(format!("{}: {}", tr("label.start_selected"), state.start_commit));
        ui.label(format!("{}: {}", tr("label.end_selected"), state.end_commit));

        ui.label(tr("label.output_archive"));
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.output_archive);
            if ui.button(tr("btn.browse")).clicked() {
                state.apply_output_archive_from_picker(picker::pick_output_archive_file());
            }
        });

        ui.checkbox(&mut state.safe_mode, tr("label.safe_mode"));

        if let Some(message) = &state.status_message {
            ui.colored_label(egui::Color32::RED, message);
        }
    });

    ui.separator();
    ui.heading(tr("preview.heading"));
    match state.range_valid {
        Some(true) => ui.colored_label(egui::Color32::LIGHT_GREEN, tr("status.range_valid")),
        Some(false) => ui.colored_label(egui::Color32::RED, tr("status.range_invalid")),
        None => ui.colored_label(egui::Color32::YELLOW, tr("status.range_pending")),
    };

    if let Some(request) = state.to_request() {
        ui.label(format!("{} -> {}", request.start_commit, request.end_commit));
        ui.label(format!("Output: {}", request.output_archive.display()));
    } else {
        ui.colored_label(egui::Color32::YELLOW, tr("status.fill_required_pack"));
    }
}
