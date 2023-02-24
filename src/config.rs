use anyhow::{Context, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::path::Path;
use toml::{map::Map, Value};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub provider: Map<String, Value>,
}

impl Config {
    /// Load configuration file from file system.
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config = std::fs::read_to_string(path).context("failed to read config file")?;
        toml::from_str(&config).context("failed to parse config file")
    }

    /// Generates a list of `(<provider name>, <version constraint>)` pairs from specified providers.
    ///
    /// # Errors
    ///
    /// Will return `Err` if a provider constraint cannot been parsed.
    pub fn providers(&self) -> Result<Vec<(String, VersionReq)>> {
        self.provider
            .iter()
            .map(|(name, provider)| match provider {
                Value::String(constraint) => {
                    let version = VersionReq::parse(constraint)
                        .context("failed to parse version constraint")?;
                    Ok((name.clone(), version))
                }
                _ => Err(anyhow::anyhow!(
                    "unexpected type of constraint `{name}` (expected: string)"
                )),
            })
            .collect()
    }
}
