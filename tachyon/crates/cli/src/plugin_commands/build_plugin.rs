use std::process::Command;

pub fn build_plugin(release: bool) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--target", "wasm32-unknown-unknown"]);
    if release {
        cmd.arg("--release");
    }

    let status = cmd
        .current_dir(".")
        .status()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if status.success() {
        println!("Plugin built successfully for wasm32-unknown-unknown");
        Ok(())
    } else {
        Err("Plugin build failed".to_string())
    }
}
