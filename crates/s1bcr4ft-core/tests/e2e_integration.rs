//! End-to-end integration tests
//!
//! Tests cover:
//! - Complete workflow from config load to sync
//! - Backup and restore operations
//! - Audit logging
//! - Command validation in context

mod common;

use common::{ConfigFixture, ModuleFixture, PacmanMock};
use s1bcr4ft_core::audit::AuditLogger;
use s1bcr4ft_core::backup::BackupManager;
use s1bcr4ft_core::command_validator::CommandValidator;
use s1bcr4ft_core::config::ConfigLoader;
use s1bcr4ft_core::module::{ModuleRegistry, ModuleResolver};
use s1bcr4ft_core::package::{PackageHelper, PackageManager, SyncOptions};

/// Test complete workflow: load config -> resolve modules -> validate
#[test]
fn test_complete_workflow_happy_path() {
    // Setup
    let config_fixture = ConfigFixture::full();
    let module_fixture = ModuleFixture::with_sample_modules();

    // 1. Load configuration
    let config = ConfigLoader::load(config_fixture.path()).expect("Failed to load config");
    assert_eq!(config.name, "full-test-config");

    // 2. Load modules
    let mut registry = ModuleRegistry::new(module_fixture.path());
    registry.load_all().expect("Failed to load modules");

    // 3. Resolve dependencies
    let resolver = ModuleResolver::new(&registry);
    let resolved = resolver
        .resolve(&config.modules)
        .expect("Failed to resolve");

    // 4. Check for conflicts
    resolver
        .check_conflicts(&resolved)
        .expect("Conflicts found");

    // 5. Validate all modules exist
    for module_id in &resolved {
        assert!(
            registry.get(module_id).is_some(),
            "Module {} should exist",
            module_id
        );
    }

    // Success!
    assert!(!resolved.is_empty());
}

/// Test workflow with security modules
#[test]
fn test_workflow_with_security_modules() {
    let config_fixture = ConfigFixture::security_modules();
    let module_fixture = ModuleFixture::with_sample_modules();

    // Load config
    let config = ConfigLoader::load(config_fixture.path()).expect("Failed to load config");

    // Load modules
    let mut registry = ModuleRegistry::new(module_fixture.path());
    registry.load_all().expect("Failed to load modules");

    // Resolve
    let resolver = ModuleResolver::new(&registry);
    let result = resolver.resolve(&config.modules);

    // Should succeed as security modules are valid
    assert!(result.is_ok());
}

/// Test command validation in workflow
#[test]
fn test_command_validation_in_workflow() {
    let validator = CommandValidator::new();

    // Valid commands from modules (using whitelist)
    let valid_commands = [
        "systemctl enable docker",
        "sysctl -p",
        "useradd -m user",
        "locale-gen",
    ];

    for cmd in &valid_commands {
        let parsed = validator
            .parse(cmd)
            .unwrap_or_else(|_| panic!("Failed to parse: {}", cmd));
        let result = validator.validate(&parsed);
        assert!(result.is_ok(), "Command should be valid: {}", cmd);
    }

    // Invalid commands (injection attempts)
    let invalid_commands = [
        "systemctl enable docker; rm -rf /",
        "sysctl $(whoami)",
        "useradd | cat /etc/passwd",
        "locale-gen `malicious`",
    ];

    for cmd in &invalid_commands {
        let parsed = validator
            .parse(cmd)
            .unwrap_or_else(|_| panic!("Failed to parse: {}", cmd));
        let result = validator.validate(&parsed);
        assert!(result.is_err(), "Command should be invalid: {}", cmd);
    }
}

/// Test backup manager integration
#[test]
fn test_backup_manager_integration() {
    let mock = PacmanMock::new();

    // Create backup manager with custom directory
    let backup_manager = BackupManager::with_dir(mock.temp_path().join("backups"))
        .expect("Failed to create backup manager");

    // List backups (should be empty initially)
    let backups = backup_manager
        .list_backups()
        .expect("Failed to list backups");
    assert!(backups.is_empty());

    // Create a test config file for backup
    let config_path = mock.temp_path().join("test_config.yml");
    std::fs::write(&config_path, "version: \"1.0\"\nname: test\nmodules: []")
        .expect("Failed to write test config");

    // Create backup
    let backup_id = backup_manager
        .create_backup(&config_path, Some("test-backup".to_string()))
        .expect("Failed to create backup");
    assert!(!backup_id.is_empty());

    // List again (should have one)
    let backups = backup_manager
        .list_backups()
        .expect("Failed to list backups");
    assert_eq!(backups.len(), 1);
}

