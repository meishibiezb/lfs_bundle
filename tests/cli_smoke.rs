use std::process::Command;

#[test]
fn help_command_mentions_pack_import_and_gui() {
    let output = Command::new(env!("CARGO_BIN_EXE_lfs_bundle"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pack"));
    assert!(stdout.contains("import"));
    assert!(stdout.contains("gui"));
}

#[test]
fn inspect_prints_manifest_commit_range() {
    let temp = tempfile::tempdir().unwrap();
    let bundle = temp.path().join("bundle.bundle");
    let lfs = temp.path().join("lfs.tar.gz");
    let manifest_path = temp.path().join("manifest.json");
    let archive = temp.path().join("archive.zip");

    std::fs::write(&bundle, b"bundle").unwrap();
    std::fs::write(&lfs, b"lfs").unwrap();

    let manifest = lfs_bundle::core::manifest::BundleManifest {
        tool_version: "0.1.0".into(),
        source_repo: "demo".into(),
        start_commit: "abc".into(),
        end_commit: "def".into(),
        target_commit: "def".into(),
        bundle: lfs_bundle::core::manifest::FileEntry {
            name: "bundle.bundle".into(),
            size_bytes: 6,
            sha256: "1".into(),
        },
        lfs: lfs_bundle::core::manifest::FileEntry {
            name: "lfs.tar.gz".into(),
            size_bytes: 3,
            sha256: "2".into(),
        },
        lfs_object_count: 0,
        created_at: "2026-03-25T00:00:00Z".into(),
    };
    lfs_bundle::core::manifest::write_manifest(&manifest_path, &manifest).unwrap();
    lfs_bundle::core::archive::build_final_archive(&archive, &bundle, &lfs, &manifest_path)
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_lfs_bundle"))
        .arg("inspect")
        .arg(&archive)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("start_commit: abc"));
    assert!(stdout.contains("end_commit: def"));
}

#[test]
fn failed_import_exits_non_zero() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    Command::new("git").arg("init").arg(&repo).output().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_lfs_bundle"))
        .arg("import")
        .arg(&repo)
        .arg("master")
        .arg(temp.path().join("missing.zip"))
        .output()
        .unwrap();

    assert!(!output.status.success());
}
