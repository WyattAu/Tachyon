//! Plugin Marketplace -- discovery, installation, and lifecycle management.
//!
//! Provides a local registry for plugin metadata, compatibility checking,
//! and installation tracking. Remote registry integration (HTTP API client)
//! is gated behind the `remote-registry` feature flag.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use utoipa::ToSchema;

use crate::signing::PluginSignature;

/// Unique plugin identifier (e.g., "org.example/my-plugin").
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
pub struct PluginId(pub String);

/// Semantic version (major.minor.patch).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl PluginVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    /// Returns true if this version is compatible with the given range (semver-compatible major).
    pub fn is_compatible_with(&self, other: &PluginVersion) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for PluginVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Plugin compatibility requirements.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginCompatibility {
    /// Minimum Tachyon version required (semver string).
    pub min_tachyon_version: String,
    /// Required runtime capabilities.
    pub required_capabilities: Vec<String>,
    /// Whether the plugin needs network access.
    pub requires_network: bool,
    /// Whether the plugin needs filesystem access.
    pub requires_filesystem: bool,
}

/// Published plugin metadata (from registry).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginManifest {
    /// Unique plugin identifier.
    pub id: PluginId,
    /// Human-readable name.
    pub name: String,
    /// Plugin description.
    pub description: String,
    /// Author information.
    pub author: String,
    /// License (SPDX identifier).
    pub license: String,
    /// Homepage URL.
    pub homepage_url: Option<String>,
    /// Repository URL.
    pub repository_url: Option<String>,
    /// Current version.
    pub version: PluginVersion,
    /// Compatibility requirements.
    pub compatibility: PluginCompatibility,
    /// Extension points this plugin hooks into.
    pub extension_points: Vec<String>,
    /// Tags for discovery.
    pub tags: Vec<String>,
    /// SHA-256 checksum of the WASM binary.
    pub wasm_checksum: String,
    /// Download size in bytes.
    pub wasm_size_bytes: u64,
    /// Whether this plugin is featured/recommended.
    pub featured: bool,
    /// Total download count.
    pub download_count: u64,
    /// Optional Ed25519 signature of the WASM binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<PluginSignature>,
}

/// Installation status of a plugin.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum PluginInstallStatus {
    /// Not installed.
    NotInstalled,
    /// Installed and enabled.
    Installed,
    /// Installed but disabled.
    Disabled,
    /// Update available.
    UpdateAvailable {
        installed: PluginVersion,
        latest: PluginVersion,
    },
}

/// Error types for marketplace operations.
#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Version conflict: {0}")]
    VersionConflict(String),
    #[error("Compatibility check failed: {0}")]
    Incompatible(String),
    #[error(
        "Checksum verification failed for plugin {plugin_id}: expected {expected}, got {actual}"
    )]
    ChecksumMismatch {
        plugin_id: String,
        expected: String,
        actual: String,
    },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type MarketplaceResult<T> = Result<T, MarketplaceError>;

/// Local plugin marketplace registry.
///
/// Manages installed plugins, their manifests, and compatibility checking.
/// Designed to work with a remote registry API (behind feature gate).
pub struct PluginMarketplace {
    /// Installed plugin manifests keyed by plugin ID.
    installed: HashMap<PluginId, PluginManifest>,
    /// Directory where plugin WASM binaries are stored.
    plugins_dir: PathBuf,
    /// Current Tachyon version (for compatibility checking).
    tachyon_version: String,
}

impl PluginMarketplace {
    /// Create a new marketplace instance.
    pub fn new(plugins_dir: PathBuf, tachyon_version: String) -> Self {
        Self {
            installed: HashMap::new(),
            plugins_dir,
            tachyon_version,
        }
    }

    /// Get the installation status of a plugin.
    pub fn install_status(&self, plugin_id: &PluginId) -> PluginInstallStatus {
        match self.installed.get(plugin_id) {
            None => PluginInstallStatus::NotInstalled,
            Some(_) => PluginInstallStatus::Installed,
        }
    }

    /// List all installed plugins.
    pub fn list_installed(&self) -> Vec<&PluginManifest> {
        self.installed.values().collect()
    }

