use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::path::PathBuf;
use tar::{Archive, Builder};

use crate::{PackageId, Symlink, SymlinkType, UhpmError, ports::FileSystemOperations};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub provides: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
}

pub struct PackageFilesRepository<FS>
where
    FS: FileSystemOperations,
{
    file_system: FS,
    packages_dir: PathBuf,
}

impl<FS> PackageFilesRepository<FS>
where
    FS: FileSystemOperations,
{
    pub fn new(file_system: FS, packages_dir: PathBuf) -> Self {
        Self {
            file_system,
            packages_dir,
        }
    }

    pub fn get_package_path(&self, package_id: &PackageId) -> PathBuf {
        self.packages_dir.join(package_id.as_str())
    }

    pub fn get_package_meta_path(&self, package_id: &PackageId) -> PathBuf {
        self.get_package_path(package_id).join("meta.toml")
    }

    pub fn get_package_instlist_path(&self, package_id: &PackageId) -> PathBuf {
        self.get_package_path(package_id).join("instlist")
    }
}

impl<FS> PackageFilesRepository<FS>
where
    FS: FileSystemOperations + Send + Sync,
{
    pub async fn extract_package(
        &self,
        package_id: &PackageId,
        package_data: &[u8],
    ) -> Result<(), UhpmError> {
        let package_path = self.get_package_path(package_id);

        self.file_system.create_dir_all(&package_path).await?;

        let temp_path = package_path.join("package.uhp");
        self.file_system
            .write_file(&temp_path, package_data)
            .await?;

        let tar_gz = std::fs::File::open(&temp_path)
            .map_err(|e| UhpmError::FileSystemError(e.to_string()))?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        archive
            .unpack(&package_path)
            .map_err(|e| UhpmError::FileSystemError(format!("Failed to extract package: {}", e)))?;

        self.file_system.remove(&temp_path).await?;

        Ok(())
    }

    pub async fn remove_package_files(&self, package_id: &PackageId) -> Result<(), UhpmError> {
        let package_path = self.get_package_path(package_id);

        if self.file_system.exists(&package_path).await {
            self.file_system.remove_dir_all(&package_path).await?;
        }

        Ok(())
    }

    pub async fn load_package_meta(
        &self,
        package_id: &PackageId,
    ) -> Result<Option<PackageMeta>, UhpmError> {
        let meta_path = self.get_package_meta_path(package_id);

        if !self.file_system.exists(&meta_path).await {
            return Ok(None);
        }

        let data = self.file_system.read_file(&meta_path).await?;
        let meta_str = std::str::from_utf8(&data)
            .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;
        let meta: PackageMeta =
            toml::from_str(meta_str).map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        Ok(Some(meta))
    }

    pub async fn save_package_meta(
        &self,
        package_id: &PackageId,
        meta: &PackageMeta,
    ) -> Result<(), UhpmError> {
        let meta_path = self.get_package_meta_path(package_id);

        if let Some(parent) = meta_path.parent() {
            self.file_system.create_dir_all(parent).await?;
        }

        let toml_str =
            toml::to_string(meta).map_err(|e| UhpmError::SerializationError(e.to_string()))?;

        self.file_system
            .write_file(&meta_path, toml_str.as_bytes())
            .await?;
        Ok(())
    }

    pub async fn load_package_instlist(
        &self,
        package_id: &PackageId,
    ) -> Result<Vec<Symlink>, UhpmError> {
        let instlist_path = self.get_package_instlist_path(package_id);
        let package_path = self.get_package_path(package_id);

        if !self.file_system.exists(&instlist_path).await {
            return Ok(Vec::new());
        }

        let data = self.file_system.read_file(&instlist_path).await?;
        let content = std::str::from_utf8(&data)
            .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        let mut symlinks = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let source_relative = PathBuf::from(parts[0]);
                let target_absolute = PathBuf::from(parts[1]);

                let source_absolute = package_path.join(&source_relative);

                let link_type =
                    if let Ok(metadata) = self.file_system.metadata(&source_absolute).await {
                        if metadata.is_directory() {
                            SymlinkType::Directory
                        } else {
                            SymlinkType::File
                        }
                    } else {
                        SymlinkType::File
                    };

                let symlink = Symlink::new(source_absolute, target_absolute, link_type);
                symlinks.push(symlink);
            }
        }

        Ok(symlinks)
    }

    pub async fn create_symlinks_from_instlist(
        &self,
        package_id: &PackageId,
    ) -> Result<Vec<Symlink>, UhpmError> {
        let symlinks = self.load_package_instlist(package_id).await?;

        for symlink in &symlinks {
            if let Some(parent) = symlink.target.parent() {
                self.file_system.create_dir_all(parent).await?;
            }

            self.file_system.create_symlink(symlink).await?;
        }

        Ok(symlinks)
    }

    pub async fn copy_files_direct(&self, package_id: &PackageId) -> Result<(), UhpmError> {
        let symlinks = self.load_package_instlist(package_id).await?;

        for symlink in symlinks {
            if let Some(parent) = symlink.target.parent() {
                self.file_system.create_dir_all(parent).await?;
            }

            self.file_system
                .copy_file(&symlink.source, &symlink.target)
                .await?;
        }

        Ok(())
    }

    pub async fn remove_installation_files(&self, package_id: &PackageId) -> Result<(), UhpmError> {
        let symlinks = self.load_package_instlist(package_id).await?;

        for symlink in symlinks {
            if self.file_system.exists(&symlink.target).await {
                if self.file_system.is_symlink(&symlink.target).await {
                    self.file_system.remove_symlink(&symlink.target).await?;
                } else {
                    self.file_system.remove(&symlink.target).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn package_exists(&self, package_id: &PackageId) -> bool {
        let package_path = self.get_package_path(package_id);
        self.file_system.exists(&package_path).await
    }

    pub async fn verify_package_integrity(
        &self,
        package_id: &PackageId,
    ) -> Result<bool, UhpmError> {
        let _package_path = self.get_package_path(package_id);

        let meta_path = self.get_package_meta_path(package_id);
        let instlist_path = self.get_package_instlist_path(package_id);

        let meta_exists = self.file_system.exists(&meta_path).await;
        let instlist_exists = self.file_system.exists(&instlist_path).await;

        if !meta_exists || !instlist_exists {
            return Ok(false);
        }

        let symlinks = self.load_package_instlist(package_id).await?;
        for symlink in symlinks {
            if !self.file_system.exists(&symlink.source).await {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn create_package_archive(
        &self,
        package_id: &PackageId,
    ) -> Result<Vec<u8>, UhpmError> {
        let package_path = self.get_package_path(package_id);

        if !self.file_system.exists(&package_path).await {
            return Err(UhpmError::PackageNotFound(package_id.as_str().to_string()));
        }

        let mut archive_data = Vec::new();
        {
            let enc = GzEncoder::new(&mut archive_data, Compression::default());
            let mut tar = Builder::new(enc);

            self.add_directory_to_tar(&mut tar, &package_path, &package_path)
                .await?;

            tar.finish()
                .map_err(|e| UhpmError::SerializationError(e.to_string()))?;
        }

        Ok(archive_data)
    }

    async fn add_directory_to_tar(
        &self,
        tar: &mut Builder<GzEncoder<&mut Vec<u8>>>,
        base_path: &PathBuf,
        current_path: &PathBuf,
    ) -> Result<(), UhpmError> {
        if let Ok(entries) = self.file_system.read_dir(current_path).await {
            for entry in entries {
                let metadata = self.file_system.metadata(&entry).await?;

                if metadata.is_directory() {
                    // Используем Box::pin для рекурсивного вызова
                    let future = Box::pin(self.add_directory_to_tar(tar, base_path, &entry));
                    future.await?;
                } else {
                    let relative_path = entry
                        .strip_prefix(base_path)
                        .map_err(|e| UhpmError::FileSystemError(e.to_string()))?;

                    let content = self.file_system.read_file(&entry).await?;

                    let mut header = tar::Header::new_gnu();
                    header
                        .set_path(relative_path)
                        .map_err(|e| UhpmError::SerializationError(e.to_string()))?;
                    header.set_size(content.len() as u64);
                    header.set_cksum();

                    tar.append(&header, &content[..])
                        .map_err(|e| UhpmError::SerializationError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }
}
