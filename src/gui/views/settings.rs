use egui::Ui;

#[derive(Clone, Debug)]
pub struct SettingsViewState {
    pub default_safe_mode: bool,
    pub custom_git_path: String,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        Self {
            default_safe_mode: true,
            custom_git_path: String::new(),
        }
    }
}

pub fn render(ui: &mut Ui, state: &mut SettingsViewState) {
    ui.heading("Settings");
    ui.checkbox(&mut state.default_safe_mode, "Default to safe mode");
    ui.label("Custom git path");
    ui.text_edit_singleline(&mut state.custom_git_path);
}
