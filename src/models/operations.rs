use crate::PackageId;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub package_id: PackageId,
    pub installed_files: Vec<PathBuf>,
    pub symlinks_created: usize,
}
