use std::collections::HashMap;

use heck::ToUpperCamelCase;
use semver::{Comparator, Op, VersionReq};
use tf_schema::provider::{
    v1_0::{Attribute, Block, BlockType, Provider, Type},
    Schema,
};

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

impl GenerationResult {
    pub fn new(schema: &Schema, version: &HashMap<String, VersionReq>) -> Self {
        match schema {
            Schema::V1_0 { provider_schemas } => {
                let mut providers = HashMap::new();
                let iter = provider_schemas.iter().map(|(name, schema)| {
                    let n = name.split('/').last().unwrap();
                    let version = version
                        .get(n)
                        .unwrap()
                        .comparators
                        .iter()
                        .cloned()
                        .map(cargo_simplify_version)
                        .fold(String::from(">=0.0.0"), |text, c| text + "," + &c);
                    (name.clone(), ProviderResult::new(name, &version, schema))
                });
                providers.extend(iter);
                Self { providers }
            }
            Schema::Unknown => unimplemented!("only schema version 1.0 supported"),
        }
    }
}

impl ProviderResult {
    pub fn new(name: &str, version: &str, schema: &Provider) -> Self {
        let declaration = provider_to_construct(name, version, &schema.provider.block);
        let resources = schema
            .resource_schemas
            .iter()
            .map(|(name, schema)| (name, &schema.block))
            .map(ConstructResult::from)
            .collect();
        let data_sources = schema
            .data_source_schemas
            .iter()
            .map(|(name, schema)| (name, &schema.block))
            .map(ConstructResult::from)
            .collect();
        Self {
            declaration,
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

/// Converts a given [`Block`] into a rust construct declaration.
fn schema_to_construct(name: &String, schema: &Block) -> String {
    let st_name = name.to_upper_camel_case();
    let codegen = tf_block_to_codegen_type(schema);
    format!("::tf_bindgen::codegen::construct! {{\n\tpub {st_name} {codegen}\n}}\n",)
}

/// Converts a given [`Block`] into a rust provider declaration.
fn provider_to_construct(name: &str, version: &str, schema: &Block) -> String {
    let codegen = tf_block_to_codegen_type(schema);
    format!("::tf_bindgen::codegen::provider! {{\n\t\"{name}\":\"{version}\",\n\tpub Provider {codegen}\n}}\n")
}

fn tf_block_to_codegen_type(block: &Block) -> String {
    let mut result = String::from("{\n");
    result += &block
        .attributes
        .iter()
        .map(attribute_to_param)
        .collect::<String>();
    result += &block
        .block_types
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
    result + "\n\t}"
}

fn attribute_to_param((name, attribute): (&String, &Attribute)) -> String {
    let name = fix_ident(name);
    let desc = match &attribute.description {
        Some(desc) => desc.lines().map(|line| format!("/// {line}")).collect(),
        None => String::new(),
    };
    let comp = attribute.computed.unwrap_or(false);
    let opt = attribute.optional.unwrap_or(false);
    let req = attribute.required.unwrap_or(comp && !opt);
    let auto = match comp {
        true => "@auto ",
        false => "",
    };
    let opt_str = match opt {
        true => "@opt ",
        false => "",
    };
    assert_ne!(opt, req, "Expected opt xor req");
    let ty = tf_type_to_codegen_type(&attribute.r#type);
    format!("{desc}\t\t{auto}{opt_str}{name}: {ty},\n")
}

fn tf_type_to_codegen_type(ty: &BlockType) -> String {
    match ty {
        BlockType::String => "::std::string::String".to_string(),
        BlockType::Bool => "bool".to_string(),
        BlockType::Number => "i32".to_string(),
        BlockType::Dynamic => "::tf_bindgen::json::Value".to_string(),
        BlockType::Set(ty) => format!("[] => {}", tf_type_to_codegen_type(ty)),
        BlockType::Map(ty) => {
            format!("[::std::string::String] => {}", tf_type_to_codegen_type(ty))
        }
        BlockType::List(ty) => format!("[..] => {}", tf_type_to_codegen_type(ty)),
        BlockType::Object(mapping) => {
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
    assert!(!input.is_empty(), "ident: '{input}' is empty");
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

pub fn cargo_simplify_version(constraint: Comparator) -> String {
    let major = constraint.major;
    let minor = constraint.minor.unwrap_or(0);
    let patch = constraint.patch.unwrap_or(0);
    assert!(
        constraint.pre.is_empty(),
        "pre release constraints are not supported"
    );
    match constraint.op {
        Op::Tilde if constraint.minor.is_some() => {
            format!(">={major}{minor}{patch},<{major}{}.0", minor + 1)
        }
        Op::Caret if major == 0 && constraint.minor.is_none() => ">=0.0.0,<1.0.0".to_string(),
        Op::Caret if major == 0 && minor == 0 && constraint.patch.is_some() => {
            format!(">=0.0.{patch},<0.0.{}", patch + 1)
        }
        Op::Caret if major == 0 => {
            format!(">=0.{minor}.0,<0.{}.0", minor + 1)
        }
        Op::Wildcard if constraint.minor.is_some() => {
            format!(">={major}.{minor}.0,<{major}.{}.0", minor + 1)
        }
        Op::Tilde | Op::Caret | Op::Wildcard => {
            format!(">={major}.{minor}.{patch},<{}.0.0", major + 1)
        }
        Op::Exact => format!("={major}.{minor}.{patch}"),
        Op::Greater => format!(">{major}.{minor}.{patch}"),
        Op::GreaterEq => format!(">={major}.{minor}.{patch}"),
        Op::Less => format!("<{major}.{minor}.{patch}"),
        Op::LessEq => format!("<={major}.{minor}.{patch}"),
        _ => unimplemented!(),
    }
}
