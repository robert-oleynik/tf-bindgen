use std::cell::RefCell;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, bail, Context, Result};
use terraform_schema::document::{Meta, Metadata, Provider, Resource};
use terraform_schema::Document;

struct Config {
    stacks: HashMap<String, Document>,
}

#[derive(Clone)]
pub struct App {
    config: Rc<RefCell<Config>>,
}

impl App {
    /// Create a new app with `name`.
    pub fn new() -> Self {
        Self {
            config: Rc::new(RefCell::new(Config {
                stacks: HashMap::default(),
            })),
        }
    }

    /// Will deploy the specified infrastructure. Will synthesize deployment before running
    /// `terraform apply`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    ///
    /// - more than one stack was declared (use [`Self::deploy_stack`] instead)
    /// - failed to create deployment directory
    /// - failed to write deployment file
    pub fn deploy(&self) -> Result<()> {
        if self.config.borrow().stacks.len() != 1 {
            bail!(
                "expected 1 stack but got {}",
                self.config.borrow().stacks.len()
            )
        }
        self.deploy_stack(self.config.borrow().stacks.keys().next().unwrap())
    }

    /// Will write deployment of stack with `name` to a file at `path`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if
    /// - failed to find stack
    /// - failed to write file at `path`.
    pub fn synth(&self, name: impl AsRef<str>, path: impl AsRef<Path>) -> Result<()> {
        let document = self
            .config
            .borrow()
            .stacks
            .iter()
            .find(|(stack_name, _)| *stack_name == name.as_ref())
            .map(|(_, document)| serde_json::to_string_pretty(document))
            .ok_or(anyhow!("no stack with name `{}", name.as_ref()))?
            .context("failed to parse stack document")?;
        std::fs::write(path.as_ref(), document).with_context(|| {
            format!(
                "failed to write stack document to file at `{}`",
                path.as_ref().to_str().unwrap()
            )
        })
    }

    /// Will deploy stack with `name` to infrastructure. Will synthesize deployment before running
    /// `terraform apply`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    ///
    /// - more than one stack was declared (use [`Self::deploy_stack`] instead)
    /// - failed to create deployment directory
    /// - failed to write deployment file
    pub fn deploy_stack(&self, name: impl AsRef<str>) -> Result<()> {
        let name = name.as_ref();
        let stack_dir = format!("target/terraform/stacks/{name}");
        std::fs::create_dir_all(&stack_dir).context("failed to create deployment directoy")?;
        let stack_file = format!("{stack_dir}/cdk.tf.json");
        self.synth(name, stack_file)
            .with_context(|| format!("failed synthesize stack with name `{name}`"))?;
        let terraform = Command::new("terraform")
            .arg(format!("-chdir={stack_dir}"))
            .arg("apply")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .context("failed to run terraform")?;
        if !terraform.status.success() {
            bail!(
                "terraform failed with exit code {}",
                terraform.status.code().unwrap_or(-1)
            )
        }
        Ok(())
    }

    /// Used to create a new stack with name `stack_name`.
    pub fn add_stack(&self, stack_name: impl Into<String>) {
        let mut this = self.config.borrow_mut();
        let stack_name = stack_name.into();
        let document = Document {
            meta: Meta {
                metadata: Metadata {
                    backend: String::from("local"),
                    stack_name: stack_name.clone(),
                    version: terraform_schema::VERSION.to_string(),
                },
                outputs: HashMap::default(),
            },
            provider: HashMap::default(),
            resource: HashMap::default(),
        };
        this.stacks.insert(stack_name, document);
    }

    /// Used to add a resource to the root document of the stack with `stack_name`.
    ///
    /// # Parameters
    ///
    /// - `stack_name` Name of the stack to add resource to.
    /// - `resource_type` Type of the resource object
    /// - `name` Name of the resource object
    /// - `resource` Resource configuration
    pub fn add_resource(
        &self,
        stack_name: impl AsRef<str>,
        resource_type: impl Into<String>,
        name: impl Into<String>,
        resource: Resource,
    ) {
        let mut this = self.config.borrow_mut();
        let stack_name = stack_name.as_ref();
        let stack = this
            .stacks
            .get_mut(stack_name)
            .expect(&format!("no stack with name `{stack_name}`"));
        let resource_type = resource_type.into();
        if !stack.resource.contains_key(&resource_type) {
            stack
                .resource
                .insert(resource_type.clone(), HashMap::default());
        }
        stack
            .resource
            .get_mut(&resource_type)
            .unwrap()
            .insert(name.into(), resource);
    }

    /// Used to add a provider to stack's document.
    ///
    /// # Parameters
    ///
    /// - `stack_name` Name of the stack to add provider to.
    /// - `provider_type` Type of the provider to add configuration to.
    /// - `config` provider configuration.
    pub fn add_provider(
        &self,
        stack_name: impl AsRef<str>,
        provider_type: impl Into<String>,
        provider: Provider,
    ) {
        let mut this = self.config.borrow_mut();
        let stack_name = stack_name.as_ref();
        let stack = this
            .stacks
            .get_mut(stack_name)
            .expect(&format!("no stack with name `{stack_name}`"));
        let provider_type = provider_type.into();
        if !stack.provider.contains_key(&provider_type) {
            stack.provider.insert(provider_type.clone(), Vec::new());
        }
        stack
            .provider
            .get_mut(&provider_type)
            .unwrap()
            .push(provider)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}