#[test]
fn help_command_mentions_pack_import_and_gui() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lfs_bundle"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pack"));
    assert!(stdout.contains("import"));
    assert!(stdout.contains("gui"));
}
