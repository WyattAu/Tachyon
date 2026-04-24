//! Internationalization support for the SSG engine.

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
