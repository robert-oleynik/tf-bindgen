use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::generator::schema::Generator;
use crate::Bindings;

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
    /// kubernetes = "2.17.0"
    /// ```
    pub fn config(&mut self, path: impl Into<String>) -> &mut Self {
        self.config_path = Some(path.into());
        self
    }

    /// Read configuration file and generate rust files from terraform providers.
    pub fn generate(&mut self) -> Result<Bindings> {
        let config_path = self
            .config_path
            .take()
            .ok_or(anyhow!("missing config path"))?;
        let cfg = Config::from_file(&config_path)
            .with_context(|| format!("failed to read config from file {config_path}"))?;
        let providers = cfg.providers().context("failed to parse providers")?;

        let schema = Generator::default()
            .providers(providers)
            .generate(std::env::var("OUT_DIR").unwrap())
            .context("failed to generate rust code from schema")?;

        Ok(Bindings { schema })
    }
}
