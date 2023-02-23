pub mod builder;
pub mod config;
pub mod dependency;
pub mod generator;
pub mod model;
pub mod version;

use std::path::Path;

use generator::rust::GenerationResult;
use terraform_schema::provider;

pub use crate::builder::Builder;

pub struct Bindings {
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
        let result = GenerationResult::from(&self.schema);
        let mut root_content = String::new();
        for (name, provider) in result.providers.iter() {
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
                    let filename = format!("{}.rs", construct.name);
                    let path = resource_dir.join(filename);
                    std::fs::write(path, &construct.declaration)?;
                    Ok(construct.name.clone())
                })
                .map(|name: std::io::Result<_>| Ok(format!("mod {};\n", name?)))
                .collect::<std::io::Result<_>>()?;
            let data_sources: String = provider
                .data_sources
                .iter()
                .map(|construct| {
                    let filename = format!("{}.rs", construct.name);
                    let path = data_dir.join(filename);
                    std::fs::write(path, &construct.declaration)?;
                    Ok(construct.name.clone())
                })
                .map(|name: std::io::Result<_>| Ok(format!("mod {};\n", name?)))
                .collect::<std::io::Result<_>>()?;
            let content = "pub mod resource {\n".to_string()
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
