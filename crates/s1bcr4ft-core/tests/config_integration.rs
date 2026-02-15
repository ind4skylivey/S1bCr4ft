//! Integration tests for configuration loading
//!
//! Tests cover:
//! - Loading valid configurations
//! - Handling invalid configurations
//! - Saving configurations
//! - Configuration validation

mod common;

use common::{assert, ConfigFixture};
use s1bcr4ft_core::config::{ConfigLoader, ConfigOptions, Hooks, SecuritySettings};

/// Test loading a minimal valid configuration
#[test]
fn test_load_minimal_config() {
    let fixture = ConfigFixture::minimal();

    let config = ConfigLoader::load(fixture.path()).expect("Failed to load config");

    assert_eq!(config.version, "1.0");
    assert_eq!(config.name, "test-config");
    assert_eq!(config.modules, vec!["core/base-system"]);
}

/// Test loading a full configuration with all options
#[test]
fn test_load_full_config() {
    let fixture = ConfigFixture::full();

    let config = ConfigLoader::load(fixture.path()).expect("Failed to load config");

    assert_eq!(config.version, "1.0");
    assert_eq!(config.name, "full-test-config");
    assert!(config.description.contains("comprehensive"));

    // Check modules (using modules that exist in fixture)
    assert!(config.modules.contains(&"core/base-system".to_string()));
    assert!(config
        .modules
        .contains(&"development/languages/rust".to_string()));
    assert!(config
        .modules
        .contains(&"development/languages/go".to_string()));

    // Check hooks
    assert!(config.hooks.pre_sync.is_some());
    assert!(config.hooks.post_sync.is_some());

    // Check options
    assert!(config.options.auto_backup);
    assert!(!config.options.dry_run);
    assert!(config.options.parallel_install);

    // Check security
    assert!(config.security.gpg_signing);
    assert!(!config.security.network_isolation);
}

/// Test that invalid version is rejected
#[test]
fn test_load_invalid_version() {
    let fixture = ConfigFixture::invalid_version();

    let result = ConfigLoader::load(fixture.path());

    assert!(result.is_err());
    assert::error_contains(&result, "Unsupported config version");
}

/// Test that invalid YAML is handled
#[test]
fn test_load_invalid_yaml() {
    let fixture = ConfigFixture::invalid_yaml();

    let result = ConfigLoader::load(fixture.path());

    assert!(result.is_err());
}

/// Test saving a configuration
#[test]
fn test_save_config() {
    let fixture = ConfigFixture::minimal();
    let mut config = ConfigLoader::load(fixture.path()).unwrap();

    // Modify the config
    config.name = "modified-name".to_string();
    config.modules.push("new/module".to_string());

    // Save it
    ConfigLoader::save(&config, fixture.path()).expect("Failed to save config");

    // Reload and verify
    let reloaded = ConfigLoader::load(fixture.path()).expect("Failed to reload config");

    assert_eq!(reloaded.name, "modified-name");
    assert!(reloaded.modules.contains(&"new/module".to_string()));
}

/// Test creating a default configuration
#[test]
fn test_create_default_config() {
    let config = ConfigLoader::new_default("my-project".to_string());

    assert_eq!(config.version, "1.0");
    assert_eq!(config.name, "my-project");
    assert!(!config.modules.is_empty());
}

/// Test configuration options defaults
#[test]
fn test_config_options_defaults() {
    let options = ConfigOptions::default();

    assert!(options.auto_backup);
    assert!(!options.dry_run);
    assert!(options.parallel_install);
    assert!(options.custom.is_empty());
}

/// Test hooks defaults
#[test]
fn test_hooks_defaults() {
    let hooks = Hooks::default();

    assert!(hooks.pre_sync.is_none());
    assert!(hooks.post_sync.is_none());
    assert!(hooks.pre_module.is_none());
    assert!(hooks.post_module.is_none());
}

/// Test security settings defaults
#[test]
fn test_security_settings_defaults() {
    let security = SecuritySettings::default();

    assert!(security.isolation_level.is_none());
    assert!(!security.network_isolation);
    assert!(security.container_sandbox.is_none());
    assert!(!security.gpg_signing);
}

/// Test loading config with security modules
#[test]
fn test_load_security_config() {
    let fixture = ConfigFixture::security_modules();

    let config = ConfigLoader::load(fixture.path()).expect("Failed to load config");

    // Check security modules
    assert!(config
        .modules
        .contains(&"red-team/c2-frameworks/sliver-c2".to_string()));
    assert!(config.security.gpg_signing);
    assert!(config.security.network_isolation);
}

/// Test round-trip save/load preserves data
#[test]
fn test_round_trip_preserves_data() {
    let fixture = ConfigFixture::full();
    let original = ConfigLoader::load(fixture.path()).unwrap();

    // Save to a new location
    let new_path = fixture.temp_dir.path().join("roundtrip.yml");
    ConfigLoader::save(&original, &new_path).unwrap();

    // Reload
    let reloaded = ConfigLoader::load(&new_path).unwrap();

    // Compare
    assert_eq!(original.version, reloaded.version);
    assert_eq!(original.name, reloaded.name);
    assert_eq!(original.description, reloaded.description);
    assert_eq!(original.modules, reloaded.modules);
}

/// Test config with empty modules list
#[test]
fn test_load_config_empty_modules() {
    let fixture = ConfigFixture::minimal();
    let content = r#"
version: "1.0"
name: empty-modules
modules: []
"#;
    std::fs::write(fixture.path(), content).unwrap();

    let config = ConfigLoader::load(fixture.path()).expect("Failed to load config");

    assert_eq!(config.name, "empty-modules");
    assert!(config.modules.is_empty());
}

/// Test config with custom options
#[test]
fn test_load_config_custom_options() {
    let fixture = ConfigFixture::minimal();
    let content = r#"
version: "1.0"
name: custom-options
modules:
  - core/base-system
options:
  auto_backup: false
  dry_run: true
  parallel_install: false
  custom:
    my_key: "my_value"
    my_number: 42
"#;
    std::fs::write(fixture.path(), content).unwrap();

    let config = ConfigLoader::load(fixture.path()).expect("Failed to load config");

    assert!(!config.options.auto_backup);
    assert!(config.options.dry_run);
    assert!(!config.options.parallel_install);
    assert!(config.options.custom.contains_key("my_key"));
}

/// Test missing required fields
#[test]
fn test_load_config_missing_required_fields() {
    let fixture = ConfigFixture::minimal();
    let content = r#"
version: "1.0"
"#;
    std::fs::write(fixture.path(), content).unwrap();

    let result = ConfigLoader::load(fixture.path());

    // Should fail because 'name' and 'modules' are required
    assert!(result.is_err());
}
