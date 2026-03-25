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
    locale_store()
        .read()
        .map(|guard| *guard)
        .unwrap_or(Locale::ZhCn)
}

pub fn tr(key: &str) -> String {
    match current_locale() {
        Locale::ZhCn => zh_cn(key).unwrap_or(key).to_string(),
    }
}

fn zh_cn(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.title" => "LFS 打包工具",
        "label.accent" => "主题色",
        "nav.packaging" => "打包",
        "nav.import" => "导入",
        "nav.history" => "历史记录",
        "nav.settings" => "设置",
        "pack.heading" => "打包",
        "pack.commit_tree" => "提交树（当前分支）",
        "preview.heading" => "预览",
        "import.heading" => "导入",
        "history.heading" => "历史记录",
        "history.empty" => "暂无已完成操作。",
        "settings.heading" => "设置",
        "settings.default_safe_mode" => "默认使用安全模式",
        "settings.custom_git_path" => "自定义 Git 路径",
        "validation.heading" => "校验",
        "label.repository" => "仓库路径",
        "label.start_commit" => "起始提交",
        "label.end_commit" => "结束提交",
        "label.output_archive" => "输出压缩包",
        "label.archive_path" => "压缩包路径",
        "label.branch" => "分支",
        "label.safe_mode" => "安全模式",
        "label.start_selected" => "已选起始提交",
        "label.end_selected" => "已选结束提交",
        "btn.browse" => "浏览...",
        "btn.load_commits" => "加载提交",
        "btn.set_start" => "设为起始提交",
        "btn.set_end" => "设为结束提交",
        "btn.package" => "执行打包",
        "btn.import" => "执行导入",
        "status.ready_import" => "可开始导入",
        "status.fill_required_import" => "请先填写压缩包、仓库和分支",
        "status.fill_required_pack" => "请先填写仓库、提交范围和输出路径",
        "status.range_valid" => "提交范围有效。",
        "status.range_invalid" => "提交范围无效。",
        "status.range_pending" => "请先设置起始提交和结束提交。",
        "status.load_commit_failed" => "加载提交失败",
        "status.validate_failed" => "范围校验失败",
        "status.package_success" => "打包成功",
        "status.package_failed" => "打包失败",
        "status.import_success" => "导入成功",
        "status.import_failed" => "导入失败",
        _ => return None,
    })
}
