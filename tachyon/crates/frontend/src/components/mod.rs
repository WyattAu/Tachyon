// Components module

pub mod activity_feed;
pub mod app_shell;
pub mod attachments;
pub mod auth_guard;
pub mod breadcrumbs;
pub mod catalog;
pub mod client_search;
pub mod collaborative_cursors;
pub mod command_palette;
pub mod common;
pub mod conflict_resolver;
pub mod drop_zone;
pub mod editor_preview;
pub mod editor_search;
pub mod editor_settings;
pub mod editor_split;
pub mod editor_toolbar;
pub mod empty_state;
pub mod error_boundary;
pub mod image_preview;
pub mod layout;
pub mod loading;
pub mod markdown_preview;
pub mod mobile_nav;
pub mod native_editor;
pub mod onboarding;
pub mod presence_indicators;
pub mod review_panel;
pub mod role_badge;
pub mod skeleton;
pub mod table_of_contents;
pub mod template_selector;
pub mod theme_toggle;
pub mod update_banner;
pub mod upload_progress;
pub mod user_avatar;
pub mod version_history;
pub mod wikilink_autocomplete;

pub use activity_feed::{Activity, ActivityFeed};
pub use app_shell::AppShell;
pub use auth_guard::{provide_auth_context, AuthGuard};
pub use breadcrumbs::{BreadcrumbItem, Breadcrumbs};
pub use client_search::ClientSearch;
pub use command_palette::CommandPalette;
pub use conflict_resolver::ConflictResolver;
#[allow(unused_imports)]
pub use drop_zone::DropZone;
#[allow(unused_imports)]
pub use drop_zone::DroppedFile;
pub use editor_preview::EditorPreview;
pub use editor_search::EditorSearch;
#[allow(unused_imports)]
pub use editor_settings::{EditorSettings, EditorSettingsData};
#[allow(unused_imports)]
pub use editor_split::{EditorSplit, SplitMode};
pub use editor_toolbar::EditorToolbar;
pub use empty_state::{EmptyDocuments, EmptySearch};
pub use error_boundary::AppErrorBoundary;
pub use loading::ButtonSpinner;
pub use markdown_preview::MarkdownPreview;
#[allow(unused_imports)]
pub use mobile_nav::MobileNav;
pub use native_editor::NativeEditor;
pub use onboarding::{should_show_onboarding, OnboardingWizard};
#[allow(unused_imports)]
pub use presence_indicators::{
    PresenceIndicators as CollabPresenceIndicators, PresenceUser as CollabPresenceUser,
};
pub use review_panel::ReviewPanel;
pub use table_of_contents::TableOfContents;
pub use template_selector::TemplateSelector;
#[allow(unused_imports)]
pub use theme_toggle::{
    get_current_theme, get_current_theme_label, Theme, ThemeInitializer, ThemeToggle,
};
#[allow(unused_imports)]
pub use upload_progress::{UploadItem, UploadProgress, UploadStatus};
#[allow(unused_imports)]
pub use user_avatar::UserAvatar;
pub use version_history::VersionHistory;
#[allow(unused_imports)]
pub use wikilink_autocomplete::{WikilinkAutocomplete, WikilinkCompletion};
