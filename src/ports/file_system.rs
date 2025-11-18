use crate::{FileMetadata, Symlink, UhpmError};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[async_trait]
pub trait FileSystemOperations: Send + Sync + Clone {
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, UhpmError>;

    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), UhpmError>;

    async fn create_dir(&self, path: &Path) -> Result<(), UhpmError>;

    async fn create_dir_all(&self, path: &Path) -> Result<(), UhpmError>;

    async fn remove(&self, path: &Path) -> Result<(), UhpmError>;

    async fn remove_dir_all(&self, path: &Path) -> Result<(), UhpmError>;

    async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), UhpmError>;

    async fn move_file(&self, from: &Path, to: &Path) -> Result<(), UhpmError>;

    async fn exists(&self, path: &Path) -> bool;

    async fn metadata(&self, path: &Path) -> Result<FileMetadata, UhpmError>;

    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, UhpmError>;

    async fn create_symlink(&self, symlink: &Symlink) -> Result<(), UhpmError>;

    async fn remove_symlink(&self, path: &Path) -> Result<(), UhpmError>;

    async fn read_symlink(&self, path: &Path) -> Result<PathBuf, UhpmError>;

    async fn is_symlink(&self, path: &Path) -> bool;

    async fn set_permissions(&self, path: &Path, permissions: u32) -> Result<(), UhpmError>;
}
