use std::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

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
