use std::fs;
use std::path::Path;

pub fn new_plugin(name: &str) -> Result<(), String> {
    let dir = Path::new(name);
    if dir.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }

    fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        name = name
    );

    let lib_rs = r##"use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PluginInput {
    content: String,
}

#[derive(Serialize)]
struct PluginOutput {
    result: String,
}

#[no_mangle]
pub extern "C" fn on_document_save(input: &str) -> String {
    let parsed: PluginInput = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return r#"{"error":"invalid input"}"#.to_string(),
    };
    let output = PluginOutput {
        result: format!("Processed: {}", parsed.content.len()),
    };
    serde_json::to_string(&output).unwrap_or_default()
}
"##;

    fs::write(dir.join("Cargo.toml"), cargo_toml).map_err(|e| e.to_string())?;
    fs::write(dir.join("src/lib.rs"), lib_rs).map_err(|e| e.to_string())?;

    println!("Created plugin '{}' with WASM target", name);
    println!(
        "Build with: cd {} && cargo build --target wasm32-unknown-unknown",
        name
    );
    Ok(())
}
