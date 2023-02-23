use std::cell::{Ref, RefCell, RefMut};
use std::{collections::HashMap, rc::Rc};

use terraform_schema::document::{Meta, Metadata, Resource};
use terraform_schema::Document;

pub struct App {
    stacks: HashMap<String, RefCell<Document>>,
}

impl App {
    /// Create a new app with `name`.
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
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
        self.stacks.insert(stack_name, RefCell::new(document));
    }

    /// Used to add  a resource to the root document.
    ///
    /// # Parameters
    ///
    /// - `stack_name` Name of the stack deploy in
    /// - `resource_type` Type of the resource object
    /// - `name` Name of the resource object
    /// - `resource` Resource configuration
    ///
    /// # Panics
    ///
    /// Will panic if document of stack `name` was already borrowed.
    pub fn add_resource(
        &self,
        stack_name: impl AsRef<str>,
        resource_type: impl Into<String>,
        name: impl Into<String>,
        resource: Resource,
    ) {
        let stack_name = stack_name.as_ref();
        let mut stack = self
            .stack_mut(stack_name)
            .expect(&format!("no stack with name `{stack_name}`",));
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

    /// Will borrow document of stack `name` and returns `None` if no such stack exists.
    ///
    /// # Panics
    ///
    /// Will panic if the document already is borrowed as a mutable reference.
    pub fn stack(&self, name: impl AsRef<str>) -> Option<Ref<'_, Document>> {
        self.stacks.get(name.as_ref()).map(|inner| inner.borrow())
    }

    /// Will borrow a mutable reference to document of stack `name` and return `None` if no such
    /// stack exists.
    ///
    /// # Panics
    ///
    /// Will panic if the document already is borrowed.
    pub fn stack_mut(&self, name: impl AsRef<str>) -> Option<RefMut<'_, Document>> {
        self.stacks
            .get(name.as_ref())
            .map(|inner| inner.borrow_mut())
    }
}
