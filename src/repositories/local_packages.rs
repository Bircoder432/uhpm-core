use crate::{
    Dependency, DependencyKind, Package, PackageReference, Repository, RepositoryIndex, UhpmError,
    VersionConstraint,
    factories::PackageFactory,
    paths::UhpmPaths,
    ports::{FileSystemOperations, PackageRepository},
};
use async_trait::async_trait;
use semver::{Version, VersionReq};
use std::path::PathBuf;

#[derive(Clone)]
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

    fn parse_dependency(&self, dep_str: &str) -> Result<Dependency, UhpmError> {
        if let Some((name, version)) = dep_str.split_once('@') {
            let constraint = VersionConstraint {
                requirement: VersionReq::parse(version).map_err(|e| {
                    UhpmError::ValidationError(format!(
                        "Invalid version constraint '{}': {}",
                        version, e
                    ))
                })?,
            };

            Ok(Dependency {
                name: name.trim().to_string(),
                constraint,
                kind: DependencyKind::Required,
                provides: None,
                features: Vec::new(),
            })
        } else {
            let constraint = VersionConstraint {
                requirement: VersionReq::parse("*")
                    .map_err(|e| UhpmError::ValidationError(e.to_string()))?,
            };

            Ok(Dependency {
                name: dep_str.trim().to_string(),
                constraint,
                kind: DependencyKind::Required,
                provides: None,
                features: Vec::new(),
            })
        }
    }

    async fn add_directory_to_tar(
        &self,
        tar: &mut tar::Builder<flate2::write::GzEncoder<&mut Vec<u8>>>,
        base_path: &PathBuf,
        current_path: &PathBuf,
    ) -> Result<(), UhpmError> {
        if let Ok(entries) = self.file_system.read_dir(current_path).await {
            for entry in entries {
                let metadata = self.file_system.metadata(&entry).await?;

                if metadata.is_directory() {
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

        let data = self.file_system.read_file(&meta_path).await?;
        let meta_str = std::str::from_utf8(&data)
            .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        let meta: crate::repositories::package_files::PackageMeta =
            toml::from_str(meta_str).map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        let dependencies: Vec<Dependency> = meta
            .dependencies
            .into_iter()
            .map(|dep_str| self.parse_dependency(&dep_str))
            .collect::<Result<Vec<_>, UhpmError>>()?;

        let package = PackageFactory::create(
            meta.name,
            package_ref.version.clone(),
            meta.author,
            crate::PackageSource::Local {
                path: self
                    .paths
                    .packages_dir()
                    .join(&package_ref.name)
                    .join(&package_ref.version.to_string()),
            },
            crate::Target::current(),
            None,
            dependencies,
        )?;

        Ok(package)
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, UhpmError> {
        let packages_dir = self.paths.packages_dir();
        let mut results = Vec::new();

        if self.file_system.exists(&packages_dir).await {
            if let Ok(entries) = self.file_system.read_dir(&packages_dir).await {
                for package_dir in entries {
                    if let Some(package_name) = package_dir.file_name().and_then(|n| n.to_str()) {
                        if package_name.contains(query) {
                            let versions = self.get_package_versions(package_name).await?;

                            for version_str in versions {
                                if let Ok(version) = Version::parse(&version_str) {
                                    let package_ref =
                                        PackageReference::new(package_name.to_string(), version);
                                    match self.get_package(&package_ref).await {
                                        Ok(package) => results.push(package),
                                        Err(_) => continue,
                                    }
                                }
                            }
                        }
                    }
                }
            }
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
            } else {
                return Err(UhpmError::ResolutionError(format!(
                    "Cannot resolve dependency: {} {}",
                    dependency.name, dependency.constraint.requirement
                )));
            }
        }

        Ok(resolved_packages)
    }

    async fn download_package(&self, package_ref: &PackageReference) -> Result<Vec<u8>, UhpmError> {
        let meta_path = self.get_package_meta_path(package_ref);
        if !self.file_system.exists(&meta_path).await {
            return Err(UhpmError::PackageNotFound(package_ref.to_string()));
        }

        let package_files_repo = crate::repositories::package_files::PackageFilesRepository::new(
            self.file_system.clone(),
            self.paths.packages_dir(),
        );

        package_files_repo
            .create_package_archive(&crate::PackageId::new(
                &package_ref.name,
                &package_ref.version,
            ))
            .await
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
