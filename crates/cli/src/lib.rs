use std::process::Output;

pub mod deploy;
pub mod init;
pub mod validate;

/// Used to store the result of the terraform command.
pub struct Terraform {
    output: Output,
}

impl Terraform {
    /// Creates a new builder for Terraform's validation.
    pub fn validate() -> validate::Validate {
        validate::Validate::default()
    }

    /// Creates a new builder for Terraform's init.
    pub fn init() -> init::Init {
        init::Init::default()
    }

    /// Creates a new builder for Terraform's deploy.
    pub fn deploy() -> deploy::Deploy {
        deploy::Deploy::default()
    }

    /// Returns `true` if Terraform completed successfully.
    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    /// Returns the stdout of Terraform.
    pub fn stdout(&self) -> &[u8] {
        &self.output.stdout
    }

    /// Returns the stderr of Terraform.
    pub fn stderr(&self) -> &[u8] {
        &self.output.stderr
    }
}
