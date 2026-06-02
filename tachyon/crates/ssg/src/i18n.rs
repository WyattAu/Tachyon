//! Internationalization support for the SSG engine.

use std::collections::HashMap;
use std::path::Path;

use crate::error::SsgResult;
use crate::manifest::Translations;

/// Get the display name for a language code.
pub fn language_display_name(code: &str) -> &'static str {
    match code {
        "en" => "English",
        "zh" => "中文",
        "ja" => "日本語",
        "ko" => "한국어",
        "de" => "Deutsch",
        "fr" => "Français",
        "es" => "Español",
        "pt" => "Português",
        "ru" => "Русский",
        "it" => "Italiano",
        "nl" => "Nederlands",
        "ar" => "العربية",
        "hi" => "हिन्दी",
        "tr" => "Türkçe",
        "pl" => "Polski",
        "cs" => "Čeština",
        "sv" => "Svenska",
        "da" => "Dansk",
        "fi" => "Suomi",
        "no" => "Norsk",
        _ => "Unknown",
    }
}

/// Get the reading direction for a language.
pub fn text_direction(code: &str) -> &'static str {
    match code {
        "ar" | "he" | "fa" | "ur" => "rtl",
        _ => "ltr",
    }
}

/// Return built-in English defaults for all UI chrome strings.
///
/// The SSG works without any translation files — English is always available
/// as fallback.
pub fn default_translations() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("on_this_page".into(), "On this page".into());
    m.insert("previous".into(), "Previous".into());
    m.insert("next".into(), "Next".into());
    m.insert("copy".into(), "Copy".into());
    m.insert("copied".into(), "Copied!".into());
    m.insert("search".into(), "Search".into());
    m.insert("search_placeholder".into(), "Search...".into());
    m.insert("built_with".into(), "Built with".into());
    m.insert("table_of_contents".into(), "Table of Contents".into());
    m.insert("last_updated".into(), "Last updated".into());
    m.insert("updated".into(), "Updated".into());
    m.insert("skip_to_content".into(), "Skip to content".into());
    m.insert("toggle_sidebar".into(), "Toggle sidebar".into());
    m.insert("close".into(), "Close".into());
    m.insert("home".into(), "Home".into());
    m.insert("back_to_docs".into(), "Back to all docs".into());
    m.insert("tag".into(), "Tag".into());
    m.insert("document".into(), "document".into());
    m.insert("documents".into(), "documents".into());
    m.insert("theme".into(), "Theme".into());
    m.insert("mode".into(), "Mode".into());
    m.insert("color_theme".into(), "Color Theme".into());
    m.insert("page_not_found".into(), "Page Not Found".into());
    m.insert("redirecting".into(), "Redirecting to".into());
    m.insert("site_navigation".into(), "Site navigation".into());
    m
}

/// Load translations from YAML files in `_i18n/` directory.
///
/// Expects files named `{lang}.yaml` (e.g., `_i18n/zh.yaml`).
/// Missing files are silently skipped. English defaults are always included
/// even if no `en.yaml` is present.
pub fn load_translations(dir: &Path, languages: &[String]) -> SsgResult<Translations> {
    let mut strings: HashMap<String, HashMap<String, String>> = HashMap::new();

    let defaults = default_translations();
    strings.insert("en".into(), defaults);

    for lang in languages {
        if *lang == "en" {
            let en_yaml = dir.join("en.yaml");
            if en_yaml.exists() {
                let content = std::fs::read_to_string(&en_yaml).map_err(|e| {
                    crate::error::SsgError::Io(format!(
                        "Failed to read {}: {}",
                        en_yaml.display(),
                        e
                    ))
                })?;
                let parsed: HashMap<String, String> =
                    serde_yaml::from_str(&content).map_err(|e| {
                        crate::error::SsgError::Config(format!(
                            "Failed to parse {}: {}",
                            en_yaml.display(),
                            e
                        ))
                    })?;
                if let Some(map) = strings.get_mut("en") {
                    for (k, v) in parsed {
                        map.insert(k, v);
                    }
                }
            }
            continue;
        }

        let lang_yaml = dir.join(format!("{}.yaml", lang));
        if lang_yaml.exists() {
            let content = std::fs::read_to_string(&lang_yaml).map_err(|e| {
                crate::error::SsgError::Io(format!("Failed to read {}: {}", lang_yaml.display(), e))
            })?;
            let parsed: HashMap<String, String> = serde_yaml::from_str(&content).map_err(|e| {
                crate::error::SsgError::Config(format!(
                    "Failed to parse {}: {}",
                    lang_yaml.display(),
                    e
                ))
            })?;
            strings.insert(lang.clone(), parsed);
        }
    }

    Ok(Translations { strings })
}

