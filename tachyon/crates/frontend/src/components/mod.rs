// Components module

pub mod activity_feed;
pub mod app_shell;
pub mod client_search;
pub mod attachments;
pub mod auth_guard;
pub mod breadcrumbs;
pub mod catalog;
pub mod collaborative_cursors;
#[allow(dead_code)]
pub mod presence_indicators;
pub mod command_palette;
pub mod common;
pub mod conflict_resolver;
pub mod editor_preview;
pub mod editor_search;
#[allow(dead_code)]
pub mod editor_settings;
#[allow(dead_code)]
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

#[allow(unused_imports)]
pub use activity_feed::{Activity, ActivityFeed, ActivityType};
pub use app_shell::AppShell;
pub use client_search::ClientSearch;
pub use auth_guard::{AuthGuard, provide_auth_context};
pub use breadcrumbs::{BreadcrumbItem, Breadcrumbs};
#[allow(unused_imports)]
pub use collaborative_cursors::{AwarenessState, CollaborativeCursors, RemoteCursor};
#[allow(unused_imports)]
pub use command_palette::{CommandItem, CommandPalette};
#[allow(unused_imports)]
pub use presence_indicators::{PresenceIndicators as CollabPresenceIndicators, PresenceUser as CollabPresenceUser};
pub use conflict_resolver::ConflictResolver;
pub use editor_preview::EditorPreview;
pub use editor_search::EditorSearch;
#[allow(unused_imports)]
pub use editor_settings::{EditorSettings, EditorSettingsData};
#[allow(unused_imports)]
pub use editor_split::{EditorSplit, SplitMode};
pub use editor_toolbar::EditorToolbar;
#[allow(unused_imports)]
pub use empty_state::{EmptyDocuments, EmptyNotifications, EmptySearch};
pub use error_boundary::AppErrorBoundary;
#[allow(unused_imports)]
pub use layout::*;
#[allow(unused_imports)]
pub use loading::{ButtonSpinner, InlineLoading, LoadingPage, LoadingSpinner};
pub use native_editor::NativeEditor;
#[allow(unused_imports)]
pub use markdown_preview::MarkdownPreview;
#[allow(unused_imports)]
pub use onboarding::{should_show_onboarding, OnboardingWizard};
pub use review_panel::ReviewPanel;
#[allow(unused_imports)]
pub use skeleton::{SkeletonCard, SkeletonDocumentList, SkeletonEditor, SkeletonTable, SkeletonText};
#[allow(unused_imports)]
pub use table_of_contents::{Heading, TableOfContents};
pub use template_selector::TemplateSelector;
pub use version_history::VersionHistory;
