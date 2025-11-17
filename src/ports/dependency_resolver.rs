use crate::{
    Dependency, DependencyConflict, Package, PackageReference, ResolutionResult, UhpmError,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait DependencyResolver: Send + Sync {
    async fn resolve_for_installation(
        &self,
        package_ref: &PackageReference,
        installed_packages: &[Package],
    ) -> Result<ResolutionResult, UhpmError>;

    async fn resolve_for_update(
        &self,
        package_ref: &PackageReference,
        installed_packages: &[Package],
    ) -> Result<ResolutionResult, UhpmError>;

    async fn resolve_for_removal(
        &self,
        package_ref: &PackageReference,
        installed_packages: &[Package],
    ) -> Result<ResolutionResult, UhpmError>;

    async fn check_conflicts(
        &self,
        packages: &[Package],
    ) -> Result<Vec<DependencyConflict>, UhpmError>;

    async fn find_satisfying_versions(
        &self,
        dependency: &Dependency,
    ) -> Result<Vec<Package>, UhpmError>;

    async fn build_dependency_graph(
        &self,
        root_packages: &[PackageReference],
    ) -> Result<HashMap<String, Vec<Dependency>>, UhpmError>;
}
