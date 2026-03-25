pub mod app;
pub mod theme;
pub mod views;

pub fn launch() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "LFS Bundle Studio",
        options,
        Box::new(|_cc| Ok(Box::new(app::BundleStudioApp::default()))),
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))
}
