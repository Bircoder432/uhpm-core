use crate::{FileMetadata, PackageId, Symlink, UhpmError};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
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
    pub fn new(package_id: PackageId) -> Self {
        Self {
            id: InstallationId::new(),
            package_id,
            installed_files: HashMap::new(),
            symlinks: Vec::new(),
            installed_at: chrono::Utc::now(),
            active: false,
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
        for (path, metadata) in &self.installed_files {
            if !path.exists() {
                return Err(UhpmError::InstallationError(format!(
                    "Missing installed file: {}",
                    path.display()
                )));
            }
            // TODO
        }
        Ok(())
    }
}
