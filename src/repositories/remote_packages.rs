use crate::{
    Dependency, DependencyKind, Package, PackageReference, Repository, RepositoryIndex, UhpmError,
    VersionConstraint,
    paths::UhpmPaths,
    ports::{CacheManager, FileSystemOperations, NetworkOperations, PackageRepository},
};
use async_trait::async_trait;
use semver::{Version, VersionReq};
use serde::Deserialize;

pub struct RemotePackagesRepository<NET, CACHE, FS, P>
where
    NET: NetworkOperations,
    CACHE: CacheManager,
    FS: FileSystemOperations,
    P: UhpmPaths,
{
    network: NET,
    cache: CACHE,
    file_system: FS,
    paths: P,
    repository: Repository,
    base_url: String,
}

#[derive(Deserialize)]
struct RemotePackageMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub provides: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
    pub checksum_algorithm: Option<String>,
    pub checksum_hash: Option<String>,
    pub target_os: Option<String>,
    pub target_arch: Option<String>,
}

impl<NET, CACHE, FS, P> RemotePackagesRepository<NET, CACHE, FS, P>
where
    NET: NetworkOperations,
    CACHE: CacheManager,
    FS: FileSystemOperations,
    P: UhpmPaths,
{
    pub fn new(
        network: NET,
        cache: CACHE,
        file_system: FS,
        paths: P,
        repository: Repository,
    ) -> Result<Self, UhpmError> {
        let base_url = match &repository {
            Repository::Http { index_url } => index_url.clone(),
            _ => {
                return Err(UhpmError::ValidationError(
                    "RemotePackagesRepository requires HTTP repository".into(),
                ));
            }
        };

        Ok(Self {
            network,
            cache,
            file_system,
            paths,
            repository,
            base_url,
        })
    }

    fn get_package_meta_url(&self, package_ref: &PackageReference) -> String {
        format!(
            "{}/packages/{}-{}-meta.toml",
            self.base_url.trim_end_matches('/'),
            package_ref.name,
            package_ref.version
        )
    }

    fn get_package_download_url(&self, package_ref: &PackageReference) -> String {
        format!(
            "{}/packages/{}-{}.uhp",
            self.base_url.trim_end_matches('/'),
            package_ref.name,
            package_ref.version
        )
    }

    fn get_index_url(&self) -> String {
        format!("{}/index.toml", self.base_url.trim_end_matches('/'))
    }

    fn parse_dependency(&self, dep_str: &str) -> Result<Dependency, UhpmError> {
        let parts: Vec<&str> = dep_str.splitn(2, '@').collect();
        let name = parts[0].trim().to_string();

        let constraint = if parts.len() == 2 {
            VersionConstraint {
                requirement: VersionReq::parse(parts[1]).map_err(|e| {
                    UhpmError::ValidationError(format!(
                        "Invalid version constraint '{}': {}",
                        parts[1], e
                    ))
                })?,
            }
        } else {
            VersionConstraint {
                requirement: VersionReq::parse("*")
                    .map_err(|e| UhpmError::ValidationError(e.to_string()))?,
            }
        };

        Ok(Dependency {
            name,
            constraint,
            kind: DependencyKind::Required,
            provides: None,
            features: Vec::new(),
        })
    }

    async fn load_remote_meta(
        &self,
        package_ref: &PackageReference,
    ) -> Result<RemotePackageMeta, UhpmError> {
        let meta_url = self.get_package_meta_url(package_ref);
        let meta_data = if let Some(cached) = self.cache.get_index(&meta_url).await? {
            cached
        } else {
            let data = self.network.get(&meta_url).await?;
            self.cache.put_index(&meta_url, &data).await?;
            data
        };

        let meta_str = std::str::from_utf8(&meta_data)
            .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        let remote_meta: RemotePackageMeta =
            toml::from_str(meta_str).map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        Ok(remote_meta)
    }
}

#[async_trait]
impl<NET, CACHE, FS, P> PackageRepository for RemotePackagesRepository<NET, CACHE, FS, P>
where
    NET: NetworkOperations + Send + Sync,
    CACHE: CacheManager + Send + Sync,
    FS: FileSystemOperations + Send + Sync,
    P: UhpmPaths + Send + Sync,
{
    async fn get_package(&self, package_ref: &PackageReference) -> Result<Package, UhpmError> {
        let remote_meta = self.load_remote_meta(package_ref).await?;

        let dependencies: Vec<Dependency> = remote_meta
            .dependencies
            .into_iter()
            .map(|dep_str| self.parse_dependency(&dep_str))
            .collect::<Result<Vec<_>, UhpmError>>()?;

        let package = Package::new(
            remote_meta.name,
            package_ref.version.clone(),
            remote_meta.author,
            crate::PackageSource::Http {
                url: self.get_package_download_url(package_ref),
            },
            crate::Target::current(),
            Some(crate::Checksum {
                algorithm: remote_meta
                    .checksum_algorithm
                    .unwrap_or_else(|| "sha256".to_string()),
                hash: remote_meta.checksum_hash.unwrap_or_default(),
            }),
            dependencies,
        )?;

        Ok(package)
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, UhpmError> {
        let index = self.get_index().await?;
        let mut results = Vec::new();

        for entry in index.packages {
            if entry.name.contains(query) {
                if let Some(latest_version) = entry.versions.last() {
                    let package_ref = PackageReference::new(
                        entry.name.clone(),
                        Version::parse(latest_version)
                            .map_err(|e| UhpmError::ValidationError(e.to_string()))?,
                    );
                    match self.get_package(&package_ref).await {
                        Ok(package) => results.push(package),
                        Err(_) => continue,
                    }
                }
            }
        }

        Ok(results)
    }

    async fn get_package_versions(&self, package_name: &str) -> Result<Vec<String>, UhpmError> {
        let index = self.get_index().await?;
        match index.get_versions(package_name) {
            Some(versions) => Ok(versions.to_vec()),
            None => Err(UhpmError::PackageNotFound(package_name.to_string())),
        }
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
        let index = self.get_index().await?;

        for dependency in dependencies {
            if let Some(version_str) = index.latest_satisfying(dependency) {
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
        if let Some(cached_data) = self.cache.get_package(package_ref).await? {
            return Ok(cached_data);
        }

        let download_url = self.get_package_download_url(package_ref);
        let data = self.network.get(&download_url).await?;

        self.cache.put_package(package_ref, &data).await?;

        Ok(data)
    }

    async fn get_index(&self) -> Result<RepositoryIndex, UhpmError> {
        if let Some(cached_data) = self.cache.get_index(&self.base_url).await? {
            let index_str = std::str::from_utf8(&cached_data)
                .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;
            let index: RepositoryIndex = toml::from_str(index_str)
                .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;
            return Ok(index);
        }

        let index_url = self.get_index_url();
        let data = self.network.get(&index_url).await?;
        let index_str = std::str::from_utf8(&data)
            .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        let index: RepositoryIndex = toml::from_str(index_str)
            .map_err(|e| UhpmError::DeserializationError(e.to_string()))?;

        self.cache.put_index(&self.base_url, &data).await?;

        Ok(index)
    }

    async fn update_index(&self) -> Result<RepositoryIndex, UhpmError> {
        self.cache.put_index(&self.base_url, &[]).await?;
        self.get_index().await
    }

    async fn is_available(&self) -> bool {
        match self.network.head(&self.get_index_url()).await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn get_repository(&self) -> &Repository {
        &self.repository
    }
}
