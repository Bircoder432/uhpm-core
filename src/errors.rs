use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UhpmError {
    #[error("Package `{0}` not found")]
    PackageNotFound(String),

    #[error("Installation `{0}` not found")]
    InstallationNotFound(String),

    #[error("No version of `{package}` matches `{required}`")]
    VersionMismatch {
        package: String,
        required: semver::VersionReq,
    },

    #[error("Dependency resolution failed: {0}")]
    ResolutionError(String),

    #[error("Dependency conflict: {0}")]
    DependencyConflict(String),

    #[error("Repository unreachable: {0}")]
    RepositoryUnavailable(String),

    #[error("Package already installed: {0}")]
    PackageAlreadyInstalled(String),

    #[error("No newer version available for package: {0}")]
    NoNewVersion(String),

    #[error("Package is currently active and cannot be removed")]
    PackageIsActive,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid package format in `{0}`")]
    InvalidPackage(PathBuf),

    #[error("Checksum verification failed for package: {0}")]
    ChecksumMismatch(String),

    #[error("Unsupported target platform: {0}")]
    UnsupportedTarget(String),

    #[error("Installation failed: {0}")]
    InstallationError(String),

    #[error("Failed to create symbolic link: {0}")]
    SymlinkError(String),

    #[error("Failed to remove package files: {0}")]
    RemovalError(String),

    #[error("Version switch failed: {0}")]
    SwitchError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Download failed: {0}")]
    DownloadError(String),

    #[error("Repository index corrupted: {0}")]
    RepositoryCorrupted(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File system error: {0}")]
    FileSystemError(#[from] crate::models::file_system::FsError),

    #[error("Insufficient permissions: {0}")]
    PermissionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("External tool error: {0}")]
    ExternalToolError(String),

    #[error("Database operation failed: {0}")]
    RusqliteError(#[from] rusqlite::Error),
}

impl UhpmError {
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }

    pub fn installation<S: Into<String>>(msg: S) -> Self {
        Self::InstallationError(msg.into())
    }

    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::NetworkError(msg.into())
    }
}
