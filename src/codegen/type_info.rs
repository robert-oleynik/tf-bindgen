use heck::ToUpperCamelCase;
use tf_bindgen_schema::provider::v1_0::BlockType;

use super::path::Path;

#[derive(Debug, Clone)]
pub enum Wrapper {
    List,
    Map,
    Type,
    Set,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    wrapper: Wrapper,
    name: String,
}

impl TypeInfo {
    pub fn from_schema(path: &Path, name: &str, schema: &BlockType) -> Self {
        let composed_type = path.type_name() + &name.to_upper_camel_case();
        let type_name = match schema {
            BlockType::Set(inner) | BlockType::Map(inner) | BlockType::List(inner) => match **inner
            {
                BlockType::Set(_) | BlockType::Map(_) | BlockType::List(_) => {
                    unimplemented!("double nested types are not supported")
                }
                BlockType::Object(_) => &composed_type,
                _ => base_type_to_string(inner),
            },
            BlockType::Object(_) => &composed_type,
            _ => base_type_to_string(schema),
        };
        let wrapper = match schema {
            BlockType::Set(_) => Wrapper::Set,
            BlockType::Map(_) => Wrapper::Map,
            BlockType::List(_) => Wrapper::List,
            _ => Wrapper::Type,
        };
        Self {
            wrapper,
            name: type_name.to_string(),
        }
    }

    pub fn new(wrapper: Wrapper, name: impl Into<String>) -> Self {
        Self {
            wrapper,
            name: name.into(),
        }
    }

    pub fn wrapper(&self) -> &Wrapper {
        &self.wrapper
    }

    /// Returns the unwrapped type.
    pub fn type_name(&self) -> &str {
        &self.name
    }

    /// Returns the composed type.
    pub fn source(&self) -> String {
        let type_name = &self.name;
        let type_name = format!("::tf_bindgen::Value<{type_name}>");
        match self.wrapper {
            Wrapper::List => format!("::std::vec::Vec<{type_name}>"),
            Wrapper::Map => {
                format!("::std::collections::HashMap<::std::string::String, {type_name}>")
            }
            Wrapper::Type => type_name.to_string(),
            Wrapper::Set => format!("::std::collections::HashSet<{type_name}>"),
        }
    }
}

fn base_type_to_string(schema: &BlockType) -> &str {
    match schema {
        BlockType::String => "::std::string::String",
        BlockType::Bool => "bool",
        BlockType::Number => "i64",
        BlockType::Dynamic => "::tf_bindgen::json::Value",
        _ => unimplemented!(),
    }
}
