use std::sync::{OnceLock, RwLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    ZhCn,
}

fn locale_store() -> &'static RwLock<Locale> {
    static STORE: OnceLock<RwLock<Locale>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(Locale::ZhCn))
}

pub fn set_locale(locale: Locale) {
    if let Ok(mut guard) = locale_store().write() {
        *guard = locale;
    }
}

fn current_locale() -> Locale {
    locale_store().read().map(|guard| *guard).unwrap_or(Locale::ZhCn)
}

pub fn tr(key: &str) -> String {
    match current_locale() {
        Locale::ZhCn => zh_cn(key).unwrap_or(key).to_string(),
    }
}

fn zh_cn(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.title" => "LFS 打包工具",
        "nav.packaging" => "打包",
        "nav.import" => "导入",
        "nav.history" => "历史记录",
        "nav.settings" => "设置",
        "label.repository" => "仓库路径",
        "label.start_commit" => "起始提交",
        "label.end_commit" => "结束提交",
        "label.output_archive" => "输出压缩包",
        "label.archive_path" => "压缩包路径",
        "label.branch" => "分支",
        "label.safe_mode" => "安全模式",
        "status.ready" => "就绪",
        "status.invalid_range" => "提交范围无效",
        _ => return None,
    })
}
