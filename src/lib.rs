pub mod builder;
pub mod config;
pub mod dependency;
pub mod generator;
pub mod model;
pub mod version;

use std::path::Path;

use terraform_schema::provider;

pub use crate::builder::Builder;

pub struct Bindings {
    schema: provider::Schema,
}

impl Bindings {
    pub fn write_to_file(self, path: impl AsRef<Path>) -> std::io::Result<()> {
        generator::rust::Rust::from_schema(&self.schema).generate(path)
    }
}
