use crate::core::manifest::{read_manifest, BundleManifest};
use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub fn build_final_archive(output_zip: &Path, bundle: &Path, lfs: &Path, manifest: &Path) -> Result<()> {
    let file = fs::File::create(output_zip)
        .with_context(|| format!("failed to create archive: {}", output_zip.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (name, path) in [
        ("bundle.bundle", bundle),
        ("lfs.tar.gz", lfs),
        ("manifest.json", manifest),
    ] {
        zip.start_file(name, options)?;
        let bytes = fs::read(path).with_context(|| format!("failed to read archive member: {}", path.display()))?;
        zip.write_all(&bytes)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn extract_archive(input_zip: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).with_context(|| format!("failed to create extraction directory: {}", dest.display()))?;
    let file = fs::File::open(input_zip)
        .with_context(|| format!("failed to open archive: {}", input_zip.display()))?;
    let mut zip = ZipArchive::new(file)?;

    for index in 0..zip.len() {
        let mut member = zip.by_index(index)?;
        let out_path = dest.join(member.name());
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out_file = fs::File::create(&out_path)?;
        io::copy(&mut member, &mut out_file)?;
    }

    Ok(())
}

pub fn inspect_archive(input_zip: &Path) -> Result<BundleManifest> {
    let temp = tempfile::tempdir()?;
    extract_archive(input_zip, temp.path())?;
    read_manifest(&temp.path().join("manifest.json"))
}
