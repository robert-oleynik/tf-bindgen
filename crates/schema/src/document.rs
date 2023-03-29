use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Provider = serde_json::Map<String, serde_json::Value>;

#[derive(Deserialize, Serialize)]
pub struct Document {
    #[serde(rename = "//")]
    pub meta: Meta,
    pub terraform: Terraform,
    pub provider: HashMap<String, Vec<Provider>>,
    pub resource: HashMap<String, HashMap<String, Resource>>,
    pub data: HashMap<String, HashMap<String, Resource>>,
}

#[derive(Deserialize, Serialize)]
pub struct Terraform {
    pub required_providers: HashMap<String, ProviderConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct ProviderConfig {
    pub source: String,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct Meta {
    pub metadata: Metadata,
    pub outputs: HashMap<String, Output>,
}

#[derive(Deserialize, Serialize)]
pub struct Metadata {
    pub backend: String,
    #[serde(rename = "stackName")]
    pub stack_name: String,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct Output {}

#[derive(Deserialize, Serialize)]
pub struct Resource {
    #[serde(rename = "//")]
    pub meta: ResourceMeta,
    #[serde(flatten)]
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct ResourceMeta {
    pub metadata: ResourceMetadata,
}

#[derive(Deserialize, Serialize)]
pub struct ResourceMetadata {
    pub path: String,
    #[serde(rename = "uniqueId")]
    pub unique_id: String,
}
