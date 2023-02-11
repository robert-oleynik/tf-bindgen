use crate::version::*;

pub struct Dependency {
    name: String,
    version: Constraints,
}

impl Dependency {
    /// Converts given dependency, version pair into this parse. Will parse the given version using
    /// [`Version::parse`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to parse version constraints.
    pub fn new(dep: impl Into<String>, ver: impl AsRef<str>) -> Result<Dependency, String> {
        Ok(Self {
            name: dep.into(),
            version: Constraint::parse(ver.as_ref()).map_err(|err| err.to_string())?,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn constraints(&self) -> &Constraints {
        &self.version
    }
}
