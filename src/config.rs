use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] toml::de::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Deserialize, Serialize)]
pub struct TerraformConfig {
    pub providers: Vec<String>,
    pub modules: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub terraform: TerraformConfig,
}

impl Config {
    /// Load configuration file from file system.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }
}
