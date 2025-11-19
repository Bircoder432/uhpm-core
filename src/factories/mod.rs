// src/factories/mod.rs

mod installation_factory;
mod package_factory;

pub use installation_factory::InstallationFactory;
pub use package_factory::PackageFactory;

/// Collection of factories for creating domain entities.
///
/// Factories encapsulate complex creation logic and ensure
/// entities are always in a valid state.
pub struct Factories {
    package: PackageFactory,
    installation: InstallationFactory,
}

impl Factories {
    /// Creates a new collection of factories.
    pub fn new() -> Self {
        Self {
            package: PackageFactory,
            installation: InstallationFactory,
        }
    }

    /// Returns the package factory.
    pub fn package(&self) -> &PackageFactory {
        &self.package
    }

    /// Returns the installation factory.
    pub fn installation(&self) -> &InstallationFactory {
        &self.installation
    }
}

impl Default for Factories {
    fn default() -> Self {
        Self::new()
    }
}
