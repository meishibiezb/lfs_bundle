use egui::Ui;

#[derive(Clone, Debug, Default)]
pub struct HistoryViewState;

pub fn render(ui: &mut Ui, _state: &mut HistoryViewState) {
    ui.heading("History");
    ui.label("Completed operations will appear here.");
}
