pub mod app;
pub mod builder;
pub mod codegen;
pub mod config;
mod construct;
pub mod generator;
pub mod model;
pub mod stack;

use std::collections::HashMap;
use std::path::Path;

use codegen::generator::Generator;
use semver::VersionReq;
use tf_schema::provider;

pub use crate::builder::Builder;
pub use construct::Construct;
pub use serde;
pub use serde_json as json;
pub use tf_schema as schema;

pub struct Bindings {
    version: HashMap<String, VersionReq>,
    schema: provider::Schema,
}

impl Bindings {
    pub fn write_to_file(
        self,
        base_path: impl AsRef<Path>,
        root_file: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        let provider_dir = base_path.as_ref().join("provider");
        std::fs::create_dir_all(&provider_dir)?;

        let result = Generator::from((&self.schema, &self.version));
        let mut root_content = String::new();
        for provider in result.provider() {
            let name = &provider.provider_type;
            let name = name.split('/').last().unwrap();
            let provider_dir = provider_dir.join(name);
            let resource_dir = provider_dir.join("resource");
            std::fs::create_dir_all(&resource_dir)?;
            let data_dir = provider_dir.join("data");
            std::fs::create_dir_all(&data_dir)?;

            let resources: String = provider
                .resource_constructs
                .iter()
                .map(|construct| {
                    let filename = format!("{}.rs", construct.resource_type);
                    let path = resource_dir.join(filename);
                    std::fs::write(path, &construct.as_binding())?;
                    Ok(construct.resource_type.clone())
                })
                .map(|name: std::io::Result<_>| Ok(format!("pub mod {};\n", name?)))
                .collect::<std::io::Result<_>>()?;
            let data_sources: String = provider
                .data_source_constructs
                .iter()
                .map(|construct| {
                    let filename = format!("{}.rs", construct.resource_type);
                    let path = data_dir.join(filename);
                    std::fs::write(path, &construct.as_binding())?;
                    Ok(construct.resource_type.clone())
                })
                .map(|name: std::io::Result<_>| Ok(format!("pub mod {};\n", name?)))
                .collect::<std::io::Result<_>>()?;
            let content = provider.as_binding()
                + "pub mod resource {\n"
                + &resources
                + "}\npub mod data {\n"
                + &data_sources
                + "}";

            let provider_file = provider_dir.join("mod.rs");
            std::fs::write(provider_file, content)?;
            root_content += &format!(
                "#[path = \"{}/mod.rs\"]\npub mod {name};\n",
                provider_dir.to_str().unwrap()
            )
        }
        let root_file = base_path.as_ref().join(root_file);
        std::fs::write(root_file, root_content)
    }
}
