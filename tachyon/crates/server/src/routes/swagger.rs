use axum::{response::Html, routing::get, Json, Router};
use tracing::error;

pub fn routes() -> Router {
    Router::new()
        .route("/swagger-ui", get(swagger_ui_html))
        .route("/api/v1/openapi.json", get(openapi_json))
}

async fn swagger_ui_html() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

async fn openapi_json() -> Json<serde_json::Value> {
    let spec = crate::api_docs::openapi_spec();
    match serde_json::to_value(spec) {
        Ok(value) => Json(value),
        Err(e) => {
            error!("Failed to serialize OpenAPI spec: {}", e);
            Json(serde_json::json!({"error": "failed to serialize OpenAPI spec"}))
        }
    }
}

const SWAGGER_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tachyon API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
    <style>
        body { margin: 0; padding: 0; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: "/api/v1/openapi.json",
            dom_id: "#swagger-ui",
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.SwaggerUIStandalonePreset
            ],
            layout: "StandaloneLayout"
        });
    </script>
</body>
</html>"##;
