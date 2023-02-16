use std::collections::HashMap;

use heck::ToUpperCamelCase;
use terraform_schema::provider::{attribute::Type, BlockSchema, Provider};

use super::r#struct::Struct;

pub struct Module {
    name: String,
    structs: Vec<Struct>,
}

impl Module {
    /// Parse the given provider `schema` with `name` to a rust namespace.
    pub fn from_schema(name: impl Into<String>, schema: &Provider) -> Self {
        let name = name
            .into()
            .split("/")
            .last()
            .map(ToString::to_string)
            .unwrap();
        let mut structs = vec![Struct::from_schema("Provider", &schema.provider)];
        structs.extend(unwrap_structs("Provider", &schema.provider));
        if let Some(resources) = &schema.resource_schemas {
            let iter = resources.iter().flat_map(|(name, schema)| {
                let name = name.to_upper_camel_case();
                let st = Struct::from_schema(&name, schema);
                let mut structs = unwrap_structs(&name, schema);
                structs.push(st);
                structs
            });
            structs.extend(iter);
        }
        if let Some(data_sources) = &schema.data_source_schemas {
            todo!("{data_sources:#?}")
        }
        Module {
            name: name.into(),
            structs,
        }
    }

    pub fn to_rust_module(&self) -> String {
        let name = &self.name;
        let structs = self
            .structs
            .iter()
            .map(Struct::to_rust_code)
            .fold(String::new(), |text, st| text + &st + "\n");
        format!("pub mod {name} {{\n{structs}}}")
    }
}

fn unwrap_structs_from_mapping(prefix: &str, mapping: &HashMap<String, Type>) -> Vec<Struct> {
    mapping
        .iter()
        .filter_map(|(name, mapping)| match mapping {
            Type::Object(mapping) => {
                let name = format!("{prefix}{}", name.to_upper_camel_case());
                let mut mappings = unwrap_structs_from_mapping(&name, mapping);
                let st = Struct::from_mapping(name, mapping);
                mappings.push(st);
                Some(mappings)
            }
            _ => None,
        })
        .flat_map(|structs| structs)
        .collect()
}

fn unwrap_structs(prefix: &str, schema: &BlockSchema) -> Vec<Struct> {
    let mut structs = Vec::new();
    if let Some(attributes) = &schema.block.attributes {
        let iter = attributes
            .iter()
            .filter_map(|(name, attr)| match attr {
                terraform_schema::provider::Attribute::Type {
                    r#type: Type::Object(mapping),
                    ..
                } => {
                    let name = format!("{prefix}{}", name.to_upper_camel_case());
                    let mut mappings = unwrap_structs_from_mapping(&name, mapping);
                    let st = Struct::from_mapping(name, mapping);
                    mappings.push(st);
                    Some(mappings)
                }
                terraform_schema::provider::Attribute::NestedType { .. } => todo!("{:#?}", attr),
                _ => None,
            })
            .flat_map(|structs| structs);
        structs.extend(iter);
    }
    if let Some(block) = &schema.block.block_types {
        todo!("{block:#?}")
    }
    structs
}
