// Pages module - all application pages

pub mod home;
pub mod documents;
pub mod search;
pub mod catalog;
pub mod settings;
pub mod login;
pub mod dashboard;
pub mod teams;
pub mod admin;

pub use home::HomePage;
pub use documents::{DocumentsPage, DocumentEditPage};
pub use search::SearchPage;
pub use catalog::CatalogPage;
pub use settings::SettingsPage;
pub use login::LoginPage;
pub use dashboard::DashboardPage;
pub use teams::TeamsPage;
pub use admin::roles::RolesPage;
