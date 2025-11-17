use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use md5::Digest as Md5Digest;
use sha1::Digest as Sha1Digest;
use sha2::Digest as Sha2Digest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub checksum: Option<FileChecksum>,
    pub permissions: FilePermissions,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub file_type: FileType,
}

impl FileMetadata {
    pub fn new(path: PathBuf, size: u64) -> Self {
        let now = Utc::now();
        Self {
            path,
            size,
            checksum: None,
            permissions: FilePermissions::default(),
            created_at: now,
            modified_at: now,
            file_type: FileType::Regular,
        }
    }

    pub fn with_checksum(mut self, algorithm: &str, hash: &str) -> Self {
        self.checksum = Some(FileChecksum {
            algorithm: algorithm.to_string(),
            hash: hash.to_string(),
        });
        self
    }

    pub fn with_permissions(mut self, permissions: FilePermissions) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_file_type(mut self, file_type: FileType) -> Self {
        self.file_type = file_type;
        self
    }

    pub fn is_executable(&self) -> bool {
        self.permissions.is_executable()
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self.file_type, FileType::Symlink)
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    pub fn verify_checksum(&self, data: &[u8]) -> Result<bool, crate::UhpmError> {
        if let Some(checksum) = &self.checksum {
            let actual_hash = match checksum.algorithm.as_str() {
                "sha256" => sha256_hash(data),
                "sha1" => sha1_hash(data),
                "md5" => md5_hash(data),
                algo => {
                    return Err(crate::UhpmError::ValidationError(format!(
                        "Unsupported checksum algorithm: {}",
                        algo
                    )));
                }
            };
            Ok(actual_hash == checksum.hash)
        } else {
            Ok(true)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FileChecksum {
    pub algorithm: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FilePermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }
}

impl FilePermissions {
    pub fn executable() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
        }
    }

    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }

    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }

    pub fn is_executable(&self) -> bool {
        self.execute
    }

    pub fn octal(&self) -> u32 {
        let mut result = 0;
        if self.read {
            result |= 0o400;
        }
        if self.write {
            result |= 0o200;
        }
        if self.execute {
            result |= 0o100;
        }
        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "directory")]
    Directory,
    #[serde(rename = "symlink")]
    Symlink,
    #[serde(rename = "executable")]
    Executable,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Regular => write!(f, "regular"),
            Self::Directory => write!(f, "directory"),
            Self::Symlink => write!(f, "symlink"),
            Self::Executable => write!(f, "executable"),
        }
    }
}

fn sha256_hash(data: &[u8]) -> String {
    use sha2::Sha256;
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn sha1_hash(data: &[u8]) -> String {
    use sha1::Sha1;
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn md5_hash(data: &[u8]) -> String {
    format!("{:x}", md5::compute(data))
}
