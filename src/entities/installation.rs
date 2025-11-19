use crate::{FileMetadata, PackageId, Symlink, UhpmError};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstallationId(Uuid);

impl InstallationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for InstallationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for InstallationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for InstallationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl TryFrom<&str> for InstallationId {
    type Error = crate::UhpmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let uuid = Uuid::parse_str(value)
            .map_err(|e| UhpmError::ValidationError(format!("Invalid installation ID: {}", e)))?;
        Ok(Self(uuid))
    }
}

pub struct Installation {
    id: InstallationId,
    package_id: PackageId,
    installed_files: HashMap<PathBuf, FileMetadata>,
    symlinks: Vec<Symlink>,
    installed_at: chrono::DateTime<chrono::Utc>,
    active: bool,
}

impl Installation {
    pub fn new(
        id: InstallationId,
        package_id: PackageId,
        installed_files: HashMap<PathBuf, FileMetadata>,
        symlinks: Vec<Symlink>,
        installed_at: chrono::DateTime<chrono::Utc>,
        active: bool,
    ) -> Self {
        Self {
            id: id,
            package_id: package_id,
            installed_files: installed_files,
            symlinks: symlinks,
            installed_at: installed_at,
            active: active,
        }
    }

    pub fn add_installed_file(&mut self, path: PathBuf, metadata: FileMetadata) {
        self.installed_files.insert(path, metadata);
    }

    pub fn add_symlink(&mut self, symlink: Symlink) {
        self.symlinks.push(symlink);
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn verify_integrity(&self) -> Result<(), crate::UhpmError> {
        for (path, _metadata) in &self.installed_files {
            if !path.exists() {
                return Err(UhpmError::InstallationError(format!(
                    "Missing installed file: {}",
                    path.display()
                )));
            }
        }
        Ok(())
    }

    pub fn id(&self) -> &InstallationId {
        &self.id
    }

    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    pub fn installed_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.installed_at
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn installed_files(&self) -> &HashMap<PathBuf, FileMetadata> {
        &self.installed_files
    }

    pub fn symlinks(&self) -> &Vec<Symlink> {
        &self.symlinks
    }

    pub fn set_id(&mut self, id: InstallationId) {
        self.id = id;
    }

    pub fn set_installed_at(&mut self, installed_at: chrono::DateTime<chrono::Utc>) {
        self.installed_at = installed_at;
    }
}
