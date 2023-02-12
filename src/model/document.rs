use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::config::Terraform;

#[derive(Debug, Deserialize, Serialize)]
pub struct Empty {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Document {
    terraform: Terraform,
    provider: HashMap<String, Empty>,
}

impl Document {
    pub fn from_config(config: Terraform) -> Self {
        let provider = config
            .provider()
            .map(|(name, _)| (name.clone(), Empty {}))
            .collect();
        Self {
            terraform: config,
            provider,
        }
    }
}
