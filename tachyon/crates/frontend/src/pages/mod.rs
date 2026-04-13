// Pages module - all application pages

pub mod graph;
pub mod home;
pub mod documents;
pub mod register;
pub mod search;
pub mod catalog;
pub mod settings;
pub mod login;
pub mod dashboard;
pub mod teams;
pub mod tags;
pub mod admin;
pub mod templates;

pub use graph::GraphPage;
pub use home::HomePage;
pub use documents::{DocumentsPage, DocumentEditPage, DocumentPage};
pub use register::RegisterPage;
pub use search::SearchPage;
pub use catalog::CatalogPage;
pub use settings::SettingsPage;
pub use login::LoginPage;
pub use dashboard::DashboardPage;
pub use teams::TeamsPage;
pub use tags::TagsPage;
pub use admin::roles::RolesPage;
pub use templates::TemplatesPage;
