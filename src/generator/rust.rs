use std::path::Path;

mod field;
mod module;
mod r#struct;

use module::Module;
use terraform_schema::provider::Schema;

/// Generator used to generate rust code from Terraform schema.
pub struct Rust {
    modules: Vec<Module>,
}

impl Rust {
    /// Parse rust structures from Terraform [`Schema`].
    pub fn from_schema(schema: &Schema) -> Self {
        let modules = match schema {
            Schema::V1_0 {
                provider_schemas, ..
            } => {
                let mut modules = Vec::new();
                if let Some(schemas) = provider_schemas {
                    let iter = schemas
                        .iter()
                        .map(|(name, provider)| Module::from_schema(name, provider));
                    modules.extend(iter);
                }
                modules
            }
            Schema::Unknown => {
                unimplemented!("Unknown schema format. Only version 1.0 is supported")
            }
        };
        Self { modules }
    }

    pub fn generate(self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let code = self
            .modules
            .iter()
            .map(Module::to_rust_module)
            .fold(String::new(), |text, m| text + &m + "\n");
        std::fs::write(path, code)
    }
}
