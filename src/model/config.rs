use std::collections::HashMap;

use semver::{Comparator, Op, VersionReq};
use serde::{Deserialize, Serialize};

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
    pub fn add_provider(&mut self, provider_name: &str, constraint: VersionReq) {
        let segments: Vec<_> = provider_name.split('/').collect();
        let name = segments.last().unwrap();
        let provider = Provider {
            source: if segments.len() == 2 {
                provider_name.to_string()
            } else {
                format!("hashicorp/{}", name)
            },
            version: constraint
                .comparators
                .into_iter()
                .map(cargo_simplify_version)
                .fold(String::from(">=0.0.0"), |text, constraint| {
                    text + "," + &constraint
                }),
        };
        self.required_providers.insert(name.to_string(), provider);
    }

    /// Returns an iterator other all registered provider.
    pub fn provider(&self) -> impl Iterator<Item = (&String, &Provider)> {
        self.required_providers.iter()
    }
}

fn cargo_simplify_version(constraint: Comparator) -> String {
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
