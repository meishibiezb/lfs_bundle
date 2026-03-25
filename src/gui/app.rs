use crate::core::import::import_archive;
use crate::core::models::ImportRequest;
use crate::core::models::PackageRequest;
use crate::core::pack::package_repository;
use crate::gui::i18n::tr;
use crate::gui::theme::AppTheme;
use crate::gui::views::{history, import, pack, settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    Packaging,
    Import,
    History,
    Settings,
}

pub struct BundleStudioApp {
    active_tab: AppTab,
    pub logs: Vec<String>,
    pub recent_ops: Vec<String>,
    pub theme: AppTheme,
    pub pack_view: pack::PackViewState,
    pub import_view: import::ImportViewState,
    pub history_view: history::HistoryViewState,
    pub settings_view: settings::SettingsViewState,
}

impl BundleStudioApp {
    pub fn active_tab(&self) -> AppTab {
        self.active_tab
    }

    pub fn record_operation(&mut self, summary: impl Into<String>) {
        self.recent_ops.push(summary.into());
    }

    fn execute_package(&mut self, request: PackageRequest) {
        match package_repository(&request) {
            Ok(summary) => {
                self.pack_view.set_status_success(format!(
                    "{}: {} commits, {} lfs objects",
                    tr("status.package_success"),
                    summary.commit_count,
                    summary.lfs_object_count
                ));
                self.record_operation(format!(
                    "pack {}..{} -> {}",
                    request.start_commit,
                    request.end_commit,
                    request.output_archive.display()
                ));
            }
            Err(err) => {
                self.pack_view
                    .set_status_error(format!("{}: {err:#}", tr("status.package_failed")));
            }
        }
    }

    fn execute_import(&mut self, request: ImportRequest) {
        match import_archive(&request) {
            Ok(()) => {
                self.import_view
                    .set_status_success(tr("status.import_success"));
                self.record_operation(format!(
                    "import {} -> {}",
                    request.archive_path.display(),
                    request.branch
                ));
            }
            Err(err) => {
                self.import_view
                    .set_status_error(format!("{}: {err:#}", tr("status.import_failed")));
            }
        }
    }
}

impl Default for BundleStudioApp {
    fn default() -> Self {
        Self {
            active_tab: AppTab::Packaging,
            logs: Vec::new(),
            recent_ops: Vec::new(),
            theme: AppTheme {
                accent: AppTheme::default_accent(),
            },
            pack_view: pack::PackViewState {
                safe_mode: true,
                ..Default::default()
            },
            import_view: import::ImportViewState {
                safe_mode: true,
                ..Default::default()
            },
            history_view: history::HistoryViewState,
            settings_view: settings::SettingsViewState::default(),
        }
    }
}

impl eframe::App for BundleStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(tr("app.title"));
                ui.label(format!(
                    "{}: #{:02X}{:02X}{:02X}",
                    tr("label.accent"),
                    self.theme.accent[0],
                    self.theme.accent[1],
                    self.theme.accent[2]
                ));
            });
        });

        egui::SidePanel::left("nav").show(ctx, |ui| {
            if ui.button(tr("nav.packaging")).clicked() {
                self.active_tab = AppTab::Packaging;
            }
            if ui.button(tr("nav.import")).clicked() {
                self.active_tab = AppTab::Import;
            }
            if ui.button(tr("nav.history")).clicked() {
                self.active_tab = AppTab::History;
            }
            if ui.button(tr("nav.settings")).clicked() {
                self.active_tab = AppTab::Settings;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            AppTab::Packaging => {
                let request = pack::render(ui, &mut self.pack_view);
                if let Some(request) = request {
                    self.execute_package(request);
                }
            }
            AppTab::Import => {
                let request = import::render(ui, &mut self.import_view);
                if let Some(request) = request {
                    self.execute_import(request);
                }
            }
            AppTab::History => history::render(ui, &self.recent_ops, &mut self.history_view),
            AppTab::Settings => settings::render(ui, &mut self.settings_view),
        });
    }
}
