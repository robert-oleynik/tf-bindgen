use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tf_bindgen_schema::document::{Meta, Metadata, ProviderConfig, Terraform};
use tf_bindgen_schema::Document;

use crate::{L1Construct, Path, Provider, Scope};

/// Used to store and manage all resources and data sources associated with an infrastructure
/// deployment.
///
/// Will use a mutable container to store resources and data sources.
#[derive(Clone)]
pub struct Stack {
    inner: Rc<InnerStack>,
}

struct InnerStack {
    name: String,
    provider: RefCell<Vec<Rc<dyn Provider>>>,
    resources: RefCell<Vec<Rc<dyn L1Construct>>>,
    data_sources: RefCell<Vec<Rc<dyn L1Construct>>>,
}

impl Stack {
    pub fn new(name: impl Into<String>) -> Rc<Self> {
        Rc::new(Self {
            inner: Rc::new(InnerStack {
                name: name.into(),
                provider: RefCell::new(Vec::new()),
                resources: RefCell::new(Vec::new()),
                data_sources: RefCell::new(Vec::new()),
            }),
        })
    }

    /// Name of the stack.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Add `provider` to this stack.
    pub fn add_provider(&self, provider: Rc<dyn Provider>) {
        self.inner.provider.borrow_mut().push(provider)
    }

    /// Add `resource` to this stack.
    pub fn add_resource(&self, resource: Rc<dyn L1Construct>) {
        self.inner.resources.borrow_mut().push(resource)
    }

    /// Add a data source to the data source store.
    pub fn add_data_source(&self, data_source: Rc<dyn L1Construct>) {
        self.inner.data_sources.borrow_mut().push(data_source)
    }

    /// Generate Terraform JSON configuration out of stored provider, resources and data sources.
    pub fn to_document(&self) -> Document {
        let mut document = Document {
            meta: Meta {
                metadata: Metadata {
                    backend: "local".to_string(),
                    stack_name: self.name().to_string(),
                    version: tf_bindgen_schema::VERSION.to_string(),
                },
                outputs: HashMap::default(),
            },
            terraform: Terraform {
                required_providers: HashMap::default(),
            },
            provider: HashMap::default(),
            resource: HashMap::default(),
            data: HashMap::default(),
        };
        for provider in self.inner.provider.borrow().iter() {
            let path = provider.path();
            let name = path.name();
            let local_name = name.split('/').last().unwrap();
            let (version, schema) = provider.to_schema();
            let config = ProviderConfig {
                source: name.to_string(),
                version,
            };
            document
                .terraform
                .required_providers
                .insert(local_name.to_string(), config);
            if !document.provider.borrow().contains_key(local_name) {
                document
                    .provider
                    .borrow_mut()
                    .insert(local_name.to_string(), Vec::new());
            }
            document
                .provider
                .borrow_mut()
                .get_mut(local_name)
                .unwrap()
                .push(schema);
        }
        for resource in self.inner.resources.borrow().iter() {
            let path = resource.path();
            let key = path.id();
            let (ty, schema) = resource.to_schema();
            if !document.resource.contains_key(&ty) {
                document.resource.insert(ty.clone(), HashMap::new());
            }
            if document
                .resource
                .get_mut(&ty)
                .unwrap()
                .insert(key, schema)
                .is_some()
            {
                panic!("resource '{path}' already exists");
            }
        }
        for data_source in self.inner.data_sources.borrow().iter() {
            let path = data_source.path();
            let key = path.id();
            let (ty, schema) = data_source.to_schema();
            if !document.data.contains_key(&ty) {
                document.data.insert(ty.clone(), HashMap::new());
            }
            if document
                .data
                .get_mut(&ty)
                .unwrap()
                .insert(key, schema)
                .is_some()
            {
                panic!("data source '{path}' already exists");
            }
        }
        document
    }
}

impl Scope for Stack {
    /// Returns a copy of it self.
    fn stack(&self) -> Stack {
        self.clone()
    }

    fn path(&self) -> crate::Path {
        Path::from(self.name().to_string())
    }
}
