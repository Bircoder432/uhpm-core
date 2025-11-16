use semver::VersionReq;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UhpmError {
    #[error("Package `{0}` not found")]
    PackageNotFound(String),

    #[error("Version `{version}` of `{package}` does not satisfy `{required}`")]
    VersionMismatch {
        package: String,
        version: String,
        required: VersionReq,
    },

    #[error("Dependency resolution failed: {0}")]
    ResolutionError(String),

    #[error("Repository unreachable: {0}")]
    RepositoryUnavailable(String),

    #[error("Failed to read file: {0}")]
    IoError(String),

    #[error("Checksum mismatch for `{0}`")]
    ChecksumMismatch(String),

    #[error("Archive format error: {0}")]
    ArchiveError(String),

    #[error("Invalid package format in `{0}`")]
    InvalidPackage(PathBuf),
}
