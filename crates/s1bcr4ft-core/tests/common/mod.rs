//! Test utilities and fixtures for integration tests
//!
//! This module provides:
//! - Mock implementations for package managers
//! - Temporary file system helpers
//! - Test fixtures for configs and modules
//! - Assertion helpers

use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

/// Test harness for simulating pacman operations without root access
pub struct PacmanMock {
    /// Temporary directory for mock operations
    pub temp_dir: TempDir,
    /// Simulated installed packages
    installed_packages: Mutex<Vec<String>>,
    /// Simulated available packages
    available_packages: Vec<String>,
    /// Whether operations should fail
    should_fail: Mutex<bool>,
}

impl PacmanMock {
    /// Create a new pacman mock
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
            installed_packages: Mutex::new(Vec::new()),
            available_packages: vec![
                "base".to_string(),
                "linux".to_string(),
                "linux-firmware".to_string(),
                "sudo".to_string(),
                "vim".to_string(),
                "git".to_string(),
                "rust".to_string(),
                "python".to_string(),
                "go".to_string(),
                "neovim".to_string(),
                "hyprland".to_string(),
                "wayland".to_string(),
            ],
            should_fail: Mutex::new(false),
        }
    }

    /// Create a mock with specific available packages
    pub fn with_packages(packages: Vec<&str>) -> Self {
        let mut mock = Self::new();
        mock.available_packages = packages.iter().map(|s| s.to_string()).collect();
        mock
    }

    /// Simulate installing a package
    pub fn install(&self, package: &str) -> Result<(), String> {
        if *self.should_fail.lock().unwrap() {
            return Err("Mock failure triggered".to_string());
        }

        if !self.available_packages.contains(&package.to_string()) {
            return Err(format!("Package not found: {}", package));
        }

        let mut installed = self.installed_packages.lock().unwrap();
        if !installed.contains(&package.to_string()) {
            installed.push(package.to_string());
        }

        Ok(())
    }

    /// Simulate installing multiple packages
    pub fn install_packages(&self, packages: &[&str]) -> Result<Vec<String>, String> {
        let mut successful = Vec::new();
        for pkg in packages {
            match self.install(pkg) {
                Ok(()) => successful.push(pkg.to_string()),
                Err(e) => return Err(e),
            }
        }
        Ok(successful)
    }

    /// Check if a package is installed
    pub fn is_installed(&self, package: &str) -> bool {
        self.installed_packages
            .lock()
            .unwrap()
            .contains(&package.to_string())
    }

    /// List all installed packages
    pub fn list_installed(&self) -> Vec<String> {
        self.installed_packages.lock().unwrap().clone()
    }

    /// Set whether operations should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    /// Get the temp directory path
    pub fn temp_path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }
}

impl Default for PacmanMock {
    fn default() -> Self {
        Self::new()
    }
}

/// Test fixture for creating temporary config files
pub struct ConfigFixture {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl ConfigFixture {
    /// Create a minimal valid config
    pub fn minimal() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("s1bcr4ft.yml");

        let content = r#"
version: "1.0"
name: test-config
modules:
  - core/base-system
"#;

