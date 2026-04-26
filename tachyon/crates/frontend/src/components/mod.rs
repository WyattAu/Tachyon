// Components module

pub mod activity_feed;
pub mod app_shell;
pub mod client_search;
pub mod attachments;
pub mod auth_guard;
pub mod breadcrumbs;
pub mod catalog;
pub mod collaborative_cursors;
#[cfg(feature = "staging")]
pub mod presence_indicators;
pub mod command_palette;
pub mod common;
pub mod conflict_resolver;
#[cfg(feature = "staging")]
pub mod editor_preview;
pub mod editor_search;
#[cfg(feature = "staging")]
pub mod editor_settings;
#[cfg(feature = "staging")]
pub mod editor_split;
pub mod editor_toolbar;
pub mod empty_state;
pub mod error_boundary;
pub mod layout;
pub mod loading;
pub mod native_editor;
pub mod onboarding;
pub mod review_panel;
pub mod role_badge;
pub mod skeleton;
pub mod table_of_contents;
pub mod template_selector;
pub mod markdown_preview;
pub mod update_banner;
pub mod version_history;

pub use activity_feed::{Activity, ActivityFeed};
pub use app_shell::AppShell;
pub use client_search::ClientSearch;
pub use auth_guard::{AuthGuard, provide_auth_context};
pub use breadcrumbs::{BreadcrumbItem, Breadcrumbs};
pub use command_palette::CommandPalette;
pub use conflict_resolver::ConflictResolver;
pub use editor_search::EditorSearch;
pub use editor_toolbar::EditorToolbar;
pub use empty_state::{EmptyDocuments, EmptySearch};
pub use error_boundary::AppErrorBoundary;
pub use loading::ButtonSpinner;
pub use native_editor::NativeEditor;
pub use markdown_preview::MarkdownPreview;
pub use onboarding::{should_show_onboarding, OnboardingWizard};
pub use review_panel::ReviewPanel;
pub use template_selector::TemplateSelector;
pub use version_history::VersionHistory;
#[cfg(feature = "staging")]
pub use editor_preview::EditorPreview;
#[cfg(feature = "staging")]
pub use editor_settings::{EditorSettings, EditorSettingsData};
#[cfg(feature = "staging")]
pub use editor_split::{EditorSplit, SplitMode};
#[cfg(feature = "staging")]
pub use presence_indicators::{PresenceIndicators as CollabPresenceIndicators, PresenceUser as CollabPresenceUser};
pub use table_of_contents::TableOfContents;
