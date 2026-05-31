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

/// German translations.
#[allow(clippy::incompatible_msrv)]
static DE_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "Startseite");
    m.insert("nav.dashboard", "Dashboard");
    m.insert("nav.documents", "Dokumente");
    m.insert("nav.search", "Suche");
    m.insert("nav.settings", "Einstellungen");
    m.insert("nav.catalog", "Katalog");
    m.insert("nav.teams", "Teams");
    m.insert("nav.plugins", "Plugins");
    m.insert("nav.spaces", "Bereiche");
    m.insert("nav.graph", "Wissensgraph");
    m.insert("nav.tags", "Tags");
    m.insert("nav.billing", "Abrechnung");
    m.insert("nav.ssg", "Statische Website");
    m.insert("nav.admin", "Verwaltung");
    m.insert("nav.audit", "Audit-Log");
    m.insert("nav.templates", "Vorlagen");
    m.insert("action.save", "Speichern");
    m.insert("action.cancel", "Abbrechen");
    m.insert("action.delete", "Loschen");
    m.insert("action.create", "Erstellen");
    m.insert("action.edit", "Bearbeiten");
    m.insert("action.search", "Suchen");
    m.insert("action.sign_in", "Anmelden");
    m.insert("action.sign_out", "Abmelden");
    m.insert("action.register", "Registrieren");
    m.insert("common.loading", "Laden...");
    m.insert("common.error", "Etwas ist schiefgelaufen");
    m.insert("common.no_results", "Keine Ergebnisse gefunden");
    m.insert("common.confirm_delete", "Mochten Sie das wirklich loschen?");
    m.insert("docs.new", "Neues Dokument");
    m.insert("docs.empty", "Noch keine Dokumente");
    m.insert("docs.not_found", "Dokument nicht gefunden");
    m.insert("docs.draft", "Entwurf");
    m.insert("docs.published", "Veroffentlicht");
    m.insert("docs.archived", "Archiviert");
    m
});

/// French translations.
#[allow(clippy::incompatible_msrv)]
static FR_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "Accueil");
    m.insert("nav.dashboard", "Tableau de bord");
    m.insert("nav.documents", "Documents");
    m.insert("nav.search", "Recherche");
    m.insert("nav.settings", "Parametres");
    m.insert("nav.catalog", "Catalogue");
    m.insert("nav.teams", "Equipes");
    m.insert("nav.plugins", "Plugins");
    m.insert("nav.spaces", "Espaces");
    m.insert("nav.graph", "Graphe de connaissances");
    m.insert("nav.tags", "Tags");
    m.insert("nav.billing", "Facturation");
    m.insert("nav.ssg", "Site statique");
    m.insert("nav.admin", "Administration");
    m.insert("nav.audit", "Journal d'audit");
    m.insert("nav.templates", "Modeles");
    m.insert("action.save", "Enregistrer");
    m.insert("action.cancel", "Annuler");
    m.insert("action.delete", "Supprimer");
    m.insert("action.create", "Creer");
    m.insert("action.edit", "Modifier");
    m.insert("action.search", "Rechercher");
    m.insert("action.sign_in", "Se connecter");
    m.insert("action.sign_out", "Se deconnecter");
    m.insert("action.register", "S'inscrire");
    m.insert("common.loading", "Chargement...");
    m.insert("common.error", "Une erreur est survenue");
    m.insert("common.no_results", "Aucun resultat");
    m.insert(
        "common.confirm_delete",
        "Etes-vous sur de vouloir supprimer ceci ?",
    );
    m.insert("docs.new", "Nouveau document");
    m.insert("docs.empty", "Aucun document pour le moment");
    m.insert("docs.not_found", "Document introuvable");
    m.insert("docs.draft", "Brouillon");
    m.insert("docs.published", "Publie");
    m.insert("docs.archived", "Archive");
    m
});

/// Spanish translations.
#[allow(clippy::incompatible_msrv)]
static ES_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "Inicio");
    m.insert("nav.dashboard", "Panel");
    m.insert("nav.documents", "Documentos");
    m.insert("nav.search", "Buscar");
    m.insert("nav.settings", "Configuracion");
    m.insert("nav.catalog", "Catalogo");
    m.insert("nav.teams", "Equipos");
    m.insert("nav.plugins", "Plugins");
    m.insert("nav.spaces", "Espacios");
    m.insert("nav.graph", "Grafo de conocimientos");
    m.insert("nav.tags", "Etiquetas");
    m.insert("nav.billing", "Facturacion");
    m.insert("nav.ssg", "Sitio estatico");
    m.insert("nav.admin", "Administracion");
    m.insert("nav.audit", "Registro de auditoria");
    m.insert("nav.templates", "Plantillas");
    m.insert("action.save", "Guardar");
    m.insert("action.cancel", "Cancelar");
    m.insert("action.delete", "Eliminar");
    m.insert("action.create", "Crear");
    m.insert("action.edit", "Editar");
    m.insert("action.search", "Buscar");
    m.insert("action.sign_in", "Iniciar sesion");
    m.insert("action.sign_out", "Cerrar sesion");
    m.insert("action.register", "Registrarse");
    m.insert("common.loading", "Cargando...");
    m.insert("common.error", "Algo salio mal");
    m.insert("common.no_results", "Sin resultados");
    m.insert("common.confirm_delete", "Seguro que desea eliminar esto?");
    m.insert("docs.new", "Nuevo documento");
    m.insert("docs.empty", "Sin documentos aun");
    m.insert("docs.not_found", "Documento no encontrado");
    m.insert("docs.draft", "Borrador");
    m.insert("docs.published", "Publicado");
    m.insert("docs.archived", "Archivado");
    m
});

