use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use tf_schema::provider;

use crate::model::{self, Document};

/// Generates Terraform projects for given providers.
#[derive(Default)]
pub struct Generator {
    providers: Vec<(String, semver::VersionReq)>,
}

impl Generator {
    /// Add a list of specified providers.
    pub fn providers(mut self, providers: Vec<(String, semver::VersionReq)>) -> Self {
        self.providers.extend(providers.into_iter());
        self
    }

    /// Generate Terraform project.
    pub fn generate(self, out_dir: impl AsRef<Path>) -> Result<provider::Schema> {
        let terraform_dir = out_dir.as_ref().join("terraform");
        std::fs::create_dir_all(&terraform_dir).context("failed to create terraform directory")?;
        let main_file = terraform_dir.join("main.tf.json");

        let mut config = model::config::Terraform::default();
        for (name, constraint) in &self.providers {
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
        serde_json::from_slice(&tf_process.stdout[..]).context("failed to parse provider schema")
    }
}
