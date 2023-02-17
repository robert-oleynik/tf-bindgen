pub mod builder;
pub mod config;
pub mod dependency;
pub mod generator;
pub mod model;
pub mod version;

use std::path::Path;

use generator::rust::generate_rust_code_from_schema;
use terraform_schema::provider;

pub use crate::builder::Builder;

pub struct Bindings {
    schema: provider::Schema,
}

impl Bindings {
    pub fn write_to_file(self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let content = generate_rust_code_from_schema(&self.schema);
        std::fs::write(path, content)
    }
}
