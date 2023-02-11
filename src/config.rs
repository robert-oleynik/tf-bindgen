use serde::{Deserialize, Serialize};
use std::path::Path;
use toml::{map::Map, Value};

use crate::dependency::Dependency;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] toml::de::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub provider: Map<String, Value>,
}

impl Config {
    /// Load configuration file from file system.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }

    /// Generates a list of `(<provider name>, <version constraint>)` pairs from specified providers.
    ///
    /// # Errors
    ///
    /// Will return `Err` if a provider constraint cannot been parsed.
    pub fn providers(&self) -> Result<Vec<Dependency>, String> {
        self.provider
            .iter()
            .map(|(name, provider)| match provider {
                Value::String(constraint) => Dependency::new(name.clone(), constraint),
                _ => todo!("handle none string provider versions"),
            })
            .collect()
    }
}
