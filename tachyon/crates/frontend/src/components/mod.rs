// Components module

pub mod activity_feed;
pub mod app_shell;
pub mod attachments;
pub mod auth_guard;
pub mod document_editor;
pub mod template_selector;
pub mod version_history;

pub use activity_feed::{Activity, ActivityFeed, ActivityFeedCompact, ActivityType};
pub use app_shell::AppShell;
pub use attachments::AttachmentManager;
pub use auth_guard::{AuthGuard, provide_auth_context};
pub use document_editor::{DocumentEditor, PresenceIndicators, PresenceUser};
pub use template_selector::{TemplateCard, TemplateSelector};
pub use version_history::{VersionDiffView, VersionHistory};