    /// Check if a plugin is compatible with the current Tachyon version.
    pub fn check_compatibility(&self, manifest: &PluginManifest) -> MarketplaceResult<()> {
        let min_version = semver_parse(&manifest.compatibility.min_tachyon_version)?;
        let current = semver_parse(&self.tachyon_version)?;
        if current < min_version {
            return Err(MarketplaceError::Incompatible(format!(
                "Plugin {} requires Tachyon >= {}, current: {}",
                manifest.id.0, manifest.compatibility.min_tachyon_version, self.tachyon_version
            )));
        }
        Ok(())
    }

    /// Register an installed plugin.
    pub fn register_plugin(&mut self, manifest: PluginManifest) -> MarketplaceResult<()> {
        self.check_compatibility(&manifest)?;
        self.installed.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    /// Unregister (uninstall) a plugin.
    pub fn unregister_plugin(&mut self, plugin_id: &PluginId) -> MarketplaceResult<PluginManifest> {
        self.installed
            .remove(plugin_id)
            .ok_or_else(|| MarketplaceError::NotFound(plugin_id.0.clone()))
    }

    /// Get plugin manifest by ID.
    pub fn get_plugin(&self, plugin_id: &PluginId) -> Option<&PluginManifest> {
        self.installed.get(plugin_id)
    }

    /// Search installed plugins by tag.
    pub fn search_by_tag(&self, tag: &str) -> Vec<&PluginManifest> {
        self.installed
            .values()
            .filter(|p| p.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }

    /// Get the WASM file path for an installed plugin.
    pub fn wasm_path(&self, plugin_id: &PluginId) -> Option<PathBuf> {
        self.installed.get(plugin_id).map(|m| {
            let filename = format!("{}.wasm", m.id.0.replace('/', "-"));
            self.plugins_dir.join(filename)
        })
    }

    /// Verify a WASM binary checksum.
    pub fn verify_checksum(
        &self,
        plugin_id: &PluginId,
        actual_checksum: &str,
    ) -> MarketplaceResult<()> {
        let manifest = self
            .get_plugin(plugin_id)
            .ok_or_else(|| MarketplaceError::NotFound(plugin_id.0.clone()))?;
        if manifest.wasm_checksum != actual_checksum {
            return Err(MarketplaceError::ChecksumMismatch {
                plugin_id: plugin_id.0.clone(),
                expected: manifest.wasm_checksum.clone(),
                actual: actual_checksum.to_string(),
            });
        }
        Ok(())
    }
}

/// Simple semver parse helper (major.minor.patch) -> (u32, u32, u32).
fn semver_parse(s: &str) -> MarketplaceResult<(u32, u32, u32)> {
    let parts: Vec<&str> = s.trim_start_matches('v').split('.').collect();
    if parts.len() != 3 {
        return Err(MarketplaceError::VersionConflict(format!(
            "Invalid semver: {s}"
        )));
    }
    Ok((
        parts[0].parse().map_err(|_| {
            MarketplaceError::VersionConflict(format!("Invalid major: {}", parts[0]))
        })?,
        parts[1].parse().map_err(|_| {
            MarketplaceError::VersionConflict(format!("Invalid minor: {}", parts[1]))
        })?,
        parts[2].parse().map_err(|_| {
            MarketplaceError::VersionConflict(format!("Invalid patch: {}", parts[2]))
        })?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest(id: &str) -> PluginManifest {
        PluginManifest {
            id: PluginId(id.to_string()),
            name: "Test Plugin".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage_url: None,
            repository_url: None,
            version: PluginVersion::new(1, 0, 0),
            compatibility: PluginCompatibility {
                min_tachyon_version: "0.50.0".to_string(),
                required_capabilities: vec![],
                requires_network: false,
                requires_filesystem: false,
            },
            extension_points: vec!["document.render".to_string()],
            tags: vec!["rendering".to_string(), "test".to_string()],
            wasm_checksum: "abc123".to_string(),
            wasm_size_bytes: 1024,
            featured: false,
            download_count: 42,
            signature: None,
        }
    }

    #[test]
    fn test_plugin_version_parse() {
        let v = PluginVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_plugin_version_parse_invalid() {
        assert!(PluginVersion::parse("1.2").is_none());
        assert!(PluginVersion::parse("abc").is_none());
        assert!(PluginVersion::parse("").is_none());
    }

    #[test]
    fn test_plugin_version_compatibility() {
        let v1 = PluginVersion::new(1, 0, 0);
        let v2 = PluginVersion::new(1, 5, 0);
        let v3 = PluginVersion::new(2, 0, 0);
        assert!(v1.is_compatible_with(&v2));
        assert!(v2.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_marketplace_register_and_list() {
        let mut mp = PluginMarketplace::new(PathBuf::from("/tmp/plugins"), "0.56.0".to_string());
        let manifest = test_manifest("test/plugin-a");
        mp.register_plugin(manifest).unwrap();
        assert_eq!(mp.list_installed().len(), 1);
        assert_eq!(
            mp.install_status(&PluginId("test/plugin-a".to_string())),
            PluginInstallStatus::Installed
        );
        assert_eq!(
            mp.install_status(&PluginId("nonexistent".to_string())),
            PluginInstallStatus::NotInstalled
        );
    }

    #[test]
    fn test_marketplace_compatibility_check() {
        let mp = PluginMarketplace::new(PathBuf::from("/tmp/plugins"), "0.50.0".to_string());
        let compatible = test_manifest("test/compatible");
        mp.check_compatibility(&compatible).unwrap();

        let mut incompatible = test_manifest("test/incompatible");
        incompatible.compatibility.min_tachyon_version = "99.0.0".to_string();
        assert!(mp.check_compatibility(&incompatible).is_err());
    }

    #[test]
    fn test_marketplace_search_by_tag() {
        let mut mp = PluginMarketplace::new(PathBuf::from("/tmp/plugins"), "0.56.0".to_string());
        mp.register_plugin(test_manifest("test/render-a")).unwrap();
        let mut render_b = test_manifest("test/render-b");
        render_b.tags = vec!["rendering".to_string(), "advanced".to_string()];
        mp.register_plugin(render_b).unwrap();
        let mut search_c = test_manifest("test/search-c");
        search_c.tags = vec!["search".to_string(), "test".to_string()];
        mp.register_plugin(search_c).unwrap();

        let results = mp.search_by_tag("rendering");
        assert_eq!(results.len(), 2);

        let results = mp.search_by_tag("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_marketplace_unregister() {
        let mut mp = PluginMarketplace::new(PathBuf::from("/tmp/plugins"), "0.56.0".to_string());
        mp.register_plugin(test_manifest("test/plugin-a")).unwrap();
        mp.unregister_plugin(&PluginId("test/plugin-a".to_string()))
            .unwrap();
        assert_eq!(mp.list_installed().len(), 0);
    }

    #[test]
    fn test_marketplace_verify_checksum() {
        let mut mp = PluginMarketplace::new(PathBuf::from("/tmp/plugins"), "0.56.0".to_string());
        mp.register_plugin(test_manifest("test/plugin-a")).unwrap();
        mp.verify_checksum(&PluginId("test/plugin-a".to_string()), "abc123")
            .unwrap();
        assert!(mp
            .verify_checksum(&PluginId("test/plugin-a".to_string()), "wrong")
            .is_err());
    }

    #[test]
    fn test_marketplace_wasm_path() {
        let mut mp = PluginMarketplace::new(PathBuf::from("/tmp/plugins"), "0.56.0".to_string());
        mp.register_plugin(test_manifest("org.example/my-plugin"))
            .unwrap();
        let path = mp
            .wasm_path(&PluginId("org.example/my-plugin".to_string()))
            .unwrap();
        assert!(path
            .to_string_lossy()
            .contains("org.example-my-plugin.wasm"));
    }

    #[test]
    fn test_semver_parse() {
        assert_eq!(semver_parse("1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(semver_parse("v1.2.3").unwrap(), (1, 2, 3));
        assert!(semver_parse("1.2").is_err());
        assert!(semver_parse("invalid").is_err());
    }
}
