use crate::{config::Config, Bindings};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Missing config file")]
    MissingConfigFile,
    #[error("Failed to parse config {1}. Reason: {0}")]
    Config(crate::config::Error, String),
}

#[derive(Default)]
pub struct Builder {
    config_path: Option<String>,
}

impl Builder {
    /// Set path of configuration file to use.
    ///
    /// # Config
    ///
    /// ```toml
    /// [terraform]
    /// providers = [
    ///		"kubernetes@2.17.0"
    /// ]
    /// ```
    pub fn config(&mut self, path: impl Into<String>) -> &mut Self {
        self.config_path = Some(path.into());
        self
    }

    /// Read configuration file and generate rust files from terraform providers.
    pub fn generate(&mut self) -> Result<Bindings, Error> {
        let config_path = self.config_path.take().ok_or(Error::MissingConfigFile)?;
        let cfg = Config::from_file(&config_path).map_err(|err| Error::Config(err, config_path))?;

        todo!()
    }
}