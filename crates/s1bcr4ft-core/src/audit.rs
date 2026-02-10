use crate::error::{Result, S1bCr4ftError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub user: String,
    pub details: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum AuditAction {
    Sync,
    Rollback,
    ConfigChange,
    PackageInstall,
    PackageRemove,
    BackupCreate,
    BackupRestore,
    ModuleAdd,
    ModuleRemove,
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::Sync => "sync",
            AuditAction::Rollback => "rollback",
            AuditAction::ConfigChange => "config_change",
            AuditAction::PackageInstall => "package_install",
            AuditAction::PackageRemove => "package_remove",
            AuditAction::BackupCreate => "backup_create",
            AuditAction::BackupRestore => "backup_restore",
            AuditAction::ModuleAdd => "module_add",
            AuditAction::ModuleRemove => "module_remove",
        }
    }
}

pub struct AuditLogger {
    log_file: PathBuf,
}

impl AuditLogger {
    pub fn new() -> Result<Self> {
        let log_dir = dirs::data_dir()
            .ok_or_else(|| S1bCr4ftError::audit("Could not determine data directory"))?
            .join("s1bcr4ft")
            .join("audit");

        fs::create_dir_all(&log_dir).map_err(|e| {
            S1bCr4ftError::audit(format!("Failed to create audit directory: {}", e))
        })?;

        let log_file = log_dir.join("audit.log");

        Ok(Self { log_file })
    }

    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let log_file = path.as_ref().to_path_buf();

        if let Some(parent) = log_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                S1bCr4ftError::audit(format!("Failed to create audit directory: {}", e))
            })?;
        }

        Ok(Self { log_file })
    }

    /// Log an audit entry
    pub fn log(
        &self,
        action: AuditAction,
        details: serde_json::Value,
        success: bool,
    ) -> Result<()> {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: action.as_str().to_string(),
            user,
            details,
            success,
        };

        self.write_entry(&entry)
    }

    /// Log a custom action
    pub fn log_custom(
        &self,
        action: &str,
        details: serde_json::Value,
        success: bool,
    ) -> Result<()> {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: action.to_string(),
            user,
            details,
            success,
        };

        self.write_entry(&entry)
    }

    fn write_entry(&self, entry: &AuditEntry) -> Result<()> {
        let json = serde_json::to_string(entry)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to serialize audit entry: {}", e)))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to open audit log: {}", e)))?;

        writeln!(file, "{}", json)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to write audit entry: {}", e)))?;

        Ok(())
    }

    /// Get all audit entries
    pub fn get_entries(&self, since: Option<DateTime<Utc>>) -> Result<Vec<AuditEntry>> {
        if !self.log_file.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.log_file)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to open audit log: {}", e)))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line =
                line.map_err(|e| S1bCr4ftError::audit(format!("Failed to read line: {}", e)))?;

            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditEntry = serde_json::from_str(&line)
                .map_err(|e| S1bCr4ftError::audit(format!("Failed to parse audit entry: {}", e)))?;

            // Filter by timestamp if provided
            if let Some(since_time) = since {
                if entry.timestamp >= since_time {
                    entries.push(entry);
                }
            } else {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get entries for a specific action
    pub fn get_entries_by_action(&self, action: AuditAction) -> Result<Vec<AuditEntry>> {
        let all_entries = self.get_entries(None)?;
        Ok(all_entries
            .into_iter()
            .filter(|e| e.action == action.as_str())
            .collect())
    }

    /// Get entries for a specific user
    pub fn get_entries_by_user(&self, user: &str) -> Result<Vec<AuditEntry>> {
        let all_entries = self.get_entries(None)?;
        Ok(all_entries.into_iter().filter(|e| e.user == user).collect())
    }

    /// Get failed entries
    pub fn get_failed_entries(&self) -> Result<Vec<AuditEntry>> {
        let all_entries = self.get_entries(None)?;
        Ok(all_entries.into_iter().filter(|e| !e.success).collect())
    }

    /// Get entry count
    pub fn count_entries(&self) -> Result<usize> {
        Ok(self.get_entries(None)?.len())
    }

    /// Rotate log file (archive old entries)
    pub fn rotate_log(&self, max_entries: usize) -> Result<usize> {
        let entries = self.get_entries(None)?;

        if entries.len() <= max_entries {
            return Ok(0);
        }

        // Keep only the most recent entries
        let to_keep = &entries[entries.len() - max_entries..];

        // Archive old entries
        let archive_file = self.log_file.with_extension("log.old");
        if archive_file.exists() {
            fs::remove_file(&archive_file).map_err(|e| {
                S1bCr4ftError::audit(format!("Failed to remove old archive: {}", e))
            })?;
        }

        fs::rename(&self.log_file, &archive_file)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to archive log: {}", e)))?;

        // Write kept entries to new log
        for entry in to_keep {
            self.write_entry(entry)?;
        }

        let archived_count = entries.len() - max_entries;
        log::info!("Archived {} audit entries", archived_count);

        Ok(archived_count)
    }

    /// Export audit log to JSON
    pub fn export_to_json(&self, output_path: &Path) -> Result<()> {
        let entries = self.get_entries(None)?;

        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to serialize entries: {}", e)))?;

        fs::write(output_path, json)
            .map_err(|e| S1bCr4ftError::audit(format!("Failed to write export: {}", e)))?;

        log::info!(
            "Exported {} audit entries to {:?}",
            entries.len(),
            output_path
        );
        Ok(())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new().expect("Failed to create AuditLogger")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_audit_logging() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::with_file(temp_file.path()).unwrap();

        logger
            .log(AuditAction::Sync, json!({"packages": ["test"]}), true)
            .unwrap();

        let entries = logger.get_entries(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "sync");
    }

    #[test]
    fn test_filter_by_action() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::with_file(temp_file.path()).unwrap();

        logger.log(AuditAction::Sync, json!({}), true).unwrap();
        logger
            .log(AuditAction::BackupCreate, json!({}), true)
            .unwrap();

        let sync_entries = logger.get_entries_by_action(AuditAction::Sync).unwrap();
        assert_eq!(sync_entries.len(), 1);
    }
}
