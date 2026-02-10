use crate::error::{Result, S1bCr4ftError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration version (e.g., "1.0")
    pub version: String,

    /// Project name
    pub name: String,

    /// Project description
    #[serde(default)]
    pub description: String,

    /// List of modules to install
    pub modules: Vec<String>,

    /// Dotfile configurations
    #[serde(default)]
    pub dotfiles: Vec<DotfileEntry>,

    /// Pre/post sync hooks
    #[serde(default)]
    pub hooks: Hooks,

    /// Configuration options
    #[serde(default)]
    pub options: ConfigOptions,

    /// Security settings
    #[serde(default)]
    pub security: SecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfileEntry {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hooks {
    #[serde(default)]
    pub pre_sync: Option<String>,

    #[serde(default)]
    pub post_sync: Option<String>,

    #[serde(default)]
    pub pre_module: Option<String>,

    #[serde(default)]
    pub post_module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOptions {
    #[serde(default = "default_true")]
    pub auto_backup: bool,

    #[serde(default)]
    pub dry_run: bool,

    #[serde(default = "default_true")]
    pub parallel_install: bool,

    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            auto_backup: true,
            dry_run: false,
            parallel_install: true,
            custom: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecuritySettings {
    #[serde(default)]
    pub isolation_level: Option<String>,

    #[serde(default)]
    pub network_isolation: bool,

    #[serde(default)]
    pub container_sandbox: Option<String>,

    #[serde(default)]
    pub gpg_signing: bool,
}

fn default_true() -> bool {
    true
}

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| S1bCr4ftError::config(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_yaml::from_str(&content)?;

        // Validate version
        if config.version != "1.0" {
            return Err(S1bCr4ftError::config(format!(
                "Unsupported config version: {}. Expected 1.0",
                config.version
            )));
        }

        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn save<P: AsRef<Path>>(config: &Config, path: P) -> Result<()> {
        let content = serde_yaml::to_string(config)?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| S1bCr4ftError::config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }

    /// Create a new default configuration
    pub fn new_default(name: String) -> Config {
        Config {
            version: "1.0".to_string(),
            name,
            description: String::new(),
            modules: vec![
                "core/base-system".to_string(),
                "core/bootloader".to_string(),
            ],
            dotfiles: Vec::new(),
            hooks: Hooks::default(),
            options: ConfigOptions::default(),
            security: SecuritySettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ConfigLoader::new_default("test-project".to_string());
        assert_eq!(config.version, "1.0");
        assert_eq!(config.name, "test-project");
        assert!(!config.modules.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let config = ConfigLoader::new_default("test".to_string());
        let temp_file = NamedTempFile::new().unwrap();

        ConfigLoader::save(&config, temp_file.path()).unwrap();
        let loaded = ConfigLoader::load(temp_file.path()).unwrap();

        assert_eq!(config.name, loaded.name);
        assert_eq!(config.version, loaded.version);
    }
}
