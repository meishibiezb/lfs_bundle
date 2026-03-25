use lfs_bundle::gui::i18n::{Locale, set_locale, tr};

#[test]
fn zh_cn_default_key_returns_chinese_text() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("nav.packaging"), "\u{6253}\u{5305}");
}

#[test]
fn chinese_nav_labels_exist_in_dictionary() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("nav.packaging"), "\u{6253}\u{5305}");
    assert_eq!(tr("nav.import"), "\u{5BFC}\u{5165}");
}

#[test]
fn missing_key_falls_back_to_key_literal() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("missing.key"), "missing.key");
}
