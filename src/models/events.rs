use crate::{Package, PackageReference};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageEvent {
    InstallationStarted {
        package_ref: PackageReference,
    },

    InstallationCompleted {
        package: Package,
    },

    InstallationFailed {
        package_ref: PackageReference,
        error: String,
    },

    UninstallationStarted {
        package_ref: PackageReference,
    },

    UninstallationCompleted {
        package_ref: PackageReference,
    },

    UpdateStarted {
        package_ref: PackageReference,
    },

    UpdateCompleted {
        package: Package,
    },

    DownloadStarted {
        package_ref: PackageReference,
        size: Option<u64>,
    },

    DownloadProgress {
        package_ref: PackageReference,
        downloaded: u64,
        total: u64,
    },

    DownloadCompleted {
        package_ref: PackageReference,
    },

    DependencyResolved {
        dependency: String,
        package: Package,
    },
}
