use lfs_bundle::core::lfs::parse_lfs_output;
use std::path::PathBuf;

#[test]
fn parse_lfs_output_converts_hashes_to_object_paths() {
    let output = "52b6c1c0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa * file.bin";
    let paths = parse_lfs_output(output);
    assert_eq!(
        paths,
        vec![PathBuf::from(
            ".git/lfs/objects/52/b6/52b6c1c0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        )]
    );
}
