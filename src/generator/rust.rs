use std::collections::HashMap;

use heck::ToUpperCamelCase;
use terraform_schema::provider::{attribute, Attribute, Block, Provider, Schema, Type};

pub struct GenerationResult {
    pub providers: HashMap<String, ProviderResult>,
}

pub struct ProviderResult {
    pub declaration: String,
    pub data_sources: Vec<ConstructResult>,
    pub resources: Vec<ConstructResult>,
}

pub struct ConstructResult {
    pub name: String,
    pub declaration: String,
}

impl From<&Schema> for GenerationResult {
    fn from(schema: &Schema) -> Self {
        match schema {
            Schema::V1_0 {
                provider_schemas,
                provider_versions: _,
            } => {
                let mut providers = HashMap::new();
                if let Some(schemas) = provider_schemas {
                    let iter = schemas
                        .iter()
                        .map(|(name, schema)| (name.clone(), ProviderResult::from(schema)));
                    providers.extend(iter);
                }
                Self { providers }
            }
            Schema::Unknown => unimplemented!("only schema version 1.0 supported"),
        }
    }
}

impl From<&Provider> for ProviderResult {
    fn from(schema: &Provider) -> Self {
        // TODO: generate provider
        let resources = schema
            .resource_schemas
            .iter()
            .flatten()
            .map(|(name, schema)| (name, &schema.block))
            .map(ConstructResult::from)
            .collect();
        let data_sources = schema
            .data_source_schemas
            .iter()
            .flatten()
            .map(|(name, schema)| (name, &schema.block))
            .map(ConstructResult::from)
            .collect();
        Self {
            declaration: String::new(),
            resources,
            data_sources,
        }
    }
}

impl From<(&String, &Block)> for ConstructResult {
    fn from((name, schema): (&String, &Block)) -> Self {
        Self {
            name: name.clone(),
            declaration: schema_to_construct(name, schema),
        }
    }
}

/// Converts a given [`BlockSchema`] into a rust construct declaration.
fn schema_to_construct(name: &String, schema: &Block) -> String {
    let st_name = name.to_upper_camel_case();
    let codegen = tf_block_to_codegen_type(schema);
    format!("::terraform_bindgen_core::codegen::construct! {{\n\tpub {st_name} {codegen}\n}}\n",)
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
            .map(|(name, schema)| (fix_ident(name), schema))
            .map(|(name, schema)| match schema {
                Type::Single { block } => {
                    format!("\t{name}: {},\n", tf_block_to_codegen_type(block))
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
    let name = fix_ident(name);
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
                .map(|(key, value)| (fix_ident(key), tf_type_to_codegen_type(value)))
                .map(|(key, value)| format!("\t\t{key}: {value},\n"))
                .collect::<String>();
            result + "}"
        }
    }
}

/// Replace rust keywords with raw names.
fn fix_ident(input: &str) -> &str {
    if input.is_empty() {
        panic!("ident: '{input}' is empty")
    }
    match input {
        "type" => "r#type",
        "as" => "r#as",
        "async" => "r#async",
        "await" => "r#await",
        "box" => "r#box",
        "break" => "r#break",
        "const" => "r#const",
        "continue" => "r#continue",
        "dyn" => "r#dyn",
        "else" => "r#else",
        "enum" => "r#enum",
        "extern" => "r#extern",
        "fn" => "r#final",
        "for" => "r#for",
        "if" => "r#if",
        "impl" => "r#impl",
        "in" => "r#in",
        "let" => "r#let",
        "loop" => "r#loop",
        "macro" => "r#macro",
        "match" => "r#match",
        "mod" => "r#mod",
        "move" => "r#move",
        "mut" => "r#mut",
        "pub" => "r#pub",
        "ref" => "r#ref",
        "return" => "r#return",
        "self" => "r#self",
        "static" => "r#static",
        "super" => "r#super",
        "trait" => "r#trait",
        "union" => "r#union",
        "unsafe" => "r#unsafe",
        "use" => "r#use",
        "where" => "r#where",
        "while" => "r#while",
        "yield" => "r#yield",
        _ => input,
    }
}
