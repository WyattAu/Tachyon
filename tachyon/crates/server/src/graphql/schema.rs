//! GraphQL schema builder.

use async_graphql::{EmptySubscription, Schema};

use super::types::{MutationRoot, QueryRoot};

pub type TachyonSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

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
                let health = map.get("health").expect("health field missing");
                assert_eq!(health, &async_graphql::Value::String("ok".to_string()));
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
    async fn test_graphql_document_query_placeholder() {
        let schema = build_schema();
        let query = r#"{ document(id: "123") { id title } }"#;
        let response: Response = schema.execute(query).await;
        assert!(!response.errors.is_empty());
    }

    #[tokio::test]
    async fn test_graphql_search_placeholder() {
        let schema = build_schema();
        let query = r#"{ search(query: "test") { total results { id title } } }"#;
        let response: Response = schema.execute(query).await;
        assert!(!response.errors.is_empty());
    }
}
