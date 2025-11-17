use crate::{
    Dependency, Package, PackageReference, Repository, RepositoryIndex, UhpmError,
    paths::UhpmPaths,
    ports::{FileSystemOperations, PackageRepository},
};
use async_trait::async_trait;
use semver::Version;
use std::path::PathBuf;

pub struct LocalPackagesRepository<FS, P>
where
    FS: FileSystemOperations,
    P: UhpmPaths,
{
    file_system: FS,
    paths: P,
    repository: Repository,
}

impl<FS, P> LocalPackagesRepository<FS, P>
where
    FS: FileSystemOperations,
    P: UhpmPaths,
{
    pub fn new(file_system: FS, paths: P, repository: Repository) -> Result<Self, UhpmError> {
        Ok(Self {
            file_system,
            paths,
            repository,
        })
    }

    fn get_package_meta_path(&self, package_ref: &PackageReference) -> PathBuf {
        self.paths
            .packages_dir()
            .join(&package_ref.name)
            .join(&package_ref.version.to_string())
            .join("meta.toml")
    }
}

#[async_trait]
impl<FS, P> PackageRepository for LocalPackagesRepository<FS, P>
where
    FS: FileSystemOperations + Send + Sync,
    P: UhpmPaths + Send + Sync,
{
    async fn get_package(&self, package_ref: &PackageReference) -> Result<Package, UhpmError> {
        let meta_path = self.get_package_meta_path(package_ref);

        if !self.file_system.exists(&meta_path).await {
            return Err(UhpmError::PackageNotFound(package_ref.to_string()));
        }

        // TODO
        Err(UhpmError::ValidationError("Not implemented".into()))
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, UhpmError> {
        let packages_dir = self.paths.packages_dir();
        let mut results = Vec::new();

        if self.file_system.exists(&packages_dir).await {
            // TODO
        }

        Ok(results)
    }

    async fn get_package_versions(&self, package_name: &str) -> Result<Vec<String>, UhpmError> {
        let packages_dir = self.paths.packages_dir();
        let package_dir = packages_dir.join(package_name);
        let mut versions = Vec::new();

        if self.file_system.exists(&package_dir).await {
            if let Ok(entries) = self.file_system.read_dir(&package_dir).await {
                for entry in entries {
                    if let Some(version_str) = entry.file_name().and_then(|n| n.to_str()) {
                        if Version::parse(version_str).is_ok() {
                            versions.push(version_str.to_string());
                        }
                    }
                }
            }
        }

        versions.sort_by(|a, b| Version::parse(a).unwrap().cmp(&Version::parse(b).unwrap()));

        Ok(versions)
    }

    async fn get_latest_version(&self, package_name: &str) -> Result<String, UhpmError> {
        let versions = self.get_package_versions(package_name).await?;
        versions
            .last()
            .cloned()
            .ok_or_else(|| UhpmError::PackageNotFound(package_name.to_string()))
    }

    async fn resolve_dependencies(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Package>, UhpmError> {
        let mut resolved_packages = Vec::new();

        for dependency in dependencies {
            let versions = self.get_package_versions(&dependency.name).await?;

            if let Some(version_str) = versions.into_iter().rev().find(|v| {
                Version::parse(v)
                    .map(|ver| dependency.matches_version(&ver))
                    .unwrap_or(false)
            }) {
                let version = Version::parse(&version_str)
                    .map_err(|e| UhpmError::ValidationError(e.to_string()))?;

                let package_ref = PackageReference::new(dependency.name.clone(), version);
                let package = self.get_package(&package_ref).await?;
                resolved_packages.push(package);
            }
        }

        Ok(resolved_packages)
    }

    async fn download_package(&self, package_ref: &PackageReference) -> Result<Vec<u8>, UhpmError> {
        let meta_path = self.get_package_meta_path(package_ref);
        if !self.file_system.exists(&meta_path).await {
            return Err(UhpmError::PackageNotFound(package_ref.to_string()));
        }

        // TODO
        Ok(Vec::new())
    }

    async fn get_index(&self) -> Result<RepositoryIndex, UhpmError> {
        let packages_dir = self.paths.packages_dir();
        let mut packages = Vec::new();

        if self.file_system.exists(&packages_dir).await {
            if let Ok(entries) = self.file_system.read_dir(&packages_dir).await {
                for package_dir in entries {
                    if let Some(package_name) = package_dir.file_name().and_then(|n| n.to_str()) {
                        let versions = self.get_package_versions(package_name).await?;
                        if !versions.is_empty() {
                            packages.push(crate::RepositoryPackageEntry {
                                name: package_name.to_string(),
                                versions,
                            });
                        }
                    }
                }
            }
        }

        Ok(RepositoryIndex {
            name: "local".to_string(),
            url: packages_dir.to_string_lossy().to_string(),
            packages,
        })
    }

    async fn update_index(&self) -> Result<RepositoryIndex, UhpmError> {
        self.get_index().await
    }

    async fn is_available(&self) -> bool {
        self.file_system.exists(&self.paths.packages_dir()).await
    }

    fn get_repository(&self) -> &Repository {
        &self.repository
    }
}
