use crate::{Dependency, Installation, Package, PackageReference, UhpmConfig, UhpmError};
use async_trait::async_trait;

#[async_trait]
pub trait PackageManager: Send + Sync {
    async fn install(&self, package_ref: &PackageReference) -> Result<Package, UhpmError>;

    async fn uninstall(&self, package_ref: &PackageReference) -> Result<(), UhpmError>;

    async fn update(&self, package_ref: &PackageReference) -> Result<Package, UhpmError>;

    async fn search(&self, query: &str) -> Result<Vec<Package>, UhpmError>;

    async fn info(&self, package_ref: &PackageReference) -> Result<Package, UhpmError>;

    async fn resolve_dependencies(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Package>, UhpmError>;

    async fn list_installed(&self) -> Result<Vec<Package>, UhpmError>;

    async fn check_updates(&self) -> Result<Vec<PackageReference>, UhpmError>;

    async fn activate(&self, package_ref: &PackageReference) -> Result<(), UhpmError>;

    async fn deactivate(&self, package_ref: &PackageReference) -> Result<(), UhpmError>;

    fn get_config(&self) -> &UhpmConfig;

    async fn get_installation(
        &self,
        package_ref: &PackageReference,
    ) -> Result<Option<Installation>, UhpmError>;
}
