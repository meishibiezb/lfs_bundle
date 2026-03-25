pub mod app;
pub mod i18n;
pub mod picker;
pub mod theme;
pub mod views;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

pub fn launch() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    let title = i18n::tr("app.title");
    eframe::run_native(
        title.as_str(),
        options,
        Box::new(|cc| {
            configure_cjk_font_fallback(&cc.egui_ctx);
            Ok(Box::new(app::BundleStudioApp::default()))
        }),
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))
}

fn configure_cjk_font_fallback(ctx: &egui::Context) {
    if let Some((name, bytes)) = load_first_existing_font(&candidate_cjk_font_paths()) {
        let mut fonts = FontDefinitions::default();
        inject_cjk_fallback_font(&mut fonts, &name, bytes);
        ctx.set_fonts(fonts);
    }
}

fn inject_cjk_fallback_font(fonts: &mut FontDefinitions, name: &str, bytes: Vec<u8>) {
    fonts
        .font_data
        .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let entries = fonts.families.entry(family).or_default();
        if !entries.iter().any(|existing| existing == name) {
            entries.push(name.to_owned());
        }
    }
}

fn load_first_existing_font(paths: &[PathBuf]) -> Option<(String, Vec<u8>)> {
    for path in paths {
        if !path.exists() {
            continue;
        }

        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if bytes.is_empty() {
            continue;
        }

        return Some((format!("cjk-{}", font_stem(path)?), bytes));
    }

    None
}

fn font_stem(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let sanitized = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

fn candidate_cjk_font_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        [
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\msyh.ttf",
            "C:\\Windows\\Fonts\\msyhbd.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "C:\\Windows\\Fonts\\simsun.ttc",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }

    #[cfg(target_os = "macos")]
    {
        [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/arphic/ukai.ttc",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use egui::FontFamily;

    use super::{candidate_cjk_font_paths, inject_cjk_fallback_font, load_first_existing_font};

    #[test]
    fn inject_cjk_fallback_font_appends_to_both_font_families() {
        let mut fonts = egui::FontDefinitions::default();
        let name = "test-cjk";
        let bytes = vec![1, 2, 3, 4];

        inject_cjk_fallback_font(&mut fonts, name, bytes.clone());

        let proportional = fonts
            .families
            .get(&FontFamily::Proportional)
            .expect("proportional family exists");
        let monospace = fonts
            .families
            .get(&FontFamily::Monospace)
            .expect("monospace family exists");

        assert_eq!(proportional.last(), Some(&name.to_string()));
        assert_eq!(monospace.last(), Some(&name.to_string()));

        let loaded = fonts.font_data.get(name).expect("font should be installed");
        assert_eq!(loaded.as_ref(), &egui::FontData::from_owned(bytes));
    }

    #[test]
    fn load_first_existing_font_prefers_first_readable_candidate() {
        let dir = tempfile::tempdir().expect("tempdir");
        let first = dir.path().join("first.ttf");
        let second = dir.path().join("second.ttf");
        fs::write(&first, [0x01, 0x02]).expect("write first");
        fs::write(&second, [0x03, 0x04]).expect("write second");

        let loaded = load_first_existing_font(&[first.clone(), second])
            .expect("should load first existing font");

        assert_eq!(loaded.0, "cjk-first");
        assert_eq!(loaded.1, vec![0x01, 0x02]);
    }

    #[test]
    fn windows_candidate_font_paths_include_microsoft_yahei() {
        if cfg!(target_os = "windows") {
            let paths = candidate_cjk_font_paths();
            assert!(paths.iter().any(|path| {
                path.to_string_lossy()
                    .to_ascii_lowercase()
                    .contains("windows\\fonts\\msyh")
            }));
        }
    }
}
