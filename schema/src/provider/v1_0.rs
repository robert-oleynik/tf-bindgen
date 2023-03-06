use std::collections::HashMap;

use serde::de::Unexpected;
use serde::ser::{SerializeMap, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize)]
pub struct Provider {
    pub provider: Schema,
    pub resource_schemas: HashMap<String, Schema>,
    pub data_source_schemas: HashMap<String, Schema>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    pub version: i64,
    pub block: Block,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    pub attributes: Option<HashMap<String, Attribute>>,
    pub block_types: Option<HashMap<String, Type>>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Attribute {
    pub r#type: BlockType,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub optional: Option<bool>,
    pub computed: Option<bool>,
    pub sensitive: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NestedType {
    pub attributes: HashMap<String, Attribute>,
    pub nesting_mode: NestedTypeNesting,
}

#[derive(Debug, Clone)]
pub enum BlockType {
    String,
    Bool,
    Number,
    Dynamic,
    Set(Box<BlockType>),
    Map(Box<BlockType>),
    List(Box<BlockType>),
    Object(HashMap<String, BlockType>),
}

#[derive(Debug)]
pub enum NestedTypeNesting {
    Invalid,
    Single,
    List,
    Set,
    Map,
}

impl Serialize for BlockType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BlockType::String => serializer.serialize_str("string"),
            BlockType::Bool => serializer.serialize_str("bool"),
            BlockType::Number => serializer.serialize_str("number"),
            BlockType::Dynamic => serializer.serialize_str("dynamic"),
            BlockType::Set(inner) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element("set")?;
                tup.serialize_element(inner)?;
                tup.end()
            }
            BlockType::List(inner) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element("list")?;
                tup.serialize_element(inner)?;
                tup.end()
            }
            BlockType::Map(inner) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element("map")?;
                tup.serialize_element(inner)?;
                tup.end()
            }
            BlockType::Object(fields) => {
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (key, value) in fields {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for BlockType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = BlockType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"one of `"string"`, `"bool"`, `"number"`, `"dynamic"`, `["set", object]`, `["map", object]`, `["list", object]` or `["object", object]` "#)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match v {
                    "string" => BlockType::String,
                    "bool" => BlockType::Bool,
                    "number" => BlockType::Number,
                    "dynamic" => BlockType::Dynamic,
                    _ => return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self)),
                })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_borrowed_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_borrowed_str(&v)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or(serde::de::Error::invalid_length(0, &self))?;
                if name == "object" {
                    let attr: HashMap<String, BlockType> = seq
                        .next_element()?
                        .ok_or(serde::de::Error::invalid_length(1, &self))?;
                    return Ok(BlockType::Object(attr));
                }
                let attr: BlockType = seq
                    .next_element()?
                    .ok_or(serde::de::Error::invalid_length(1, &self))?;
                Ok(match name.as_str() {
                    "set" => BlockType::Set(Box::new(attr)),
                    "map" => BlockType::Map(Box::new(attr)),
                    "list" => BlockType::List(Box::new(attr)),
                    _ => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Str(&name),
                            &self,
                        ))
                    }
                })
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

impl Serialize for NestedTypeNesting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NestedTypeNesting::Invalid => serializer.serialize_str("invalid"),
            NestedTypeNesting::Single => serializer.serialize_str("single"),
            NestedTypeNesting::List => serializer.serialize_str("list"),
            NestedTypeNesting::Set => serializer.serialize_str("set"),
            NestedTypeNesting::Map => serializer.serialize_str("map"),
        }
    }
}

impl<'de> Deserialize<'de> for NestedTypeNesting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = NestedTypeNesting;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str(r#"one of `"invalid"`, `"single"`, `"list"`, `"set"` and `"map"`"#)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match v {
                    "invalid" => NestedTypeNesting::Invalid,
                    "single" => NestedTypeNesting::Single,
                    "list" => NestedTypeNesting::List,
                    "set" => NestedTypeNesting::Set,
                    "map" => NestedTypeNesting::Map,
                    _ => return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self)),
                })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_borrowed_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_borrowed_str(&v)
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockType, NestedTypeNesting};

    #[test]
    fn serialize_attr_type_string() {
        let v = serde_json::json!("string");
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::String = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_bool() {
        let v = serde_json::json!("bool");
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::Bool = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_number() {
        let v = serde_json::json!("number");
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::Number = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_dynamic() {
        let v = serde_json::json!("dynamic");
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::Dynamic = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_set() {
        let v = serde_json::json!(["set", "string"]);
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::Set(attr) = attr {
            if let BlockType::String = *attr {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_map() {
        let v = serde_json::json!(["map", "string"]);
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::Map(attr) = attr {
            if let BlockType::String = *attr {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_list() {
        let v = serde_json::json!(["list", "string"]);
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::List(attr) = attr {
            if let BlockType::String = *attr {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_object() {
        let v = serde_json::json!(["object", { "name": "string" }]);
        let attr: BlockType = serde_json::from_value(v).unwrap();
        if let BlockType::Object(attr) = attr {
            if let Some(BlockType::String) = attr.get("name") {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_nested_type_invalid() {
        let v = serde_json::json!("invalid");
        let attr: NestedTypeNesting = serde_json::from_value(v).unwrap();
        if let NestedTypeNesting::Invalid = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_nested_type_list() {
        let v = serde_json::json!("list");
        let attr: NestedTypeNesting = serde_json::from_value(v).unwrap();
        if let NestedTypeNesting::List = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_nested_type_set() {
        let v = serde_json::json!("set");
        let attr: NestedTypeNesting = serde_json::from_value(v).unwrap();
        if let NestedTypeNesting::Set = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_nested_type_map() {
        let v = serde_json::json!("map");
        let attr: NestedTypeNesting = serde_json::from_value(v).unwrap();
        if let NestedTypeNesting::Map = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_nested_type_single() {
        let v = serde_json::json!("single");
        let attr = serde_json::from_value(v).unwrap();
        if let NestedTypeNesting::Single = attr {
            return;
        }
        panic!()
    }
}
