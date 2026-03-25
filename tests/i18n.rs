use lfs_bundle::gui::i18n::{set_locale, tr, Locale};

#[test]
fn zh_cn_default_key_returns_chinese_text() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("nav.packaging"), "打包");
}

#[test]
fn missing_key_falls_back_to_key_literal() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("missing.key"), "missing.key");
}
