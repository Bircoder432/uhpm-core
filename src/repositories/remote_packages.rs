use crate::{
    Dependency, Package, PackageReference, Repository, RepositoryIndex, UhpmError,
    paths::UhpmPaths,
    ports::{CacheManager, FileSystemOperations, NetworkOperations, PackageRepository},
};
use async_trait::async_trait;
use semver::Version;

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
        // TODO
        Err(UhpmError::ValidationError("Not implemented".into()))
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
