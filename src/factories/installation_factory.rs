// src/factories/installation_factory.rs

use crate::{FileMetadata, Installation, InstallationId, PackageId, Symlink, UhpmError};
use std::collections::HashMap;
use std::path::PathBuf;

/// Factory for creating Installation entities with validation.
///
/// Ensures that installations are created with proper metadata and
/// validates the installation state.
#[derive(Debug, Clone)]
pub struct InstallationFactory;

impl InstallationFactory {
    /// Creates a new Installation for a package.
    ///
    /// # Arguments
    /// * `package_id` - ID of the package being installed
    ///
    /// # Returns
    /// * `Installation` - New installation instance
    ///
    pub fn create(package_id: PackageId) -> Installation {
        Installation::new(
            InstallationId::new(),
            package_id,
            HashMap::new(),
            Vec::new(),
            chrono::Utc::now(),
            false,
        )
    }

    /// Creates an installation from database data (for reconstruction).
    ///
    /// # Arguments
    /// * `installation_id` - Existing installation ID
    /// * `package_id` - Package ID
    /// * `installed_at` - Installation timestamp
    /// * `active` - Whether installation is active
    ///
    /// # Returns
    /// * `Installation` - Reconstructed installation
    pub fn from_existing(
        installation_id: InstallationId,
        package_id: PackageId,
        installed_at: chrono::DateTime<chrono::Utc>,
        active: bool,
    ) -> Installation {
        let mut installation = Installation::new(
            installation_id,
            package_id,
            HashMap::new(),
            Vec::new(),
            installed_at,
            active, // Will set properly below
        );

        installation
    }

    /// Validates if an installation can be activated.
    ///
    /// # Arguments
    /// * `installation` - Installation to validate
    ///
    /// # Returns
    /// * `Ok(())` - Installation can be activated
    /// * `Err(UhpmError)` - Installation cannot be activated
    pub fn validate_activation(installation: &Installation) -> Result<(), UhpmError> {
        if installation.installed_files().is_empty() {
            return Err(UhpmError::ValidationError(
                "Cannot activate installation with no files".to_string(),
            ));
        }

        // Check if all required files are present in the installation
        if installation.symlinks().is_empty() && installation.installed_files().is_empty() {
            return Err(UhpmError::ValidationError(
                "Installation has no files or symlinks".to_string(),
            ));
        }

        Ok(())
    }

    /// Creates a file metadata entry for installation tracking.
    ///
    /// # Arguments
    /// * `path` - File path
    /// * `size` - File size in bytes
    /// * `checksum_algorithm` - Optional checksum algorithm
    /// * `checksum_hash` - Optional checksum hash
    ///
    /// # Returns
    /// * `FileMetadata` - File metadata instance
    pub fn create_file_metadata(
        &self,
        path: PathBuf,
        size: u64,
        checksum_algorithm: Option<String>,
        checksum_hash: Option<String>,
    ) -> Result<FileMetadata, UhpmError> {
        if path.as_os_str().is_empty() {
            return Err(UhpmError::ValidationError(
                "File path cannot be empty".to_string(),
            ));
        }

        let mut metadata = FileMetadata::new(path, size);

        if let (Some(algorithm), Some(hash)) = (checksum_algorithm, checksum_hash) {
            if algorithm.trim().is_empty() || hash.trim().is_empty() {
                return Err(UhpmError::ValidationError(
                    "Checksum algorithm and hash cannot be empty".to_string(),
                ));
            }
            metadata.checksum = Some(crate::FileChecksum { algorithm, hash });
        }

        Ok(metadata)
    }

    /// Validates a symlink before adding to installation.
    ///
    /// # Arguments
    /// * `symlink` - Symlink to validate
    ///
    /// # Returns
    /// * `Ok(())` - Symlink is valid
    /// * `Err(UhpmError)` - Symlink is invalid
    pub fn validate_symlink(&self, symlink: &Symlink) -> Result<(), UhpmError> {
        symlink.validate()?;

        // Additional business rules
        if symlink.source.as_os_str().is_empty() {
            return Err(UhpmError::ValidationError(
                "Symlink source cannot be empty".to_string(),
            ));
        }

        if symlink.target.as_os_str().is_empty() {
            return Err(UhpmError::ValidationError(
                "Symlink target cannot be empty".to_string(),
            ));
        }

        // Prevent symlinking to system directories in home mode
        if Self::is_system_directory(&symlink.target) {
            return Err(UhpmError::ValidationError(format!(
                "Cannot create symlink to system directory: {}",
                symlink.target.display()
            )));
        }

        Ok(())
    }

    /// Checks if a path is a system directory (for safety).
    fn is_system_directory(path: &PathBuf) -> bool {
        let system_dirs = [
            "/bin",
            "/sbin",
            "/usr/bin",
            "/usr/sbin",
            "/lib",
            "/usr/lib",
            "/etc",
            "/var",
        ];

        system_dirs.iter().any(|&dir| path.starts_with(dir))
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;

    #[test]
    fn test_create_installation() {
        let package_id = PackageId::new("test-pkg", &Version::parse("1.0.0").unwrap());
        let installation = InstallationFactory::create(package_id);

        assert!(!installation.is_active());
        assert!(installation.installed_files().is_empty());
        assert!(installation.symlinks().is_empty());
    }

    #[test]
    fn test_validate_activation_empty_installation() {
        let package_id = PackageId::new("test-pkg", &Version::parse("1.0.0").unwrap());
        let installation = InstallationFactory::create(package_id);

        let result = InstallationFactory::validate_activation(&installation);
        assert!(result.is_err());
    }
}
