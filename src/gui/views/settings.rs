use crate::gui::i18n::tr;
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
    ui.heading(tr("settings.heading"));
    ui.checkbox(
        &mut state.default_safe_mode,
        tr("settings.default_safe_mode"),
    );
    ui.label(tr("settings.custom_git_path"));
    ui.text_edit_singleline(&mut state.custom_git_path);
}
