use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::Command;

use crate::dependency::Dependency;
use crate::model::provider::ProviderSchema;
use crate::model::{self, Document};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to prepare terraform directory")]
    FailedTerraformDir(std::io::Error),
    #[error("failed to create Terraform main file (reason: {0})")]
    FailedCreateMain(std::io::Error),
    #[error("failed to initialize Terraform")]
    FailedInitTerraform(std::io::Error),
    #[error("failed to parse Terraform schema")]
    InvalidSchema(#[from] serde_json::Error),
}

/// Generates Terraform projects for given providers.
#[derive(Default)]
pub struct Generator {
    providers: Vec<Dependency>,
}

impl Generator {
    /// Add a list of specified providers.
    pub fn providers(mut self, providers: Vec<Dependency>) -> Self {
        self.providers.extend(providers.into_iter());
        self
    }

    /// Generate Terraform project.
    pub fn generate(self, out_dir: impl AsRef<Path>) -> Result<ProviderSchema, Error> {
        let terraform_dir = out_dir.as_ref().join("terraform");
        std::fs::create_dir_all(&terraform_dir).map_err(Error::FailedTerraformDir)?;
        let main_file = terraform_dir.join("main.tf.json");

        let mut config = model::config::Terraform::default();
        for provider in &self.providers {
            config.add_provider(provider)
        }
        let document = Document::from_config(config);

        let file = File::create(main_file).map_err(Error::FailedCreateMain)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &document).unwrap();
        writer.flush().map_err(Error::FailedCreateMain)?;

        let tf_process = Command::new("terraform")
            .arg(format!("-chdir={}", terraform_dir.to_str().unwrap()))
            .arg("init")
            .output()
            .map_err(Error::FailedInitTerraform)?;
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
            .map_err(Error::FailedInitTerraform)?;
        if !tf_process.status.success() {
            print!("{}", String::from_utf8(tf_process.stdout).unwrap());
            print!("{}", String::from_utf8(tf_process.stderr).unwrap());
            panic!("failed to read Terraform provider information")
        }
        Ok(serde_json::from_slice(&tf_process.stdout[..])?)
    }
}
