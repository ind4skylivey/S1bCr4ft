use crate::error::{Result, S1bCr4ftError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub type BackupId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: BackupId,
    pub timestamp: DateTime<Utc>,
    pub config_snapshot: String,
    pub packages: Vec<String>,
    pub description: String,
}

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let backup_dir = dirs::data_dir()
            .ok_or_else(|| S1bCr4ftError::backup("Could not determine data directory"))?
            .join("s1bcr4ft")
            .join("backups");

        fs::create_dir_all(&backup_dir).map_err(|e| {
            S1bCr4ftError::backup(format!("Failed to create backup directory: {}", e))
        })?;

        Ok(Self { backup_dir })
    }

    pub fn with_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let backup_dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&backup_dir).map_err(|e| {
            S1bCr4ftError::backup(format!("Failed to create backup directory: {}", e))
        })?;

        Ok(Self { backup_dir })
    }

    /// Create a new backup snapshot
    pub fn create_backup(
        &self,
        config_path: &Path,
        description: Option<String>,
    ) -> Result<BackupId> {
        let backup_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Read current config
        let config_snapshot = fs::read_to_string(config_path)
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to read config: {}", e)))?;

        // Get list of installed packages
        let packages = self.get_installed_packages()?;

        let backup = Backup {
            id: backup_id.clone(),
            timestamp,
            config_snapshot: config_snapshot.clone(),
            packages,
            description: description.unwrap_or_else(|| "Manual backup".to_string()),
        };

        // Save backup metadata
        let backup_file = self.backup_dir.join(format!("{}.json", backup_id));
        let backup_json = serde_json::to_string_pretty(&backup)
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to serialize backup: {}", e)))?;

        fs::write(&backup_file, backup_json)
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to write backup: {}", e)))?;

        // Save config snapshot separately
        let config_file = self.backup_dir.join(format!("{}.config.yml", backup_id));
        fs::write(&config_file, config_snapshot).map_err(|e| {
            S1bCr4ftError::backup(format!("Failed to write config snapshot: {}", e))
        })?;

        log::info!("Created backup: {}", backup_id);
        Ok(backup_id)
    }

    /// Restore from a backup
    pub fn restore(&self, backup_id: &str, config_path: &Path) -> Result<()> {
        let backup = self.get_backup(backup_id)?;

        log::info!("Restoring backup: {}", backup_id);

        // Restore config file
        fs::write(config_path, &backup.config_snapshot)
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to restore config: {}", e)))?;

        log::info!("Config restored from backup {}", backup_id);
        log::warn!("Note: Package restoration requires manual sync with 's1bcr4ft sync'");

        Ok(())
    }

    /// List all backups
    pub fn list_backups(&self) -> Result<Vec<Backup>> {
        let mut backups = Vec::new();

        let entries = fs::read_dir(&self.backup_dir).map_err(|e| {
            S1bCr4ftError::backup(format!("Failed to read backup directory: {}", e))
        })?;

        for entry in entries {
            let entry =
                entry.map_err(|e| S1bCr4ftError::backup(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).map_err(|e| {
                    S1bCr4ftError::backup(format!("Failed to read backup file: {}", e))
                })?;

                let backup: Backup = serde_json::from_str(&content)
                    .map_err(|e| S1bCr4ftError::backup(format!("Failed to parse backup: {}", e)))?;

                backups.push(backup);
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// Get a specific backup
    pub fn get_backup(&self, backup_id: &str) -> Result<Backup> {
        let backup_file = self.backup_dir.join(format!("{}.json", backup_id));

        if !backup_file.exists() {
            return Err(S1bCr4ftError::backup(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        let content = fs::read_to_string(&backup_file)
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to read backup: {}", e)))?;

        let backup: Backup = serde_json::from_str(&content)
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to parse backup: {}", e)))?;

        Ok(backup)
    }

    /// Delete a backup
    pub fn delete_backup(&self, backup_id: &str) -> Result<()> {
        let backup_file = self.backup_dir.join(format!("{}.json", backup_id));
        let config_file = self.backup_dir.join(format!("{}.config.yml", backup_id));

        if backup_file.exists() {
            fs::remove_file(&backup_file)
                .map_err(|e| S1bCr4ftError::backup(format!("Failed to delete backup: {}", e)))?;
        }

        if config_file.exists() {
            fs::remove_file(&config_file).map_err(|e| {
                S1bCr4ftError::backup(format!("Failed to delete config snapshot: {}", e))
            })?;
        }

        log::info!("Deleted backup: {}", backup_id);
        Ok(())
    }

    /// Get list of currently installed packages
    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("pacman")
            .args(["-Q"])
            .output()
            .map_err(|e| S1bCr4ftError::backup(format!("Failed to get package list: {}", e)))?;

        if !output.status.success() {
            return Err(S1bCr4ftError::backup("Failed to query installed packages"));
        }

        let packages_str = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = packages_str
            .lines()
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect();

        Ok(packages)
    }

    /// Clean old backups (keep only N most recent)
    pub fn clean_old_backups(&self, keep_count: usize) -> Result<usize> {
        let mut backups = self.list_backups()?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        // Already sorted by timestamp (newest first)
        let to_delete = backups.split_off(keep_count);
        let deleted_count = to_delete.len();

        for backup in to_delete {
            self.delete_backup(&backup.id)?;
        }

        log::info!("Cleaned {} old backups", deleted_count);
        Ok(deleted_count)
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new().expect("Failed to create BackupManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_backup_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = BackupManager::with_dir(temp_dir.path()).unwrap();

        let config_path = temp_dir.path().join("test_config.yml");
        fs::write(&config_path, "test: config").unwrap();

        let backup_id = manager
            .create_backup(&config_path, Some("Test backup".to_string()))
            .unwrap();
        assert!(!backup_id.is_empty());
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = tempdir().unwrap();
        let manager = BackupManager::with_dir(temp_dir.path()).unwrap();

        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 0);
    }
}
