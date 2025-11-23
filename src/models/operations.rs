use semver::Version;

use crate::{PackageId, Target};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub package_id: PackageId,
    pub installed_files: Vec<PathBuf>,
    pub symlinks_created: usize,
}

#[derive(Debug, Clone)]
pub struct RemovalResult {
    pub package_id: PackageId,
    pub removed_files: usize,
    pub freed_space: usize,
}

#[derive(Debug, Clone)]
pub struct SwitchResult {
    pub package_name: String,
    pub from_version: Option<Version>,
    pub to_version: Version,
    pub removed_files: usize,
    pub installed_files: usize,
    pub warnings: Vec<String>,
}
