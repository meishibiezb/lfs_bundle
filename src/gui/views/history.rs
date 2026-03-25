use egui::Ui;

#[derive(Clone, Debug, Default)]
pub struct HistoryViewState;

pub fn render(ui: &mut Ui, entries: &[String], _state: &mut HistoryViewState) {
    ui.heading("History");
    if entries.is_empty() {
        ui.label("No completed operations yet.");
    } else {
        for entry in entries {
            ui.group(|ui| {
                ui.label(entry);
            });
        }
    }
}
