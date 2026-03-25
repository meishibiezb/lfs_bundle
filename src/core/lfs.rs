use anyhow::{Context, Result};
use regex::Regex;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn parse_lfs_output(output: &str) -> Vec<PathBuf> {
    let re = Regex::new(r"^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]+)").expect("valid regex");

    output
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|caps| {
                let p1 = &caps[1];
                let p2 = &caps[2];
                let rest = &caps[3];

                Path::new(".git")
                    .join("lfs")
                    .join("objects")
                    .join(p1)
                    .join(p2)
                    .join(format!("{p1}{p2}{rest}"))
            })
        })
        .collect()
}

pub fn create_lfs_tar(filename: &Path, repo_root: &Path, files: &[PathBuf]) -> Result<()> {
    let file = File::create(filename)
        .with_context(|| format!("failed to create lfs archive: {}", filename.display()))?;
    let gz_encoder = flate2::GzBuilder::new().write(file, flate2::Compression::default());
    let mut tar_builder = tar::Builder::new(gz_encoder);

    for relative_path in files {
        let source = repo_root.join(relative_path);
        if source.exists() {
            tar_builder.append_path_with_name(&source, relative_path)?;
        }
    }

    let _ = tar_builder.into_inner()?;
    Ok(())
}
