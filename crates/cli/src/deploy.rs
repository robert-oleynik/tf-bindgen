use std::process::{Command, Stdio};

use crate::Terraform;

/// A builder used to generate Terraform validation command.
pub struct Deploy {
    cmd: String,
    working_dir: Option<String>,
}

impl Deploy {
    /// Set the path to the Terraform executable. Default is `"terraform"`.
    pub fn cmd(&mut self, cmd: impl Into<String>) -> &mut Self {
        self.cmd = cmd.into();
        self
    }

    /// Set the working directory of Terraform.
    pub fn working_dir(&mut self, dir: impl Into<String>) -> &mut Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Build and run the terraform command. Returns the gathered Terraform resource.
    pub fn run(&mut self) -> std::io::Result<Terraform> {
        let mut terraform = Command::new(&self.cmd);
        if let Some(dir) = &self.working_dir {
            terraform.arg(format!("-chdir={dir}"));
        }
        let output = terraform
            .arg("deploy")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        Ok(Terraform { output })
    }
}

impl Default for Deploy {
    fn default() -> Self {
        Self {
            cmd: String::from("terraform"),
            working_dir: None,
        }
    }
}