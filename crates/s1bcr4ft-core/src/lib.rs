//! # S1bCr4ft Core
//!
//! Core engine for S1bCr4ft - declarative system configuration for Arch Linux.
//!
//! ## Features
//! - YAML configuration parsing and validation
//! - Module dependency resolution
//! - Package management (pacman/paru/yay wrapper)
//! - Backup/rollback system
//! - Audit logging with GPG signing
//! - Lua scripting for hooks

pub mod config;
pub mod module;
pub mod package;
pub mod backup;
pub mod audit;
pub mod hooks;
pub mod validation;
pub mod error;

pub use config::{Config, ConfigLoader};
pub use module::{Module, ModuleResolver, ModuleRegistry};
pub use package::{PackageManager, SyncOptions, SyncReport};
pub use backup::{BackupManager, BackupId};
pub use audit::{AuditLogger, AuditEntry};
pub use error::{Result, S1bCr4ftError};

/// S1bCr4ft version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration directory
pub fn default_config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join("s1bcr4ft")
}

/// Default data directory
pub fn default_data_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("Failed to get data directory")
        .join("s1bcr4ft")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_dirs() {
        let config_dir = default_config_dir();
        let data_dir = default_data_dir();
        assert!(config_dir.to_string_lossy().contains("s1bcr4ft"));
        assert!(data_dir.to_string_lossy().contains("s1bcr4ft"));
    }
}
