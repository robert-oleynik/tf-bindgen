use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod attribute;

pub use attribute::Attribute;

#[derive(Deserialize, Serialize)]
pub struct Schema {
    pub providers: ProviderSchema,
    pub modules: ModuleSchema,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "format_version")]
pub enum ProviderSchema {
    #[serde(rename = "0.2")]
    V0_2 {
        provider_schemas: HashMap<String, Provider>,
        provider_version: HashMap<String, String>,
    },
}

#[derive(Deserialize, Serialize)]
pub struct ModuleSchema {}

#[derive(Deserialize, Serialize)]
pub struct Provider {
    pub provider: BlockSchema,
    pub resource_schemas: HashMap<String, BlockSchema>,
    pub data_source_schemas: HashMap<String, BlockSchema>,
}

#[derive(Deserialize, Serialize)]
pub struct BlockSchema {
    version: usize,
    block: Block,
}

#[derive(Deserialize, Serialize)]
pub struct Block {
    attributes: HashMap<String, Attribute>,
    block_types: HashMap<String, Type>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "nested_mode")]
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
