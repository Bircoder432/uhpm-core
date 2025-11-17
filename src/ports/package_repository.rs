use crate::{Dependency, Package, PackageReference, Repository, RepositoryIndex, UhpmError};
use async_trait::async_trait;

#[async_trait]
pub trait PackageRepository: Send + Sync {
    async fn get_package(&self, package_ref: &PackageReference) -> Result<Package, UhpmError>;

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, UhpmError>;

    async fn get_package_versions(&self, package_name: &str) -> Result<Vec<String>, UhpmError>;

    async fn get_latest_version(&self, package_name: &str) -> Result<String, UhpmError>;

    async fn resolve_dependencies(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Package>, UhpmError>;

    async fn download_package(&self, package_ref: &PackageReference) -> Result<Vec<u8>, UhpmError>;

    async fn get_index(&self) -> Result<RepositoryIndex, UhpmError>;

    async fn update_index(&self) -> Result<RepositoryIndex, UhpmError>;

    async fn is_available(&self) -> bool;

    fn get_repository(&self) -> &Repository;
}
