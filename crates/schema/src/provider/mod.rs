use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod v1_0;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "format_version")]
pub enum Schema {
    #[serde(rename = "1.0")]
    V1_0 {
        provider_schemas: HashMap<String, v1_0::Provider>,
    },
    #[serde(other)]
    Unknown,
}