/// Get a translated string for the given key and language.
///
/// Falls back to English if the key is missing in the requested language.
/// If the key doesn't exist at all, returns the key itself (useful for
/// debugging missing translations).
pub fn translate(translations: &Translations, key: &str, language: &str) -> String {
    if let Some(lang_map) = translations.strings.get(language)
        && let Some(value) = lang_map.get(key) {
            return value.clone();
        }

    if let Some(en_map) = translations.strings.get("en")
        && let Some(value) = en_map.get(key) {
            return value.clone();
        }

    key.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_translations_english() {
        let defaults = default_translations();
        let expected_keys = [
            "on_this_page",
            "previous",
            "next",
            "copy",
            "copied",
            "search",
            "search_placeholder",
            "built_with",
            "table_of_contents",
            "last_updated",
            "updated",
            "skip_to_content",
            "toggle_sidebar",
            "close",
            "home",
            "back_to_docs",
            "tag",
            "document",
            "documents",
            "theme",
            "mode",
            "color_theme",
            "page_not_found",
            "redirecting",
            "site_navigation",
        ];
        for key in &expected_keys {
            assert!(defaults.contains_key(*key), "missing default key: {}", key);
            assert!(
                !defaults.get(*key).unwrap().is_empty(),
                "empty default for key: {}",
                key
            );
        }
        assert_eq!(defaults.get("on_this_page").unwrap(), "On this page");
        assert_eq!(defaults.get("previous").unwrap(), "Previous");
        assert_eq!(defaults.get("next").unwrap(), "Next");
        assert_eq!(defaults.get("copy").unwrap(), "Copy");
        assert_eq!(defaults.get("copied").unwrap(), "Copied!");
        assert_eq!(defaults.get("search").unwrap(), "Search");
    }

    #[test]
    fn test_load_translations_from_yaml() {
        let tmp = std::env::temp_dir().join("tachyon-i18n-load-test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        fs::write(
            tmp.join("zh.yaml"),
            r#"on_this_page: "在此页上"
previous: "上一页"
next: "下一页"
copy: "复制"
copied: "已复制！"
search: "搜索"
table_of_contents: "目录"
last_updated: "最后更新"
updated: "更新于"
"#,
        )
        .unwrap();

        let translations = load_translations(&tmp, &["en".into(), "zh".into()]).unwrap();

        assert_eq!(translate(&translations, "on_this_page", "zh"), "在此页上");
        assert_eq!(translate(&translations, "previous", "zh"), "上一页");
        assert_eq!(translate(&translations, "next", "zh"), "下一页");
        assert_eq!(translate(&translations, "copy", "zh"), "复制");
        assert_eq!(translate(&translations, "copied", "zh"), "已复制！");
        assert_eq!(translate(&translations, "search", "zh"), "搜索");
        assert_eq!(translate(&translations, "table_of_contents", "zh"), "目录");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_translate_fallback_to_english() {
        let tmp = std::env::temp_dir().join("tachyon-i18n-fallback-test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        fs::write(
            tmp.join("zh.yaml"),
            r#"on_this_page: "在此页上"
"#,
        )
        .unwrap();

        let translations = load_translations(&tmp, &["en".into(), "zh".into()]).unwrap();

        assert_eq!(translate(&translations, "on_this_page", "zh"), "在此页上");
        assert_eq!(translate(&translations, "previous", "zh"), "Previous");
        assert_eq!(translate(&translations, "next", "zh"), "Next");
        assert_eq!(translate(&translations, "copy", "zh"), "Copy");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_translate_missing_key_returns_key() {
        let tmp = std::env::temp_dir().join("tachyon-i18n-missing-test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let translations = load_translations(&tmp, &["en".into()]).unwrap();

        assert_eq!(
            translate(&translations, "nonexistent_key", "en"),
            "nonexistent_key"
        );
        assert_eq!(
            translate(&translations, "totally_unknown", "zh"),
            "totally_unknown"
        );

        let _ = fs::remove_dir_all(&tmp);
    }
}
