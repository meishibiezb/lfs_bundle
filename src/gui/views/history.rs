use crate::gui::i18n::tr;
use egui::Ui;

#[derive(Clone, Debug, Default)]
pub struct HistoryViewState;

pub fn render(ui: &mut Ui, entries: &[String], _state: &mut HistoryViewState) {
    ui.heading(tr("history.heading"));
    if entries.is_empty() {
        ui.label(tr("history.empty"));
    } else {
        for entry in entries {
            ui.group(|ui| {
                ui.label(entry);
            });
        }
    }
}
