use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::dependency::Dependency;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Terraform {
    required_providers: HashMap<String, Provider>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Provider {
    source: String,
    version: String,
}

impl Terraform {
    /// Add provider from specified dependency.
    pub fn add_provider(&mut self, provider: &Dependency) {
        let segments: Vec<_> = provider.name().split("/").collect();
        let name = segments.last().unwrap();
        let provider = Provider {
            source: if segments.len() == 2 {
                provider.name().to_string()
            } else {
                format!("hashicorp/{}", name)
            },
            version: provider
                .constraints()
                .iter()
                .fold(String::new(), |mut out, constraint| {
                    if !out.is_empty() {
                        out = format!("{out}, ")
                    }
                    format!("{out}{constraint}")
                }),
        };
        self.required_providers.insert(name.to_string(), provider);
    }

    /// Returns an iterator other all registered provider.
    pub fn provider(&self) -> impl Iterator<Item = (&String, &Provider)> {
        self.required_providers.iter()
    }
}
