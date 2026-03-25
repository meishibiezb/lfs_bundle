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
            AppTab::Packaging => pack::render(ui, &mut self.pack_view),
            AppTab::Import => import::render(ui, &mut self.import_view),
            AppTab::History => history::render(ui, &self.recent_ops, &mut self.history_view),
            AppTab::Settings => settings::render(ui, &mut self.settings_view),
        });
    }
}
