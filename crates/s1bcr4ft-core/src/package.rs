use crate::error::{Result, S1bCr4ftError};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub force: bool,
    pub parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub packages_installed: Vec<String>,
    pub packages_failed: Vec<String>,
    pub commands_executed: Vec<String>,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum PackageHelper {
    Pacman,
    Paru,
    Yay,
}

impl PackageHelper {
    /// Detect which package helper is available
    pub fn detect() -> Self {
        if Command::new("paru").arg("--version").output().is_ok() {
            PackageHelper::Paru
        } else if Command::new("yay").arg("--version").output().is_ok() {
            PackageHelper::Yay
        } else {
            PackageHelper::Pacman
        }
    }

    pub fn command(&self) -> &str {
        match self {
            PackageHelper::Pacman => "pacman",
            PackageHelper::Paru => "paru",
            PackageHelper::Yay => "yay",
        }
    }

    pub fn can_install_aur(&self) -> bool {
        matches!(self, PackageHelper::Paru | PackageHelper::Yay)
    }
}

pub struct PackageManager {
    helper: PackageHelper,
}

impl PackageManager {
    pub fn new() -> Self {
        Self {
            helper: PackageHelper::detect(),
        }
    }

    pub fn with_helper(helper: PackageHelper) -> Self {
        Self { helper }
    }

    /// Install official repository packages
    pub fn install_packages(
        &self,
        packages: &[String],
        options: &SyncOptions,
    ) -> Result<Vec<String>> {
        if packages.is_empty() {
            return Ok(Vec::new());
        }

        let mut installed = Vec::new();
        let mut args = vec!["-S", "--noconfirm"];

        if !options.force {
            args.push("--needed");
        }

        if options.dry_run {
            log::info!("DRY RUN: Would install packages: {:?}", packages);
            return Ok(packages.to_vec());
        }

        log::info!(
            "Installing {} packages with {}",
            packages.len(),
            self.helper.command()
        );

        if options.parallel && packages.len() > 1 {
            // Install packages in parallel (in chunks)
            for chunk in packages.chunks(5) {
                match self.install_package_chunk(chunk, &args) {
                    Ok(pkgs) => installed.extend(pkgs),
                    Err(e) => log::error!("Failed to install chunk: {}", e),
                }
            }
        } else {
            // Install sequentially
            for package in packages {
                match self.install_single_package(package, &args) {
                    Ok(_) => installed.push(package.clone()),
                    Err(e) => log::error!("Failed to install {}: {}", package, e),
                }
            }
        }

        Ok(installed)
    }

    /// Install AUR packages
    pub fn install_aur_packages(
        &self,
        packages: &[String],
        options: &SyncOptions,
    ) -> Result<Vec<String>> {
        if packages.is_empty() {
            return Ok(Vec::new());
        }

        if !self.helper.can_install_aur() {
            return Err(S1bCr4ftError::package(
                "AUR packages require paru or yay. Please install one of them first.",
            ));
        }

        if options.dry_run {
            log::info!("DRY RUN: Would install AUR packages: {:?}", packages);
            return Ok(packages.to_vec());
        }

        log::info!(
            "Installing {} AUR packages with {}",
            packages.len(),
            self.helper.command()
        );

        let mut installed = Vec::new();
        let args = vec!["-S", "--noconfirm", "--needed"];

        for package in packages {
            match self.install_single_package(package, &args) {
                Ok(_) => installed.push(package.clone()),
                Err(e) => log::error!("Failed to install AUR package {}: {}", package, e),
            }
        }

        Ok(installed)
    }

    fn install_single_package(&self, package: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(self.helper.command())
            .args(args)
            .arg(package)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| {
                S1bCr4ftError::package(format!(
                    "Failed to execute {}: {}",
                    self.helper.command(),
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(S1bCr4ftError::package(format!(
                "Failed to install package: {}",
                package
            )));
        }

        Ok(())
    }

    fn install_package_chunk(&self, packages: &[String], args: &[&str]) -> Result<Vec<String>> {
        let mut cmd_args = args.to_vec();
        cmd_args.extend(packages.iter().map(|s| s.as_str()));

        let output = Command::new(self.helper.command())
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| {
                S1bCr4ftError::package(format!(
                    "Failed to execute {}: {}",
                    self.helper.command(),
                    e
                ))
            })?;

        if output.status.success() {
            Ok(packages.to_vec())
        } else {
            Err(S1bCr4ftError::package("Failed to install package chunk"))
        }
    }

    /// Execute system commands
    pub fn execute_commands(&self, commands: &[String], dry_run: bool) -> Result<Vec<String>> {
        let mut executed = Vec::new();

        for command in commands {
            if dry_run {
                log::info!("DRY RUN: Would execute: {}", command);
                executed.push(command.clone());
                continue;
            }

            log::info!("Executing: {}", command);

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .map_err(|e| S1bCr4ftError::package(format!("Failed to execute command: {}", e)))?;

            if output.status.success() {
                executed.push(command.clone());
            } else {
                log::error!("Command failed: {}", command);
            }
        }

        Ok(executed)
    }

    /// Full sync operation
    pub fn sync(
        &self,
        packages: &[String],
        aur_packages: &[String],
        commands: &[String],
        options: &SyncOptions,
    ) -> Result<SyncReport> {
        let start = Instant::now();

        let packages_installed = self.install_packages(packages, options)?;
        let aur_installed = self.install_aur_packages(aur_packages, options)?;
        let commands_executed = self.execute_commands(commands, options.dry_run)?;

        let mut all_installed = packages_installed;
        all_installed.extend(aur_installed);

        let duration_secs = start.elapsed().as_secs();

        Ok(SyncReport {
            packages_installed: all_installed,
            packages_failed: Vec::new(),
            commands_executed,
            duration_secs,
        })
    }

    /// Check if a package is installed
    pub fn is_installed(&self, package: &str) -> bool {
        Command::new("pacman")
            .args(["-Q", package])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Update system
    pub fn update_system(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            log::info!("DRY RUN: Would update system");
            return Ok(());
        }

        log::info!("Updating system with {}", self.helper.command());

        let output = Command::new(self.helper.command())
            .args(["-Syu", "--noconfirm"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| S1bCr4ftError::package(format!("Failed to update system: {}", e)))?;

        if !output.status.success() {
            return Err(S1bCr4ftError::package("System update failed"));
        }

        Ok(())
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_helper_detection() {
        let helper = PackageHelper::detect();
        assert!(matches!(
            helper,
            PackageHelper::Pacman | PackageHelper::Paru | PackageHelper::Yay
        ));
    }

    #[test]
    fn test_sync_options() {
        let options = SyncOptions {
            dry_run: true,
            force: false,
            parallel: true,
        };
        assert!(options.dry_run);
    }
}
