use crate::{PackageReference, UhpmError};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

#[async_trait]
pub trait CacheManager: Send + Sync {
    async fn get_package(
        &self,
        package_ref: &PackageReference,
    ) -> Result<Option<Vec<u8>>, UhpmError>;

    async fn put_package(
        &self,
        package_ref: &PackageReference,
        data: &[u8],
    ) -> Result<(), UhpmError>;

    async fn remove_package(&self, package_ref: &PackageReference) -> Result<(), UhpmError>;

    async fn clear_packages(&self) -> Result<(), UhpmError>;

    async fn get_index(&self, repository_url: &str) -> Result<Option<Vec<u8>>, UhpmError>;

    async fn put_index(&self, repository_url: &str, data: &[u8]) -> Result<(), UhpmError>;

    async fn get_cache_size(&self) -> Result<u64, UhpmError>;

    async fn cleanup_old_entries(&self, max_age: Duration) -> Result<(), UhpmError>;

    fn get_cache_path(&self) -> &PathBuf;

    async fn has_package(&self, package_ref: &PackageReference) -> bool;
}
