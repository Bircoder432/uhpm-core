use crate::{Package, PackageReference, UhpmError, ports::PackageRepository};

pub struct PackageService<LM, RM>
where
    LM: PackageRepository,
    RM: PackageRepository,
{
    local_repo: LM,
    remote_repo: RM,
}

impl<LM, RM> PackageService<LM, RM>
where
    LM: PackageRepository,
    RM: PackageRepository,
{
    pub fn new(local_repo: LM, remote_repo: RM) -> Self {
        Self {
            local_repo,
            remote_repo,
        }
    }

    pub async fn find_best_package(
        &self,
        package_ref: &PackageReference,
    ) -> Result<Package, UhpmError> {
        match self.local_repo.get_package(package_ref).await {
            Ok(package) => Ok(package),
            Err(UhpmError::PackageNotFound(_)) => self.remote_repo.get_package(package_ref).await,
            Err(e) => Err(e),
        }
    }

    pub async fn sync_repositories(&self) -> Result<(), UhpmError> {
        let _local_index = self.local_repo.update_index().await?;
        let _remote_index = self.remote_repo.update_index().await?;

        Ok(())
    }

    pub async fn search_all_packages(&self, query: &str) -> Result<Vec<Package>, UhpmError> {
        let local_results = self.local_repo.search_packages(query).await?;
        let remote_results = self.remote_repo.search_packages(query).await?;

        let mut all_results = local_results;
        all_results.extend(remote_results);

        all_results.sort_by(|a, b| a.name().cmp(b.name()));
        all_results.dedup_by(|a, b| a.id() == b.id());

        Ok(all_results)
    }
}
