//! Internationalization (i18n) infrastructure for Tachyon frontend.
//!
//! Provides a simple, type-safe translation system with:
//! - Locale detection from browser / user preference
//! - Translation files per locale
//! - Reactive locale switching
//! - Fallback chain: requested locale → English

use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Supported locale codes (ISO 639-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::incompatible_msrv)]
pub enum Locale {
    En,
    Zh,
    Ja,
    De,
    Fr,
    Es,
    Ko,
    Pt,
}

/// Locale implementation.
///
/// Reserved for future use: internationalization support.
impl Locale {
    /// All supported locales.
    pub const ALL: &[Locale] = &[
        Locale::En,
        Locale::Zh,
        Locale::Ja,
        Locale::De,
        Locale::Fr,
        Locale::Es,
        Locale::Ko,
        Locale::Pt,
    ];

    /// ISO 639-1 code.
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Zh => "zh",
            Locale::Ja => "ja",
            Locale::De => "de",
            Locale::Fr => "fr",
            Locale::Es => "es",
            Locale::Ko => "ko",
            Locale::Pt => "pt",
        }
    }

    /// Display name in the locale's own language.
    pub fn native_name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Zh => "中文",
            Locale::Ja => "日本語",
            Locale::De => "Deutsch",
            Locale::Fr => "Français",
            Locale::Es => "Español",
            Locale::Ko => "한국어",
            Locale::Pt => "Português",
        }
    }

    /// Parse from ISO 639-1 code, defaulting to English.
    pub fn from_code(code: &str) -> Self {
        match code
            .split('-')
            .next()
            .unwrap_or("en")
            .to_lowercase()
            .as_str()
        {
            "zh" => Locale::Zh,
            "ja" => Locale::Ja,
            "de" => Locale::De,
            "fr" => Locale::Fr,
            "es" => Locale::Es,
            "ko" => Locale::Ko,
            "pt" => Locale::Pt,
            _ => Locale::En,
        }
    }

    /// Detect locale from browser `navigator.language`.
    pub fn detect_from_browser() -> Self {
        let window = web_sys::window().expect("no window");
        let nav = window.navigator();
        let lang = nav.language().unwrap_or_else(|| "en".to_string());
        Self::from_code(&lang)
    }
}

/// Translation key identifiers.
///
/// Reserved for future use: i18n key constants for UI translations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TranslationKey(&'static str);

impl TranslationKey {
    // Navigation
    pub const HOME: Self = TranslationKey("nav.home");
    pub const DASHBOARD: Self = TranslationKey("nav.dashboard");
    pub const DOCUMENTS: Self = TranslationKey("nav.documents");
    pub const SEARCH: Self = TranslationKey("nav.search");
    pub const SETTINGS: Self = TranslationKey("nav.settings");
    pub const CATALOG: Self = TranslationKey("nav.catalog");
    pub const TEAMS: Self = TranslationKey("nav.teams");
    pub const PLUGINS: Self = TranslationKey("nav.plugins");
    pub const SPACES: Self = TranslationKey("nav.spaces");
    pub const GRAPH: Self = TranslationKey("nav.graph");
    pub const TAGS: Self = TranslationKey("nav.tags");
    pub const BILLING: Self = TranslationKey("nav.billing");
    pub const SSG: Self = TranslationKey("nav.ssg");
    pub const ADMIN: Self = TranslationKey("nav.admin");
    pub const AUDIT: Self = TranslationKey("nav.audit");
    pub const TEMPLATES: Self = TranslationKey("nav.templates");

    // Actions
    pub const SAVE: Self = TranslationKey("action.save");
    pub const CANCEL: Self = TranslationKey("action.cancel");
    pub const DELETE: Self = TranslationKey("action.delete");
    pub const CREATE: Self = TranslationKey("action.create");
    pub const EDIT: Self = TranslationKey("action.edit");
    pub const SIGN_IN: Self = TranslationKey("action.sign_in");
    pub const SIGN_OUT: Self = TranslationKey("action.sign_out");
    pub const REGISTER: Self = TranslationKey("action.register");

