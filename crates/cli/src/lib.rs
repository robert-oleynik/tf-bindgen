use serde_json::Value;
use std::process::Command;
use tf_bindgen_core::Stack;

const PATH: &str = "target/stacks/";

/// Used to store the result of the terraform command.
pub struct Terraform;

impl Terraform {
    /// Generates the Terraform JSON configuration and writes to a file at
    /// `target/stacks/{stack_name}/cdk.tf.json`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to write JSON document or to create stack directory.
    ///
    /// # Panics
    ///
    /// Will panic if failed to generate JSON document.
    pub fn synth(stack: &Stack) -> std::io::Result<()> {
        let document = stack.to_document();
        let document = serde_json::to_value(&document).expect("valid JSON document");
        let document = match document {
            Value::Object(mut mapping) => {
                let fields = vec!["data", "resource", "provider"];
                for field in fields {
                    if let Some(Value::Object(fields)) = mapping.get(field) {
                        if fields.is_empty() {
                            mapping.remove(field);
                        }
                    }
                }
                Value::Object(mapping)
            }
            _ => unimplemented!(),
        };
        let document = serde_json::to_string_pretty(&document).unwrap();
        let path = format!("{PATH}/{}", stack.name());
        std::fs::create_dir_all(&path)?;
        std::fs::write(format!("{path}/cdk.tf.json"), document)?;
        Ok(())
    }

    /// Will synthesize (see [`Terraform::synth`]). Returns a prepared Terraform command to run
    /// initialization.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to synthesize stack (see [`Terraform::synth`]).
    ///
    /// # Panics
    ///
    /// Will panic if failed to generate document (see [`Terraform::synth`]).
    pub fn init(stack: &Stack) -> std::io::Result<Command> {
        Self::synth(stack)?;
        let mut command = Command::new("terraform");
        let path = format!("{PATH}/{}", stack.name());
        command.arg(format!("-chdir={}", path));
        command.arg("init");
        Ok(command)
    }

    /// Will synthesize (see [`Terraform::synth`]). Returns a prepared Terraform command to run
    /// validate.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to synthesize stack (see [`Terraform::synth`]).
    ///
    /// # Panics
    ///
    /// Will panic if failed to generate document (see [`Terraform::synth`]).
    pub fn validate(stack: &Stack) -> std::io::Result<Command> {
        Self::synth(stack)?;
        let mut command = Command::new("terraform");
        let path = format!("{PATH}/{}", stack.name());
        command.arg(format!("-chdir={}", path));
        command.arg("validate");
        Ok(command)
    }

    /// Will synthesize (see [`Terraform::synth`]). Returns a prepared Terraform command to run
    /// deploy.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to synthesize stack (see [`Terraform::synth`]).
    ///
    /// # Panics
    ///
    /// Will panic if failed to generate document (see [`Terraform::synth`]).
    pub fn apply(stack: &Stack) -> std::io::Result<Command> {
        Self::synth(stack)?;
        let mut command = Command::new("terraform");
        let path = format!("{PATH}/{}", stack.name());
        command.arg(format!("-chdir={}", path));
        command.arg("apply");
        Ok(command)
    }

    /// Will synthesize (see [`Terraform::synth`]). Returns a prepared Terraform command to run
    /// destroy.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to synthesize stack (see [`Terraform::synth`]).
    ///
    /// # Panics
    ///
    /// Will panic if failed to generate document (see [`Terraform::synth`]).
    pub fn destroy(stack: &Stack) -> std::io::Result<Command> {
        Self::synth(stack)?;
        let mut command = Command::new("terraform");
        let path = format!("{PATH}/{}", stack.name());
        command.arg(format!("-chdir={}", path));
        command.arg("destroy");
        Ok(command)
    }

    /// Will synthesize (see [`Terraform::synth`]). Returns a prepared Terraform command to run
    /// plan.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to synthesize stack (see [`Terraform::synth`]).
    ///
    /// # Panics
    ///
    /// Will panic if failed to generate document (see [`Terraform::synth`]).
    pub fn plan(stack: &Stack) -> std::io::Result<Command> {
        Self::synth(stack)?;
        let mut command = Command::new("terraform");
        let path = format!("{PATH}/{}", stack.name());
        command.arg(format!("-chdir={}", path));
        command.arg("plan");
        Ok(command)
    }
}
