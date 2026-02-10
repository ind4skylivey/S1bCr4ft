use crate::error::{Result, S1bCr4ftError};
use std::path::{Path, PathBuf};

/// GPG signature verifier
///
/// This struct provides methods to verify GPG signatures on configuration files.
#[cfg(feature = "gpg-signing")]
pub struct GpgVerifier {
    keyring: PathBuf,
    trusted_keys: Vec<String>,
}

#[cfg(feature = "gpg-signing")]
impl GpgVerifier {
    /// Create a new GPG verifier with default keyring
    ///
    /// Uses the default GPG home directory (~/.gnupg)
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| S1bCr4ftError::gpg("Could not determine home directory".to_string()))?
            .join(".gnupg");

        Self::with_keyring(home)
    }

    /// Create a GPG verifier with a custom keyring
    ///
    /// # Arguments
    ///
    /// * `keyring` - Path to the GPG keyring directory
    pub fn with_keyring(keyring: PathBuf) -> Result<Self> {
        Ok(Self {
            keyring,
            trusted_keys: Vec::new(),
        })
    }

    /// Add a trusted key ID
    ///
    /// # Arguments
    ///
    /// * `key_id` - GPG key ID or fingerprint to trust
    pub fn add_trusted_key(&mut self, key_id: String) {
        self.trusted_keys.push(key_id);
    }

    /// Load trusted keys from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to file containing trusted key IDs (one per line)
    pub fn load_trusted_keys_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| S1bCr4ftError::gpg(format!("Failed to read trusted keys file: {}", e)))?;

        self.trusted_keys = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect();

        Ok(())
    }

    /// Verify the GPG signature of a configuration file
    ///
    /// This method checks if the file has a valid GPG signature and if the signer
    /// is in the list of trusted keys.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to verify
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Signature is valid and signer is trusted
    /// * `Ok(false)` - File is not signed or signer is not trusted
    /// * `Err(_)` - Verification failed (invalid signature, error reading file, etc.)
    pub fn verify_config<P: AsRef<Path>>(&self, config_path: P) -> Result<bool> {
        let config_path = config_path.as_ref();

        // Check if the file exists
        if !config_path.exists() {
            return Err(S1bCr4ftError::gpg(format!(
                "Config file does not exist: {}",
                config_path.display()
            )));
        }

        // Check for detached signature file (.sig or .asc)
        let sig_path = config_path.with_extension("sig");
        let asc_path = config_path.with_extension("asc");

        let signature_path = if sig_path.exists() {
            Some(sig_path)
        } else if asc_path.exists() {
            Some(asc_path)
        } else {
            // No signature file found
            log::warn!(
                "No GPG signature found for config: {}",
                config_path.display()
            );
            return Ok(false);
        };

        log::info!(
            "Verifying GPG signature: {}",
            signature_path.as_ref().unwrap().display()
        );

        // Use gpg command-line to verify (simpler and more reliable)
        let output = std::process::Command::new("gpg")
            .args(["--verify", "--keyring", &self.keyring.display().to_string()])
            .arg(signature_path.as_ref().unwrap())
            .arg(config_path)
            .output()
            .map_err(|e| {
                S1bCr4ftError::gpg(format!("Failed to execute GPG verification: {}", e))
            })?;

        if !output.status.success() {
            log::debug!(
                "GPG verification stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Ok(false);
        }

        // Parse output to get signer key
        let stderr = String::from_utf8_lossy(&output.stderr);
        let signer_id = stderr
            .lines()
            .find(|line| line.contains("Good signature"))
            .and_then(|line| line.split("using").nth(1).map(|s| s.trim().to_string()));

        if let Some(key) = signer_id {
            log::info!("Valid signature from key: {}", key);

            // Check if signer is trusted
            if !self.trusted_keys.is_empty() && !self.trusted_keys.contains(&key) {
                log::warn!("Key {} is not in trusted keys", key);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Sign a configuration file
    ///
    /// This method signs the configuration file with the default GPG key.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to sign
    /// * `key_id` - GPG key ID to use for signing (optional, uses default if None)
    ///
    /// # Returns
    ///
    /// * `Ok(signature_path)` - Path to the created signature file
    pub fn sign_config<P: AsRef<Path>>(
        &self,
        config_path: P,
        key_id: Option<String>,
    ) -> Result<PathBuf> {
        let config_path = config_path.as_ref();

        if !config_path.exists() {
            return Err(S1bCr4ftError::gpg(format!(
                "Config file does not exist: {}",
                config_path.display()
            )));
        }

        log::info!(
            "Signing config: {} with key: {:?}",
            config_path.display(),
            key_id
        );

        let sig_path = config_path.with_extension("sig");

        // Build gpg command
        let mut cmd = std::process::Command::new("gpg");
        cmd.arg("--detach-sign");
        cmd.arg("--armor");
        cmd.arg("--output");
        cmd.arg(&sig_path);

        if let Some(key) = key_id {
            cmd.arg("--local-user");
            cmd.arg(&key);
        }

        cmd.arg(config_path);

        // Execute signing
        let output = cmd
            .output()
            .map_err(|e| S1bCr4ftError::gpg(format!("Failed to execute GPG signing: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(S1bCr4ftError::gpg(format!(
                "GPG signing failed: {}",
                stderr
            )));
        }

        log::info!("Signature created: {}", sig_path.display());

        Ok(sig_path)
    }

    /// Get the path to the GPG keyring
    pub fn keyring_path(&self) -> &Path {
        &self.keyring
    }

    /// List trusted keys
    pub fn trusted_keys(&self) -> &[String] {
        &self.trusted_keys
    }
}

/// Stub implementation when GPG feature is not enabled
#[cfg(not(feature = "gpg-signing"))]
pub struct GpgVerifier;

#[cfg(not(feature = "gpg-signing"))]
impl GpgVerifier {
    /// Create a new GPG verifier (stub)
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Verify config (stub - always returns true)
    pub fn verify_config<P: AsRef<Path>>(&self, _config_path: P) -> Result<bool> {
        log::warn!("GPG signing is not enabled, skipping verification");
        Ok(true)
    }

    /// Sign config (stub - no-op)
    pub fn sign_config<P: AsRef<Path>>(
        &self,
        _config_path: P,
        _key_id: Option<String>,
    ) -> Result<PathBuf> {
        Err(S1bCr4ftError::gpg(
            "GPG signing feature is not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verify_nonexistent_file() {
        let verifier = GpgVerifier::new().unwrap();
        let result = verifier.verify_config("/nonexistent/file.yml");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_unsigned_file() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "test config").unwrap();

        let verifier = GpgVerifier::new().unwrap();
        let result = verifier.verify_config(temp_file.path()).unwrap();
        // Should return false for unsigned files
        assert!(!result);
    }

    #[test]
    fn test_add_trusted_key() {
        let mut verifier = GpgVerifier::new().unwrap();
        verifier.add_trusted_key("ABCDEF1234567890".to_string());
        assert!(verifier
            .trusted_keys()
            .contains(&"ABCDEF1234567890".to_string()));
    }

    #[test]
    fn test_load_trusted_keys_from_file() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "KEY1\nKEY2\n# comment\nKEY3\n\n").unwrap();

        let mut verifier = GpgVerifier::new().unwrap();
        let result = verifier.load_trusted_keys_from_file(temp_file.path());
        assert!(result.is_ok());

        let keys = verifier.trusted_keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"KEY1".to_string()));
        assert!(keys.contains(&"KEY2".to_string()));
        assert!(keys.contains(&"KEY3".to_string()));
    }

    #[test]
    fn test_sign_config_without_feature() {
        let _verifier = GpgVerifier::new().unwrap();
        let _temp_file = NamedTempFile::new().unwrap();

        // Without GPG feature, signing should fail
        #[cfg(not(feature = "gpg-signing"))]
        {
            let result = _verifier.sign_config(_temp_file.path(), None);
            assert!(result.is_err());
        }
    }
}
