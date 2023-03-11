use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::config::Config;
use crate::model::config::Terraform;
use crate::model::Document;
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
        let version = providers.iter().cloned().collect();

        let terraform_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("terraform");
        std::fs::create_dir_all(&terraform_dir).context("failed to create terraform directory")?;
        let main_file = terraform_dir.join("main.tf.json");

        let mut config = Terraform::default();
        for (name, constraint) in &providers {
            config.add_provider(name, constraint.clone())
        }
        let document = Document::from_config(config);

        let file = File::create(main_file).context("failed to write main bindings file")?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &document).unwrap();
        writer
            .flush()
            .context("failed to write to terraform provider document")?;

        let tf_process = Command::new("terraform")
            .arg(format!("-chdir={}", terraform_dir.to_str().unwrap()))
            .arg("init")
            .output()
            .context("failed to initialize terraform provider")?;
        if !tf_process.status.success() {
            print!("{}", String::from_utf8(tf_process.stdout).unwrap());
            print!("{}", String::from_utf8(tf_process.stderr).unwrap());
            panic!("failed to initialize Terraform")
        }

        let tf_process = Command::new("terraform")
            .arg(format!("-chdir={}", terraform_dir.to_str().unwrap()))
            .arg("providers")
            .arg("schema")
            .arg("-json")
            .output()
            .context("failed to read terraform provider schemas")?;
        if !tf_process.status.success() {
            print!("{}", String::from_utf8(tf_process.stdout).unwrap());
            print!("{}", String::from_utf8(tf_process.stderr).unwrap());
            bail!("failed to read terraform provider schema")
        }
        let schema = serde_json::from_slice(&tf_process.stdout[..])
            .context("failed to parse provider schema")?;

        Ok(Bindings { schema, version })
    }
}