/// Test audit logger integration
#[test]
fn test_audit_logger_integration() {
    let mock = PacmanMock::new();

    // Create audit logger with custom file
    let audit_logger = AuditLogger::with_file(mock.temp_path().join("audit.log"))
        .expect("Failed to create audit logger");

    // Log some actions using the log method
    audit_logger
        .log(
            s1bcr4ft_core::audit::AuditAction::Sync,
            serde_json::json!({"packages": ["vim", "git"]}),
            true,
        )
        .expect("Failed to log sync");

    audit_logger
        .log(
            s1bcr4ft_core::audit::AuditAction::ModuleAdd,
            serde_json::json!({"module": "development/languages/rust"}),
            true,
        )
        .expect("Failed to log module add");

    // Get entries
    let entries = audit_logger
        .get_entries(None)
        .expect("Failed to get entries");
    assert!(!entries.is_empty());
}

/// Test dry run mode
#[test]
fn test_dry_run_mode() {
    let mock = PacmanMock::new();

    let options = SyncOptions {
        dry_run: true,
        force: false,
        parallel: false,
    };

    let manager = PackageManager::with_helper(PackageHelper::Pacman);

    // In dry run mode, nothing should actually be installed
    let result = manager.install_packages(&["vim".to_string(), "git".to_string()], &options);

    assert!(result.is_ok());
    let installed = result.unwrap();
    assert_eq!(installed.len(), 2);

    // But mock shouldn't have them
    assert!(!mock.is_installed("vim"));
    assert!(!mock.is_installed("git"));
}

/// Test package helper detection
#[test]
fn test_package_helper_detection() {
    // This test verifies the detection doesn't crash
    let helper = PackageHelper::detect();

    // We should get some helper
    let cmd = helper.command();
    assert!(!cmd.is_empty());

    // Check AUR capability
    match helper {
        PackageHelper::Pacman => assert!(!helper.can_install_aur()),
        PackageHelper::Paru | PackageHelper::Yay => assert!(helper.can_install_aur()),
    }
}

/// Test error handling: missing module
#[test]
fn test_error_handling_missing_module() {
    let config_fixture = ConfigFixture::minimal();
    let module_fixture = ModuleFixture::empty(); // Empty modules

    // Load config
    let config = ConfigLoader::load(config_fixture.path()).expect("Failed to load config");

    // Load modules (empty)
    let mut registry = ModuleRegistry::new(module_fixture.path());
    registry.load_all().expect("Failed to load modules");

    // Try to resolve (should fail because module doesn't exist)
    let resolver = ModuleResolver::new(&registry);
    let result = resolver.resolve(&config.modules);

    assert!(result.is_err());
}

/// Test error handling: invalid config version
#[test]
fn test_error_handling_invalid_config_version() {
    let fixture = ConfigFixture::invalid_version();

    let result = ConfigLoader::load(fixture.path());

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Unsupported config version"));
}

/// Test module with complex dependency chain
#[test]
fn test_complex_dependency_chain() {
    let module_fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(module_fixture.path());
    registry.load_all().expect("Failed to load modules");

    let resolver = ModuleResolver::new(&registry);

    // sliver-c2 depends on go which depends on base-system
    let resolved = resolver
        .resolve(&["red-team/c2-frameworks/sliver-c2".to_string()])
        .expect("Failed to resolve");

    // Verify order
    assert_eq!(resolved[0], "core/base-system");
    assert_eq!(resolved[1], "development/languages/go");
    assert_eq!(resolved[2], "red-team/c2-frameworks/sliver-c2");
}

/// Test sync report generation
#[test]
fn test_sync_report_generation() {
    let manager = PackageManager::with_helper(PackageHelper::Pacman);

    let options = SyncOptions {
        dry_run: true,
        force: false,
        parallel: true,
    };

    let report = manager
        .sync(
            &["vim".to_string(), "git".to_string()],
            &[],
            &["echo test".to_string()],
            &options,
        )
        .expect("Sync failed");

    assert!(!report.packages_installed.is_empty());
    assert!(report.packages_failed.is_empty());
    assert!(!report.commands_executed.is_empty());
}

/// Test multiple configs can coexist
#[test]
fn test_multiple_configs() {
    let fixture1 = ConfigFixture::minimal();
    let fixture2 = ConfigFixture::full();

    let config1 = ConfigLoader::load(fixture1.path()).expect("Failed to load config1");
    let config2 = ConfigLoader::load(fixture2.path()).expect("Failed to load config2");

    assert_ne!(config1.name, config2.name);
    assert_ne!(config1.modules.len(), config2.modules.len());
}

/// Test config modification and reload
#[test]
fn test_config_modification_reload() {
    let fixture = ConfigFixture::minimal();

    // Load original
    let mut config = ConfigLoader::load(fixture.path()).expect("Failed to load config");
    let original_module_count = config.modules.len();

    // Modify
    config.modules.push("new/module".to_string());
    ConfigLoader::save(&config, fixture.path()).expect("Failed to save");

    // Reload
    let reloaded = ConfigLoader::load(fixture.path()).expect("Failed to reload");

    assert_eq!(reloaded.modules.len(), original_module_count + 1);
}
