use heck::{ToSnakeCase, ToUpperCamelCase};
use terraform_schema::provider::{
    attribute, Attribute, Block, BlockSchema, Provider, Schema, Type,
};

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
        .map(ToSnakeCase::to_snake_case)
        .unwrap();
    let structs = generate_constructs(schema);
    format!("pub mod {name} {{\n{structs}\n}}")
}

fn generate_constructs(schema: &Provider) -> String {
    // let mut result = generate_structs_from_block("Provider", &schema.provider.block);
    let mut result = String::new();
    if let Some(resources) = &schema.resource_schemas {
        result += "pub mod resource {\n";
        result += &resources
            .iter()
            .map(schemas_to_constructs)
            .fold(String::new(), |text, (_, content)| text + &content + "\n");
        result += "}\n";
    }
    if let Some(data_sources) = &schema.data_source_schemas {
        result += "pub mod data {\n";
        result += &data_sources
            .iter()
            .map(schemas_to_constructs)
            .fold(String::new(), |text, (_, content)| text + &content + "\n");
        result += "}\n";
    }
    result
}

/// Converts a given [`BlockSchema`] into a rust construct declaration.
fn schemas_to_constructs<'a, 'b>(
    (name, schema): (&'a String, &'b BlockSchema),
) -> (&'a String, String) {
    let st_name = name.to_upper_camel_case();
    let code = format!(
        "::terraform_bindgen_core::codegen::construct! {{\n\tpub {st_name} {}\n}}\n",
        tf_block_to_codegen_type(&schema.block)
    );
    (name, code)
}

fn tf_block_to_codegen_type(block: &Block) -> String {
    let mut result = String::from("{\n");
    if let Some(attributes) = &block.attributes {
        result += &attributes
            .iter()
            .map(attribute_to_param)
            .collect::<String>()
    }
    if let Some(block_types) = &block.block_types {
        result += &block_types
            .iter()
            .map(|(name, schema)| match schema {
                Type::Single { block } => {
                    format!("\t{name}: {{\n{}\n\t}},\n", tf_block_to_codegen_type(block))
                }
                Type::List {
                    block,
                    min_items,
                    max_items,
                } => {
                    let min = min_items.map(|m| m.to_string()).unwrap_or(String::new());
                    let max = max_items.map(|m| m.to_string()).unwrap_or(String::new());
                    format!(
                        "\t{name}: [{min}..{max}] => {},\n",
                        tf_block_to_codegen_type(block)
                    )
                }
            })
            .collect::<String>();
    }
    result + "\n\t}"
}

fn attribute_to_param((name, attribute): (&String, &Attribute)) -> String {
    let desc = match attribute {
        Attribute::Type { description, .. } | Attribute::NestedType { description, .. } => {
            description
                .as_ref()
                .map(|desc| {
                    desc.lines()
                        .map(|line| format!("\t\t/// {line}\n"))
                        .collect::<String>()
                })
                .unwrap_or(String::new())
        }
    };
    let (req, opt, comp) = match attribute {
        Attribute::Type {
            required,
            optional,
            computed,
            ..
        }
        | Attribute::NestedType {
            required,
            optional,
            computed,
            ..
        } => (required, optional, computed),
    };
    let comp = comp.unwrap_or(false);
    let opt = opt.unwrap_or(false);
    let req = req.unwrap_or(comp && !opt);
    let auto = comp.then(|| "auto ").unwrap_or("");
    assert_ne!(opt, req, "Expected opt xor req");
    let ty = match attribute {
        Attribute::Type { r#type, .. } => tf_type_to_codegen_type(r#type),
        Attribute::NestedType { nested_type: _, .. } => todo!(),
    };
    format!("{desc}\t\t{auto}{name}: {ty},\n")
}

fn tf_type_to_codegen_type(ty: &attribute::Type) -> String {
    match ty {
        attribute::Type::String => "::std::string::String".to_string(),
        attribute::Type::Bool => "bool".to_string(),
        attribute::Type::Number => "usize".to_string(),
        attribute::Type::Dynamic => "::terraform_bindgen_core::json::Value".to_string(),
        attribute::Type::Set(ty) => format!("[] => {}", tf_type_to_codegen_type(ty)),
        attribute::Type::Map(ty) => {
            format!("[::std::string::String] => {}", tf_type_to_codegen_type(ty))
        }
        attribute::Type::List(ty) => format!("[..] => {}", tf_type_to_codegen_type(ty)),
        attribute::Type::Object(mapping) => {
            let mut result = "{\n".to_string();
            result += &mapping
                .iter()
                .map(|(key, value)| format!("{key}: {},\n", tf_type_to_codegen_type(value)))
                .collect::<String>();
            result + "}"
        }
    }
}
