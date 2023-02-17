mod block;

use heck::ToUpperCamelCase;
use terraform_schema::provider::{Provider, Schema};

use self::block::generate_structs_from_block;

/// Generate rust source code from Terraform provider schema.
pub fn generate_rust_code_from_schema(schema: &Schema) -> String {
    match schema {
        Schema::V1_0 {
            provider_schemas,
            provider_versions: _,
        } => {
            let mut result = String::new();
            if let Some(schemas) = provider_schemas {
                result += &schemas
                    .iter()
                    .map(|(name, schema)| generate_rust_module(name, schema))
                    .fold(String::new(), |text, module| text + &module + "\n")
            }
            result
        }
        Schema::Unknown => unimplemented!("only schema version 1.0 supported"),
    }
}

fn generate_rust_module(name: &str, schema: &Provider) -> String {
    let name = name
        .split("/")
        .last()
        .map(ToUpperCamelCase::to_upper_camel_case)
        .unwrap();
    let structs = generate_structs(schema);
    let impls = generate_impls(schema);
    format!("pub mod {name} {{\n{structs}\n{impls}\n}}")
}

fn generate_structs(schema: &Provider) -> String {
    let mut result = String::new();
    if let Some(resources) = &schema.resource_schemas {
        let res: String = resources
            .iter()
            .map(|(name, schema)| generate_structs_from_block(name, &schema.block))
            .fold(String::new(), |text, st| text + &st + "\n");
        result += &res
    }
    if let Some(_data_sources) = &schema.data_source_schemas {
        todo!()
    }
    result
}

fn generate_impls(_schema: &Provider) -> String {
    // todo!()
    String::new()
}
