pub mod builder;
pub mod config;
pub mod version;

use std::path::Path;

pub use crate::builder::Builder;

pub struct Bindings;

impl Bindings {
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        todo!()
    }
}
