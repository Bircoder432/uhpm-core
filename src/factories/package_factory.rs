// src/factories/package_factory.rs

use crate::{Checksum, Dependency, Package, PackageId, PackageSource, Target, UhpmError};
use semver::Version;

/// Factory for creating Package entities with validation and business rules.
///
/// The PackageFactory ensures that all Package instances are created in a valid state
/// and enforces business rules during creation.
#[derive(Debug, Clone)]
pub struct PackageFactory;

impl PackageFactory {
    /// Creates a new Package with validation.
    ///
    /// # Arguments
    /// * `name` - Package name (must be non-empty)
    /// * `version` - Package version
    /// * `author` - Package author (must be non-empty)
    /// * `source` - Package source (Git, HTTP, or Local)
    /// * `target` - Target platform
    /// * `checksum` - Optional checksum for verification
    /// * `dependencies` - Package dependencies
    ///
    /// # Returns
    /// * `Ok(Package)` - Valid package instance
    /// * `Err(UhpmError)` - Validation error
    ///
    /// # Examples
    /// ```
    /// use semver::Version;
    /// use uhpm_core::{Target, PackageSource, factories::PackageFactory};
    ///
    /// let package = PackageFactory::create(
    ///     "my-package".to_string(),
    ///     Version::parse("1.0.0").unwrap(),
    ///     "author".to_string(),
    ///     PackageSource::Local { path: "/path".into() },
    ///     Target::current(),
    ///     None,
    ///     vec![]
    /// );
    /// ```
    pub fn create(
        name: String,
        version: Version,
        author: String,
        source: PackageSource,
        target: Target,
        checksum: Option<Checksum>,
        dependencies: Vec<Dependency>,
    ) -> Result<Package, UhpmError> {
        // Validate name
        if name.trim().is_empty() {
            return Err(UhpmError::ValidationError(
                "Package name cannot be empty or whitespace".to_string(),
            ));
        }

        // Validate name format (alphanumeric, hyphens, underscores)
        if !Self::is_valid_package_name(&name) {
            return Err(UhpmError::ValidationError(format!(
                "Invalid package name '{}'. Must contain only alphanumeric characters, hyphens, and underscores",
                name
            )));
        }

        // Validate author
        if author.trim().is_empty() {
            return Err(UhpmError::ValidationError(
                "Package author cannot be empty".to_string(),
            ));
        }

        // Validate version (semver already validates format)
        if version.pre.contains("broken") || version.pre.contains("unstable") {
            return Err(UhpmError::ValidationError(
                "Version pre-release tags cannot contain 'broken' or 'unstable'".to_string(),
            ));
        }

        // Validate source
        Self::validate_source(&source)?;

        // Create package ID
        let id = PackageId::new(&name, &version);

        // Create package instance
        let package = Package::new(
            id,
            name,
            version,
            author,
            source,
            target,
            checksum,
            dependencies.into_iter().collect(),
            false,
            false,
        );

        Ok(package)
    }

    /// Creates a package from remote metadata (for downloaded packages)
    pub fn from_remote_metadata(
        name: String,
        version: Version,
        author: String,
        source: PackageSource,
        target: Target,
        checksum: Option<Checksum>,
        dependencies: Vec<Dependency>,
    ) -> Result<Package, UhpmError> {
        let mut package = Self::create(
            name,
            version,
            author,
            source,
            target,
            checksum,
            dependencies,
        )?;

        // Additional validation for remote packages
        if package.checksum().is_none() {
            return Err(UhpmError::ValidationError(
                "Remote packages must include checksum for verification".to_string(),
            ));
        }

        Ok(package)
    }

    /// Creates a package from local files (for existing installations)
    pub fn from_local_files(
        name: String,
        version: Version,
        author: String,
        source: PackageSource,
        target: Target,
        dependencies: Vec<Dependency>,
    ) -> Result<Package, UhpmError> {
        Self::create(name, version, author, source, target, None, dependencies)
    }

    /// Validates package name format
    fn is_valid_package_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 50
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            && name
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
    }

    /// Validates package source
    fn validate_source(source: &PackageSource) -> Result<(), UhpmError> {
        match source {
            PackageSource::Git { url, release: _ } => {
                if url.trim().is_empty() {
                    return Err(UhpmError::ValidationError(
                        "Git URL cannot be empty".to_string(),
                    ));
                }
                // Basic URL validation
                if !url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.starts_with("git@")
                {
                    return Err(UhpmError::ValidationError(
                        "Git URL must be http, https, or git@ format".to_string(),
                    ));
                }
            }
            PackageSource::Http { url } => {
                if url.trim().is_empty() {
                    return Err(UhpmError::ValidationError(
                        "HTTP URL cannot be empty".to_string(),
                    ));
                }
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(UhpmError::ValidationError(
                        "HTTP URL must start with http:// or https://".to_string(),
                    ));
                }
            }
            PackageSource::Local { path } => {
                if path.as_os_str().is_empty() {
                    return Err(UhpmError::ValidationError(
                        "Local path cannot be empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_package() {
        let package = PackageFactory::create(
            "my-package".to_string(),
            Version::parse("1.0.0").unwrap(),
            "John Doe".to_string(),
            PackageSource::Local {
                path: "/tmp".into(),
            },
            Target::current(),
            None,
            vec![],
        )
        .unwrap();

        assert_eq!(package.name(), "my-package");
        assert!(!package.is_installed());
        assert!(!package.is_active());
    }

    #[test]
    fn test_invalid_package_name() {
        let result = PackageFactory::create(
            "".to_string(),
            Version::parse("1.0.0").unwrap(),
            "John Doe".to_string(),
            PackageSource::Local {
                path: "/tmp".into(),
            },
            Target::current(),
            None,
            vec![],
        );

        assert!(result.is_err());
    }
}
