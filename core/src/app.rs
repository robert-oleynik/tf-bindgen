use std::{collections::HashMap, rc::Rc};

use terraform_schema::document::{Meta, Metadata, Resource};
use terraform_schema::Document;

pub struct App {
    name: String,
    stacks: HashMap<String, Document>,
}

impl App {
    /// Create a new app with `name`.
    pub fn new(name: String) -> Rc<Self> {
        Rc::new(Self {
            name,
            stacks: HashMap::default(),
        })
    }

    /// Used to create a new stack with name `stack_name`.
    pub fn add_stack(&mut self, stack_name: impl Into<String>) {
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
        self.stacks.insert(stack_name, document);
    }

    /// Used to add  a resource to the root document.
    ///
    /// # Parameters
    ///
    /// - `stack_name` Name of the stack deploy in
    /// - `resource_type` Type of the resource object
    /// - `name` Name of the resource object
    /// - `resource` Resource configuration
    pub fn add_resource(
        &mut self,
        stack_name: impl AsRef<String>,
        resource_type: impl Into<String>,
        name: impl Into<String>,
        resource: Resource,
    ) {
        if let Some(stack) = self.stacks.get_mut(stack_name.as_ref()) {
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
    }
}