    // Common
    pub const LOADING: Self = TranslationKey("common.loading");
    pub const ERROR: Self = TranslationKey("common.error");
    pub const NO_RESULTS: Self = TranslationKey("common.no_results");
    pub const CONFIRM_DELETE: Self = TranslationKey("common.confirm_delete");

    // Documents
    pub const NEW_DOCUMENT: Self = TranslationKey("docs.new");
    pub const NO_DOCUMENTS: Self = TranslationKey("docs.empty");
    pub const DOCUMENT_NOT_FOUND: Self = TranslationKey("docs.not_found");
    pub const DRAFT: Self = TranslationKey("docs.draft");
    pub const PUBLISHED: Self = TranslationKey("docs.published");
    pub const ARCHIVED: Self = TranslationKey("docs.archived");
}

/// English translations (source of truth).
#[allow(clippy::incompatible_msrv)]
static EN_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Navigation
    m.insert("nav.home", "Home");
    m.insert("nav.dashboard", "Dashboard");
    m.insert("nav.documents", "Documents");
    m.insert("nav.search", "Search");
    m.insert("nav.settings", "Settings");
    m.insert("nav.catalog", "Catalog");
    m.insert("nav.teams", "Teams");
    m.insert("nav.plugins", "Plugins");
    m.insert("nav.spaces", "Spaces");
    m.insert("nav.graph", "Graph");
    m.insert("nav.tags", "Tags");
    m.insert("nav.billing", "Billing");
    m.insert("nav.ssg", "Static Site");
    m.insert("nav.admin", "Admin");
    m.insert("nav.audit", "Audit Log");
    m.insert("nav.templates", "Templates");
    // Actions
    m.insert("action.save", "Save");
    m.insert("action.cancel", "Cancel");
    m.insert("action.delete", "Delete");
    m.insert("action.create", "Create");
    m.insert("action.edit", "Edit");
    m.insert("action.search", "Search");
    m.insert("action.sign_in", "Sign In");
    m.insert("action.sign_out", "Sign Out");
    m.insert("action.register", "Register");
    // Common
    m.insert("common.loading", "Loading...");
    m.insert("common.error", "Something went wrong");
    m.insert("common.no_results", "No results found");
    m.insert(
        "common.confirm_delete",
        "Are you sure you want to delete this?",
    );
    // Documents
    m.insert("docs.new", "New Document");
    m.insert("docs.empty", "No documents yet");
    m.insert("docs.not_found", "Document not found");
    m.insert("docs.draft", "Draft");
    m.insert("docs.published", "Published");
    m.insert("docs.archived", "Archived");
    m
});

/// Chinese translations.
#[allow(clippy::incompatible_msrv)]
static ZH_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "首页");
    m.insert("nav.dashboard", "仪表板");
    m.insert("nav.documents", "文档");
    m.insert("nav.search", "搜索");
    m.insert("nav.settings", "设置");
    m.insert("nav.catalog", "目录");
    m.insert("nav.teams", "团队");
    m.insert("nav.plugins", "插件");
    m.insert("nav.spaces", "空间");
    m.insert("nav.graph", "知识图谱");
    m.insert("nav.tags", "标签");
    m.insert("nav.billing", "账单");
    m.insert("nav.ssg", "静态站点");
    m.insert("nav.admin", "管理");
    m.insert("nav.audit", "审计日志");
    m.insert("nav.templates", "模板");
    m.insert("action.save", "保存");
    m.insert("action.cancel", "取消");
    m.insert("action.delete", "删除");
    m.insert("action.create", "创建");
    m.insert("action.edit", "编辑");
    m.insert("action.search", "搜索");
    m.insert("action.sign_in", "登录");
    m.insert("action.sign_out", "退出");
    m.insert("action.register", "注册");
    m.insert("common.loading", "加载中...");
    m.insert("common.error", "出了点问题");
    m.insert("common.no_results", "未找到结果");
    m.insert("common.confirm_delete", "确定要删除吗？");
    m.insert("docs.new", "新建文档");
    m.insert("docs.empty", "暂无文档");
    m.insert("docs.not_found", "文档未找到");
    m.insert("docs.draft", "草稿");
    m.insert("docs.published", "已发布");
    m.insert("docs.archived", "已归档");
    m
});

