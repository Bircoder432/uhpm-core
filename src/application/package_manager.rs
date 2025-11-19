use crate::{
    Dependency, InstallResult, Installation, Package, PackageReference, RemovalResult,
    SwitchResult, UhpmError,
    factories::{InstallationFactory, PackageFactory},
    ports::{
        CacheManager, EventPublisher, FileSystemOperations, NetworkOperations, PackageRepository,
    },
};
use std::sync::Arc;

/// Main application service that orchestrates package management operations.
///
/// This is the primary entry point for all package management functionality.
/// It coordinates between repositories, services, and factories to perform
/// complex operations like install, remove, and switch.
pub struct PackageManager<FS, NET, REPO, CACHE, EVENTS>
where
    FS: FileSystemOperations,
    NET: NetworkOperations,
    REPO: PackageRepository,
    CACHE: CacheManager,
    EVENTS: EventPublisher,
{
    file_system: Arc<FS>,
    network: Arc<NET>,
    repository: Arc<REPO>,
    cache: Arc<CACHE>,
    event_publisher: Arc<EVENTS>,
}

impl<FS, NET, REPO, CACHE, EVENTS> PackageManager<FS, NET, REPO, CACHE, EVENTS>
where
    FS: FileSystemOperations + Send + Sync,
    NET: NetworkOperations + Send + Sync,
    REPO: PackageRepository + Send + Sync,
    CACHE: CacheManager + Send + Sync,
    EVENTS: EventPublisher + Send + Sync,
{
    pub fn new(
        file_system: FS,
        network: NET,
        repository: REPO,
        cache: CACHE,
        event_publisher: EVENTS,
    ) -> Self {
        Self {
            file_system: Arc::new(file_system),
            network: Arc::new(network),
            repository: Arc::new(repository),
            cache: Arc::new(cache),
            event_publisher: Arc::new(event_publisher),
        }
    }

    pub async fn install(
        &self,
        package_ref: &PackageReference,
    ) -> Result<InstallResult, UhpmError> {
        self.event_publisher
            .publish(crate::PackageEvent::InstallationStarted {
                package_ref: package_ref.clone(),
            })
            .await?;

        let package = self.repository.get_package(package_ref).await?;
        let dependencies = self
            .repository
            .resolve_dependencies(package.dependencies())
            .await?;

        let all_packages = std::iter::once(&package)
            .chain(&dependencies)
            .collect::<Vec<_>>();
        for pkg in all_packages {
            self.download_package_if_needed(pkg).await?;
        }

        let mut installed_files = Vec::new();
        let mut symlinks_created = 0;

        for pkg in dependencies {
            let result = self.install_single_package(&pkg).await?;
            installed_files.extend(result.installed_files);
            symlinks_created += result.symlinks_created;
        }

        let main_result = self.install_single_package(&package).await?;
        installed_files.extend(main_result.installed_files);
        symlinks_created += main_result.symlinks_created;

        let install_result = InstallResult {
            package_id: package.id().clone(),
            installed_files,
            symlinks_created,
        };

        self.event_publisher
            .publish(crate::PackageEvent::InstallationCompleted { package })
            .await?;

        Ok(install_result)
    }

    pub async fn remove(&self, package_ref: &PackageReference) -> Result<RemovalResult, UhpmError> {
        self.event_publisher
            .publish(crate::PackageEvent::RemoveStarted {
                package_ref: package_ref.clone(),
            })
            .await?;

        let package = self.repository.get_package(package_ref).await?;

        if package.is_active() {
            return Err(UhpmError::PackageIsActive);
        }

        let removal_result = self.remove_single_package(&package).await?;

        self.event_publisher
            .publish(crate::PackageEvent::RemoveCompleted {
                package_ref: package_ref.clone(),
            })
            .await?;

        Ok(removal_result)
    }

    pub async fn switch(
        &self,
        package_name: &str,
        target_version: &semver::Version,
    ) -> Result<SwitchResult, UhpmError> {
        let current_ref = PackageReference::new(
            package_name.to_string(),
            self.get_current_version(package_name).await?,
        );

        let target_ref = PackageReference::new(package_name.to_string(), target_version.clone());
        let target_package = self.repository.get_package(&target_ref).await?;

        let removal_result = self.remove(&current_ref).await?;

        let install_result = self.install(&target_ref).await?;

        let switch_result = SwitchResult {
            package_name: package_name.to_string(),
            from_version: Some(current_ref.version),
            to_version: target_version.clone(),
            removed_files: removal_result.removed_files,
            installed_files: install_result.installed_files.len(),
            warnings: Vec::new(),
        };

        Ok(switch_result)
    }

    pub async fn list_installed(&self) -> Result<Vec<Package>, UhpmError> {
        let all_packages = self.repository.search_packages("").await?;
        let installed = all_packages
            .into_iter()
            .filter(|pkg| pkg.is_installed())
            .collect();

        Ok(installed)
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Package>, UhpmError> {
        self.repository.search_packages(query).await
    }

    pub async fn info(&self, package_ref: &PackageReference) -> Result<Package, UhpmError> {
        self.repository.get_package(package_ref).await
    }

    async fn download_package_if_needed(&self, package: &Package) -> Result<(), UhpmError> {
        if self
            .cache
            .has_package(&PackageReference::from_package(package))
            .await
        {
            return Ok(());
        }

        self.event_publisher
            .publish(crate::PackageEvent::DownloadStarted {
                package_ref: PackageReference::from_package(package),
                size: None,
            })
            .await?;

        let package_data = self
            .repository
            .download_package(&PackageReference::from_package(package))
            .await?;

        self.cache
            .put_package(&PackageReference::from_package(package), &package_data)
            .await?;

        self.event_publisher
            .publish(crate::PackageEvent::DownloadCompleted {
                package_ref: PackageReference::from_package(package),
            })
            .await?;

        Ok(())
    }

    async fn install_single_package(&self, package: &Package) -> Result<InstallResult, UhpmError> {
        Ok(InstallResult {
            package_id: package.id().clone(),
            installed_files: Vec::new(),
            symlinks_created: 0,
        })
    }

    async fn remove_single_package(&self, package: &Package) -> Result<RemovalResult, UhpmError> {
        Ok(RemovalResult {
            package_id: package.id().clone(),
            removed_files: 0,
            freed_space: 0,
        })
    }

    async fn get_current_version(&self, package_name: &str) -> Result<semver::Version, UhpmError> {
        let installed = self.list_installed().await?;
        let package = installed
            .into_iter()
            .find(|pkg| pkg.name() == package_name)
            .ok_or_else(|| UhpmError::PackageNotFound(package_name.to_string()))?;

        Ok(package.version().clone())
    }
}
