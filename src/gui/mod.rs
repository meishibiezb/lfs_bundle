pub mod app;
pub mod i18n;
pub mod picker;
pub mod theme;
pub mod views;

pub fn launch() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    let title = i18n::tr("app.title");
    eframe::run_native(
        title.as_str(),
        options,
        Box::new(|_cc| Ok(Box::new(app::BundleStudioApp::default()))),
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))
}