/// Korean translations.
#[allow(clippy::incompatible_msrv)]
static KO_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "홈");
    m.insert("nav.dashboard", "대시보드");
    m.insert("nav.documents", "문서");
    m.insert("nav.search", "검색");
    m.insert("nav.settings", "설정");
    m.insert("nav.catalog", "카탈로그");
    m.insert("nav.teams", "팀");
    m.insert("nav.plugins", "플러그인");
    m.insert("nav.spaces", "스페이스");
    m.insert("nav.graph", "지식 그래프");
    m.insert("nav.tags", "태그");
    m.insert("nav.billing", "결제");
    m.insert("nav.ssg", "정적 사이트");
    m.insert("nav.admin", "관리");
    m.insert("nav.audit", "감사 로그");
    m.insert("nav.templates", "템플릿");
    m.insert("action.save", "저장");
    m.insert("action.cancel", "취소");
    m.insert("action.delete", "삭제");
    m.insert("action.create", "만들기");
    m.insert("action.edit", "편집");
    m.insert("action.search", "검색");
    m.insert("action.sign_in", "로그인");
    m.insert("action.sign_out", "로그아웃");
    m.insert("action.register", "가입");
    m.insert("common.loading", "로딩 중...");
    m.insert("common.error", "오류가 발생했습니다");
    m.insert("common.no_results", "검색 결과가 없습니다");
    m.insert("common.confirm_delete", "정말 삭제하시겠습니까?");
    m.insert("docs.new", "새 문서");
    m.insert("docs.empty", "문서가 아직 없습니다");
    m.insert("docs.not_found", "문서를 찾을 수 없습니다");
    m.insert("docs.draft", "초안");
    m.insert("docs.published", "게시됨");
    m.insert("docs.archived", "보관 처리됨");
    m
});

/// Portuguese translations.
#[allow(clippy::incompatible_msrv)]
static PT_TRANSLATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("nav.home", "Inicio");
    m.insert("nav.dashboard", "Painel");
    m.insert("nav.documents", "Documentos");
    m.insert("nav.search", "Pesquisa");
    m.insert("nav.settings", "Configuracoes");
    m.insert("nav.catalog", "Catalogo");
    m.insert("nav.teams", "Equipes");
    m.insert("nav.plugins", "Plugins");
    m.insert("nav.spaces", "Espacos");
    m.insert("nav.graph", "Grafo de conhecimento");
    m.insert("nav.tags", "Tags");
    m.insert("nav.billing", "Faturamento");
    m.insert("nav.ssg", "Site estatico");
    m.insert("nav.admin", "Administracao");
    m.insert("nav.audit", "Registro de auditoria");
    m.insert("nav.templates", "Modelos");
    m.insert("action.save", "Salvar");
    m.insert("action.cancel", "Cancelar");
    m.insert("action.delete", "Excluir");
    m.insert("action.create", "Criar");
    m.insert("action.edit", "Editar");
    m.insert("action.search", "Pesquisar");
    m.insert("action.sign_in", "Entrar");
    m.insert("action.sign_out", "Sair");
    m.insert("action.register", "Registrar");
    m.insert("common.loading", "Carregando...");
    m.insert("common.error", "Algo deu errado");
    m.insert("common.no_results", "Nenhum resultado encontrado");
    m.insert(
        "common.confirm_delete",
        "Tem certeza que deseja excluir isso?",
    );
    m.insert("docs.new", "Novo documento");
    m.insert("docs.empty", "Nenhum documento ainda");
    m.insert("docs.not_found", "Documento nao encontrado");
    m.insert("docs.draft", "Rascunho");
    m.insert("docs.published", "Publicado");
    m.insert("docs.archived", "Arquivado");
    m
});

/// Get translations for a locale (falls back to English).
///
/// Reserved for future use: i18n translation lookup.
fn translations_for(locale: Locale) -> &'static HashMap<&'static str, &'static str> {
    match locale {
        Locale::En => &EN_TRANSLATIONS,
        Locale::Zh => &ZH_TRANSLATIONS,
        Locale::Ja => &JA_TRANSLATIONS,
        Locale::De => &DE_TRANSLATIONS,
        Locale::Fr => &FR_TRANSLATIONS,
        Locale::Es => &ES_TRANSLATIONS,
        Locale::Ko => &KO_TRANSLATIONS,
        Locale::Pt => &PT_TRANSLATIONS,
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
