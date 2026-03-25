#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    if let Err(err) = lfs_bundle::gui::launch() {
        let message = format!("{err:#}");
        #[cfg(not(target_os = "windows"))]
        eprintln!("{message}");

        let _ = rfd::MessageDialog::new()
            .set_title("LFS Bundle")
            .set_description(&message)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}
