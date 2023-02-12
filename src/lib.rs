pub mod builder;
pub mod config;
pub mod dependency;
pub mod generator;
pub mod model;
pub mod version;

use std::path::Path;

pub use crate::builder::Builder;

pub struct Bindings {
    document: model::Document,
}

impl Bindings {
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        todo!("{:#?}", self.document)
    }
}
