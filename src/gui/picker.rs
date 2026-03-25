use std::path::PathBuf;

pub fn pick_repo_dir() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

pub fn pick_archive_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Archive", &["zip"])
        .pick_file()
}

pub fn pick_output_archive_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Archive", &["zip"])
        .set_file_name("bundle-package.zip")
        .save_file()
}
