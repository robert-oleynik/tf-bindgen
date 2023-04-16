pub mod builder;
pub mod codegen;
pub mod config;
pub mod model;
pub mod value;

use std::collections::HashMap;

use codegen::Generator;
use semver::VersionReq;
use tf_bindgen_schema::provider;

pub use tf_bindgen_core::*;

pub use crate::builder::Builder;
pub use serde;
pub use serde_json as json;
pub use tf_bindgen_cli as cli;
pub use tf_bindgen_schema as schema;

pub use value::Value;

pub struct Bindings {
    version: HashMap<String, VersionReq>,
    schema: provider::Schema,
}
use std::path::Path as StdPath;

impl Bindings {
    pub fn write_to_file(
        self,
        base_path: impl AsRef<StdPath>,
        root_file: impl AsRef<StdPath>,
    ) -> std::io::Result<()> {
        let provider_dir = base_path.as_ref().join("provider");
        std::fs::create_dir_all(&provider_dir)?;

        let result = Generator::from_schema(self.schema, self.version);
        let mut root_content = String::new();
        for provider in result.providers {
            let name = &provider.provider.name();
            let name = name.split('/').last().unwrap();
            let provider_dir = provider_dir.join(name);
            let resource_dir = provider_dir.join("resource");
            std::fs::create_dir_all(&resource_dir)?;
            let data_dir = provider_dir.join("data");
            std::fs::create_dir_all(&data_dir)?;

            let resources: String = provider
                .resources
                .iter()
                .map(|construct| {
                    let filename = format!("{}.rs", construct.ty());
                    let path = resource_dir.join(filename);
                    std::fs::write(path, construct.gen_rust())?;
                    Ok(construct.ty())
                })
                .map(|name: std::io::Result<_>| Ok(format!("pub mod {};\n", name?)))
                .collect::<std::io::Result<_>>()?;
            let data_sources: String = provider
                .data_sources
                .iter()
                .map(|construct| {
                    let filename = format!("{}.rs", construct.ty());
                    let path = data_dir.join(filename);
                    std::fs::write(path, construct.gen_rust())?;
                    Ok(construct.ty())
                })
                .map(|name: std::io::Result<_>| Ok(format!("pub mod {};\n", name?)))
                .collect::<std::io::Result<_>>()?;
            let content = provider.provider.gen_rust()
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
