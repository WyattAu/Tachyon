// Components module

pub mod activity_feed;
pub mod app_shell;
pub mod attachments;
pub mod auth_guard;
pub mod breadcrumbs;
pub mod catalog;
pub mod command_palette;
pub mod common;
pub mod conflict_resolver;
pub mod document_editor;
pub mod error_boundary;
pub mod layout;
pub mod prose_mirror_editor;
pub mod review_panel;
pub mod role_badge;
pub mod table_of_contents;
pub mod template_selector;
pub mod version_history;

pub use activity_feed::{Activity, ActivityFeed, ActivityType};
pub use app_shell::AppShell;
pub use auth_guard::{AuthGuard, provide_auth_context};
pub use breadcrumbs::{BreadcrumbItem, Breadcrumbs};
#[allow(unused_imports)]
pub use command_palette::{CommandItem, CommandPalette};
pub use conflict_resolver::ConflictResolver;
pub use document_editor::DocumentEditor;
pub use error_boundary::AppErrorBoundary;
pub use review_panel::ReviewPanel;
#[allow(unused_imports)]
pub use table_of_contents::{Heading, TableOfContents};
pub use template_selector::TemplateSelector;
pub use version_history::VersionHistory;