        std::fs::write(&config_path, content).expect("Failed to write config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Create a full config with all options
    pub fn full() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("s1bcr4ft.yml");

        // Use modules that exist in the test fixture
        let content = r#"
version: "1.0"
name: full-test-config
description: A comprehensive test configuration
modules:
  - core/base-system
  - development/languages/rust
  - development/languages/go

hooks:
  pre_sync: "echo 'Pre-sync hook'"
  post_sync: "echo 'Post-sync hook'"

options:
  auto_backup: true
  dry_run: false
  parallel_install: true
  custom:
    test_option: "test_value"

security:
  gpg_signing: true
  network_isolation: false
"#;

        std::fs::write(&config_path, content).expect("Failed to write config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Create an invalid config (wrong version)
    pub fn invalid_version() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("s1bcr4ft.yml");

        let content = r#"
version: "2.0"
name: invalid-config
modules:
  - core/base-system
"#;

        std::fs::write(&config_path, content).expect("Failed to write config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Create a config with invalid YAML
    pub fn invalid_yaml() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("s1bcr4ft.yml");

        // This is truly invalid YAML - unclosed bracket
        let content = r#"
version: "1.0"
name: broken
modules: [
  - unclosed bracket here
"#;

        std::fs::write(&config_path, content).expect("Failed to write config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Create a config with security-sensitive modules
    pub fn security_modules() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("s1bcr4ft.yml");

        // Use modules that exist in the test fixture
        let content = r#"
version: "1.0"
name: security-config
modules:
  - red-team/c2-frameworks/sliver-c2

security:
  gpg_signing: true
  network_isolation: true
"#;

        std::fs::write(&config_path, content).expect("Failed to write config");

        Self {
            temp_dir,
            config_path,
        }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.config_path
    }
}

/// Test fixture for creating temporary module directories
pub struct ModuleFixture {
    pub temp_dir: TempDir,
    pub modules_path: PathBuf,
}

impl ModuleFixture {
    /// Create a module fixture with sample modules
    pub fn with_sample_modules() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let modules_path = temp_dir.path().join("modules");

        // Create module directory structure
        Self::create_module(
            &modules_path,
            "core/base-system",
            r#"
id: core/base-system
name: Base System
description: Essential Arch Linux packages
category: core
version: "1.0.0"
packages:
  - base
  - linux
  - linux-firmware
  - sudo
"#,
        );

        Self::create_module(
            &modules_path,
            "development/languages/rust",
            r#"
id: development/languages/rust
name: Rust
description: Rust programming language
category: development
version: "1.0.0"
dependencies:
  - core/base-system
packages:
  - rust
  - cargo
"#,
        );

        Self::create_module(
            &modules_path,
            "development/languages/go",
            r#"
id: development/languages/go
name: Go
description: Go programming language
category: development
version: "1.0.0"
dependencies:
  - core/base-system
packages:
  - go
"#,
        );

        Self::create_module(
            &modules_path,
            "red-team/c2-frameworks/sliver-c2",
            r#"
id: red-team/c2-frameworks/sliver-c2
name: Sliver C2
description: Cross-platform adversary emulation
category: red-team
version: "1.0.0"
dependencies:
  - development/languages/go
packages: []
aur_packages:
  - sliver
"#,
        );

        Self::create_module(
            &modules_path,
            "conflict/module-a",
            r#"
id: conflict/module-a
name: Module A
description: First conflicting module
category: conflict
version: "1.0.0"
conflicts:
  - conflict/module-b
packages:
  - package-a
"#,
        );

        Self::create_module(
            &modules_path,
            "conflict/module-b",
            r#"
id: conflict/module-b
name: Module B
description: Second conflicting module
category: conflict
version: "1.0.0"
conflicts:
  - conflict/module-a
packages:
  - package-b
"#,
        );

        Self {
            temp_dir,
            modules_path,
        }
    }

    /// Create a module with circular dependencies
    pub fn with_circular_deps() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let modules_path = temp_dir.path().join("modules");

        Self::create_module(
            &modules_path,
            "circular/a",
            r#"
id: circular/a
name: Circular A
description: First circular dep
category: circular
version: "1.0.0"
dependencies:
  - circular/b
packages:
  - package-a
"#,
        );

        Self::create_module(
            &modules_path,
            "circular/b",
            r#"
id: circular/b
name: Circular B
description: Second circular dep
category: circular
version: "1.0.0"
dependencies:
  - circular/a
packages:
  - package-b
"#,
        );

        Self {
            temp_dir,
            modules_path,
        }
    }

    /// Create an empty module fixture
    pub fn empty() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let modules_path = temp_dir.path().join("modules");
        std::fs::create_dir_all(&modules_path).expect("Failed to create modules dir");

        Self {
            temp_dir,
            modules_path,
        }
    }

    fn create_module(base: &std::path::Path, id: &str, content: &str) {
        let module_path = base.join(id);
        std::fs::create_dir_all(&module_path).expect("Failed to create module dir");
        std::fs::write(module_path.join("module.yml"), content).expect("Failed to write module");
    }

    pub fn path(&self) -> &std::path::Path {
        &self.modules_path
    }
}

/// Assertion helpers
pub mod assert {
    use std::path::Path;

    /// Assert that a file exists
    pub fn file_exists(path: &Path) {
        assert!(path.exists(), "Expected file to exist: {}", path.display());
    }

    /// Assert that a file does not exist
    pub fn file_not_exists(path: &Path) {
        assert!(
            !path.exists(),
            "Expected file to NOT exist: {}",
            path.display()
        );
    }

    /// Assert that a file contains specific content
    pub fn file_contains(path: &Path, expected: &str) {
        let content = std::fs::read_to_string(path).expect("Failed to read file");
        assert!(
            content.contains(expected),
            "Expected file to contain '{}'\nActual content:\n{}",
            expected,
            content
        );
    }

    /// Assert that a result is an error containing specific text
    pub fn error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
        result: &Result<T, E>,
        expected: &str,
    ) {
        assert!(result.is_err(), "Expected error but got success");
        let error = result.as_ref().unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "Expected error to contain '{}', got: {}",
            expected,
            error
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_mock_install() {
        let mock = PacmanMock::new();

        // Install a package
        mock.install("vim").unwrap();
        assert!(mock.is_installed("vim"));
        assert!(!mock.is_installed("nonexistent"));
    }

    #[test]
    fn test_pacman_mock_not_found() {
        let mock = PacmanMock::new();

        // Try to install non-existent package
        let result = mock.install("nonexistent-package");
        assert!(result.is_err());
    }

    #[test]
    fn test_pacman_mock_failure() {
        let mock = PacmanMock::new();

        mock.set_should_fail(true);
        let result = mock.install("vim");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_fixture_minimal() {
        let fixture = ConfigFixture::minimal();
        assert::file_exists(&fixture.config_path);
        assert::file_contains(&fixture.config_path, "version: \"1.0\"");
    }

    #[test]
    fn test_config_fixture_full() {
        let fixture = ConfigFixture::full();
        assert::file_exists(&fixture.config_path);
        assert::file_contains(&fixture.config_path, "gpg_signing: true");
    }

    #[test]
    fn test_module_fixture() {
        let fixture = ModuleFixture::with_sample_modules();
        assert!(fixture
            .modules_path
            .join("core/base-system/module.yml")
            .exists());
        assert!(fixture
            .modules_path
            .join("development/languages/rust/module.yml")
            .exists());
    }
}
