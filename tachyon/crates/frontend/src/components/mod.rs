// Components module

pub mod activity_feed;
pub mod app_shell;
pub mod attachments;
pub mod auth_guard;
pub mod conflict_resolver;
pub mod document_editor;
pub mod review_panel;
pub mod template_selector;
pub mod version_history;

pub use activity_feed::{Activity, ActivityFeed, ActivityType};
pub use app_shell::AppShell;
pub use auth_guard::{AuthGuard, provide_auth_context};
#[allow(unused_imports)]
pub use conflict_resolver::ConflictResolver;
pub use document_editor::DocumentEditor;
#[allow(unused_imports)] // Will be wired into documents page
pub use review_panel::ReviewPanel;
pub use template_selector::TemplateSelector;
pub use version_history::VersionHistory;
