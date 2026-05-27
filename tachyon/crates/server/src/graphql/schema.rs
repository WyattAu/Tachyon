//! GraphQL schema builder.

use async_graphql::{EmptySubscription, Schema};

use super::types::{MutationRoot, QueryRoot};

pub type TachyonSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[cfg(test)]
pub fn build_schema() -> TachyonSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}

pub fn build_schema_with_data(pool: tachyon_database::DatabasePool) -> TachyonSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::Response;

    #[tokio::test]
    async fn test_graphql_health_query() {
        let schema = build_schema();
        let query = "{ health }";
        let response: Response = schema.execute(query).await;
        assert_eq!(response.errors, []);
        match &response.data {
            async_graphql::Value::Object(map) => {
                if let Some(health) = map.get("health") {
                    assert_eq!(health, &async_graphql::Value::String("ok".to_string()));
                } else {
                    panic!("expected 'health' field in response object");
                }
            }
            _ => panic!("expected object response"),
        }
    }

    #[tokio::test]
    async fn test_graphql_introspection() {
        let schema = build_schema();
        let query = r#"
            {
                __schema {
                    types {
                        name
                    }
                    queryType {
                        fields {
                            name
                        }
                    }
                    mutationType {
                        fields {
                            name
                        }
                    }
                }
            }
        "#;
        let response: Response = schema.execute(query).await;
        assert_eq!(response.errors, []);
    }

    #[tokio::test]
    async fn test_graphql_document_query_requires_pool() {
        let schema = build_schema();
        let query = r#"{ document(id: "123") { id title } }"#;
        let response: Response = schema.execute(query).await;
        assert!(!response.errors.is_empty());
    }

    #[tokio::test]
    async fn test_graphql_search_requires_pool() {
        let schema = build_schema();
        let query = r#"{ search(query: "test") { total results { id title } } }"#;
        let response: Response = schema.execute(query).await;
        assert!(!response.errors.is_empty());
    }
}
