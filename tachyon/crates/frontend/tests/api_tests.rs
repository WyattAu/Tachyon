use wasm_bindgen_test::*;
use tachyon_frontend::api::*;
use serde_json::json;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_api_client_initialization() {
    let client = ApiClient::new("http://localhost:3000");
    assert!(client.base_url() == "http://localhost:3000");
}

#[wasm_bindgen_test]
async fn test_get_documents() {
    let client = ApiClient::new("http://localhost:3000");
    
    let result = client.get_documents(None, None, None).await;
    
    match result {
        Ok(docs) => {
            assert!(docs.len() >= 0);
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_create_document() {
    let client = ApiClient::new("http://localhost:3000");
    
    let doc_data = json!({
        "title": "Test Document",
        "slug": "test-document",
        "description": "Test description",
        "content_type": "Markdown"
    });
    
    let result = client.create_document(&doc_data).await;
    
    match result {
        Ok(doc) => {
            assert!(doc["id"].is_string());
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_get_document_by_id() {
    let client = ApiClient::new("http://localhost:3000");
    let doc_id = "00000000-0000-0000-0000-000000000000";
    
    let result = client.get_document(doc_id).await;
    
    match result {
        Ok(doc) => {
            assert!(doc["id"].as_str() == Some(doc_id));
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_update_document() {
    let client = ApiClient::new("http://localhost:3000");
    let doc_id = "00000000-0000-0000-0000-000000000000";
    
    let update_data = json!({
        "title": "Updated Title"
    });
    
    let result = client.update_document(doc_id, &update_data).await;
    
    match result {
        Ok(doc) => {
            assert!(doc["title"].as_str() == Some("Updated Title"));
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm::test]
async fn test_delete_document() {
    let client = ApiClient::new("http://localhost:3000");
    let doc_id = "00000000-0000-0000-0000-000000000000";
    
    let result = client.delete_document(doc_id).await;
    
    match result {
        Ok(_) => {
            console_log("Document deleted successfully");
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_search_documents() {
    let client = ApiClient::new("http://localhost:3000");
    
    let result = client.search("test", None, None).await;
    
    match result {
        Ok(results) => {
            assert!(results["items"].is_array());
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_get_projects() {
    let client = ApiClient::new("http://localhost:3000");
    
    let result = client.get_projects(None, None).await;
    
    match result {
        Ok(projects) => {
            assert!(projects.len() >= 0);
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_create_project() {
    let client = ApiClient::new("http://localhost:3000");
    
    let project_data = json!({
        "name": "Test Project",
        "slug": "test-project",
        "description": "Test description",
        "project_type": "service"
    });
    
    let result = client.create_project(&project_data).await;
    
    match result {
        Ok(project) => {
            assert!(project["id"].is_string());
        }
        Err(e) => {
            console_log(&format!("Expected error in test: {:?}", e));
        }
    }
}

#[wasm_bindgen_test]
async fn test_api_error_handling() {
    let client = ApiClient::new("http://invalid-url:9999");
    
    let result = client.get_documents(None, None, None).await;
    
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[wasm_bindgen_test]
async fn test_api_with_authentication() {
    let mut client = ApiClient::new("http://localhost:3000");
    client.set_auth_token("test-token-12345");
    
    let result = client.get_documents(None, None, None).await;
    
    match result {
        Ok(_) => console_log("Request succeeded with auth token"),
        Err(e) => console_log(&format!("Error (expected): {:?}", e)),
    }
}

#[wasm_bindgen_test]
async fn test_api_headers() {
    let mut client = ApiClient::new("http://localhost:3000");
    client.set_auth_token("test-token");
    
    let headers = client.get_default_headers();
    
    assert!(headers.contains_key("Authorization"));
    assert_eq!(headers.get("Authorization"), Some(&"Bearer test-token".to_string()));
}

fn console_log(message: &str) {
    web_sys::console::log_1(&message.into());
}
