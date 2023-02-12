pub mod builder;
pub mod config;
pub mod dependency;
pub mod generator;
pub mod model;
pub mod version;

use std::path::Path;

use model::provider::Schema;

pub use crate::builder::Builder;

pub struct Bindings {
    schema: Schema,
}

impl Bindings {
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        todo!("schema = {:#?}", self.schema)
    }
}