/// Japanese translations.
#[allow(clippy::incompatible_msrv)]
static JA_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "ホーム");
    m.insert("nav.dashboard", "ダッシュボード");
    m.insert("nav.documents", "ドキュメント");
    m.insert("nav.search", "検索");
    m.insert("nav.settings", "設定");
    m.insert("nav.catalog", "カタログ");
    m.insert("nav.teams", "チーム");
    m.insert("nav.plugins", "プラグイン");
    m.insert("nav.spaces", "スペース");
    m.insert("nav.graph", "ナレッジグラフ");
    m.insert("nav.tags", "タグ");
    m.insert("nav.billing", "請求");
    m.insert("nav.ssg", "静的サイト");
    m.insert("nav.admin", "管理");
    m.insert("nav.audit", "監査ログ");
    m.insert("nav.templates", "テンプレート");
    m.insert("action.save", "保存");
    m.insert("action.cancel", "キャンセル");
    m.insert("action.delete", "削除");
    m.insert("action.create", "作成");
    m.insert("action.edit", "編集");
    m.insert("action.search", "検索");
    m.insert("action.sign_in", "サインイン");
    m.insert("action.sign_out", "サインアウト");
    m.insert("action.register", "登録");
    m.insert("common.loading", "読み込み中...");
    m.insert("common.error", "エラーが発生しました");
    m.insert("common.no_results", "結果が見つかりません");
    m.insert("common.confirm_delete", "本当に削除しますか？");
    m.insert("docs.new", "新規ドキュメント");
    m.insert("docs.empty", "ドキュメントがありません");
    m.insert("docs.not_found", "ドキュメントが見つかりません");
    m.insert("docs.draft", "下書き");
    m.insert("docs.published", "公開済み");
    m.insert("docs.archived", "アーカイブ");
    m
});

/// Get translations for a locale (falls back to English).
///
/// Reserved for future use: i18n translation lookup.
fn translations_for(locale: Locale) -> &'static HashMap<&'static str, &'static str> {
    match locale {
        Locale::Zh => &ZH_TRANSLATIONS,
        Locale::Ja => &JA_TRANSLATIONS,
        _ => &EN_TRANSLATIONS, // English + all other locales fall back
    }
}

/// Look up a translation by key.
///
/// Reserved for future use: i18n string resolution.
pub fn t(locale: Locale, key: TranslationKey) -> &'static str {
    translations_for(locale)
        .get(key.0)
        .copied()
        .unwrap_or_else(|| {
            // Fallback to English if key missing in current locale
            EN_TRANSLATIONS.get(key.0).copied().unwrap_or(key.0)
        })
}

/// Reactive translation helper.
///
/// Returns a Leptos signal carrying the current locale. Components that
/// need translated strings should read this signal and pass the locale
/// code to the `t()` function.
///
/// Usage in a component:
/// ```rust,ignore
/// let locale = use_locale();
/// view! { <span>{t(locale.get(), TranslationKey::HOME)}</span> }
/// ```
pub fn use_locale() -> ReadSignal<Locale> {
    let (locale, set_locale) = signal(Locale::detect_from_browser());

    // Persist preference
    let stored = crate::storage::get_locale();
    set_locale.set(Locale::from_code(&stored));

    // Effect: save on change
    Effect::new(move |_| {
        let lang = locale.get().code().to_string();
        crate::storage::set_locale(&lang);
    });

    locale
}
