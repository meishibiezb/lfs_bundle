use lfs_bundle::core::repo::split_short_id;

#[test]
fn split_short_id_truncates_long_commit_ids() {
    assert_eq!(split_short_id("1234567890abcdef"), "12345678");
}
