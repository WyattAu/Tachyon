// OpenAPI Documentation Module
// Provides OpenAPI specification generation and Swagger UI

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Import all route types for documentation

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Tachyon API",
        version = "1.0.0",
        description = "Tachyon Knowledge Management System API\n\nA comprehensive API for managing documents, knowledge graphs, projects, and collaborative workflows.",
        contact(
            name = "Tachyon Support",
            email = "support@tachyon.local"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Development server"),
        (url = "http://localhost:8080/api/v1", description = "API v1 endpoint")
    ),
    tags(
        (name = "documents", description = "Document management endpoints"),
        (name = "search", description = "Search endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "spaces", description = "Space management endpoints"),
        (name = "templates", description = "Document template endpoints"),
        (name = "versions", description = "Document version management"),
        (name = "attachments", description = "Document attachment endpoints"),
        (name = "rendering", description = "Content rendering endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "sessions", description = "Session management endpoints"),
        (name = "projects", description = "Project/Service catalog endpoints"),
        (name = "components", description = "Component catalog endpoints"),
        (name = "nodes", description = "Knowledge graph node endpoints"),
        (name = "edges", description = "Knowledge graph edge endpoints"),
        (name = "graph", description = "Graph query endpoints"),
        (name = "repositories", description = "Repository management endpoints"),
    ),
    paths(
        crate::routes::document::document_crud::list_documents,
        crate::routes::document::document_crud::create_document,
        crate::routes::document::document_crud::get_document,
        crate::routes::document::document_crud::update_document,
        crate::routes::document::document_crud::delete_document,
        crate::routes::search::search,
        crate::routes::search::create_saved_search,
        crate::routes::search::list_saved_searches,
        crate::routes::search::get_saved_search,
        crate::routes::search::update_saved_search,
        crate::routes::search::delete_saved_search,
        crate::routes::user::list_users,
        crate::routes::user::create_user,
        crate::routes::user::get_user,
        crate::routes::user::update_user,
        crate::routes::user::get_me,
        crate::routes::space::list_spaces,
        crate::routes::space::create_space,
        crate::routes::space::get_space,
        crate::routes::space::update_space,
        crate::routes::space::delete_space,
    ),
    components(schemas(
        tachyon_database::types::DocumentMetadata,
        tachyon_database::types::RepositoryMetadata,
        tachyon_database::types::SessionRecord,
        tachyon_database::types::Project,
        tachyon_database::types::ProjectMember,
        tachyon_database::types::Component,
        tachyon_database::Space,
        tachyon_database::UserRecord,
        crate::routes::document::DocumentResponse,
        crate::routes::document::DocumentSearchResponse,
        crate::routes::document::document_crud::CreateDocumentRequest,
        crate::routes::document::document_crud::UpdateDocumentRequest,
        crate::routes::search::SearchResultsResponse,
        crate::routes::search::SavedSearchResponse,
        crate::routes::search::CreateSavedSearchBody,
        crate::routes::search::UpdateSavedSearchBody,
        crate::routes::user::UserResponse,
        crate::routes::user::UserListResponse,
        crate::routes::user::CreateUserRequest,
        crate::routes::user::UpdateUserRequest,
        crate::routes::space::SpaceResponse,
        crate::routes::space::CreateSpaceBody,
        crate::routes::space::UpdateSpaceBody,
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}

/// Create Swagger UI instance for merging into router
pub fn create_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = ApiDoc::openapi();
        let json = spec.to_pretty_json().expect("Failed to generate JSON");
        assert!(json.contains("Tachyon API"));
        assert!(json.contains("documents"));
        assert!(json.contains("users"));
        assert!(json.contains("auth"));
    }

    #[test]
    fn test_openapi_spec_has_security_schemes() {
        let spec = ApiDoc::openapi();
        let json = spec.to_pretty_json().expect("Failed to generate JSON");
        assert!(json.contains("bearer_auth"));
        assert!(json.contains("api_key"));
    }

    #[test]
    fn test_openapi_spec_has_all_tags() {
        let spec = ApiDoc::openapi();
        let json = spec.to_pretty_json().expect("Failed to generate JSON");
        assert!(json.contains("documents"));
        assert!(json.contains("users"));
        assert!(json.contains("auth"));
        assert!(json.contains("sessions"));
        assert!(json.contains("nodes"));
        assert!(json.contains("repositories"));
        assert!(json.contains("projects"));
    }
}
