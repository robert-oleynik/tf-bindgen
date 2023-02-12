use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod attribute;

pub use attribute::Attribute;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    pub providers: ProviderSchema,
    pub modules: ModuleSchema,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "format_version")]
pub enum ProviderSchema {
    #[serde(rename = "1.0")]
    V1_0 {
        provider_schemas: Option<HashMap<String, Provider>>,
        provider_versions: Option<HashMap<String, String>>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleSchema {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Provider {
    pub provider: BlockSchema,
    pub resource_schemas: Option<HashMap<String, BlockSchema>>,
    pub data_source_schemas: Option<HashMap<String, BlockSchema>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockSchema {
    version: usize,
    block: Block,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    attributes: Option<HashMap<String, Attribute>>,
    block_types: Option<HashMap<String, Type>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "nesting_mode")]
pub enum Type {
    #[serde(rename = "single", alias = "map")]
    Single { block: Box<Block> },
    #[serde(rename = "list", alias = "set")]
    List {
        block: Box<Block>,
        min_items: Option<usize>,
        max_items: Option<usize>,
    },
}
