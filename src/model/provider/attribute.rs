use std::collections::HashMap;

use serde::de::Unexpected;
use serde::ser::{SerializeMap, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Attribute {
    Type {
        r#type: Type,
        description: Option<String>,
        required: Option<bool>,
        optional: Option<bool>,
        computed: Option<bool>,
        sensitive: Option<bool>,
    },
    NestedType {
        nested_type: NestedType,
        description: Option<String>,
        required: Option<bool>,
        optional: Option<bool>,
        computed: Option<bool>,
        sensitive: Option<bool>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NestedType {
    attributes: HashMap<String, Attribute>,
    nesting_mode: NestedTypeNesting,
}

#[derive(Debug, Clone)]
pub enum Type {
    String,
    Bool,
    Number,
    Dynamic,
    Set(Box<Type>),
    Map(Box<Type>),
    List(Box<Type>),
    Object(HashMap<String, Type>),
}

#[derive(Debug)]
pub enum NestedTypeNesting {
    Invalid,
    Single,
    List,
    Set,
    Map,
}

impl Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Type::String => serializer.serialize_str("string"),
            Type::Bool => serializer.serialize_str("bool"),
            Type::Number => serializer.serialize_str("number"),
            Type::Dynamic => serializer.serialize_str("dynamic"),
            Type::Set(inner) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element("set")?;
                tup.serialize_element(&*inner)?;
                tup.end()
            }
            Type::List(inner) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element("list")?;
                tup.serialize_element(&*inner)?;
                tup.end()
            }
            Type::Map(inner) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element("map")?;
                tup.serialize_element(&*inner)?;
                tup.end()
            }
            Type::Object(fields) => {
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (key, value) in fields {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Type;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"one of `"string"`, `"bool"`, `"number"`, `"dynamic"`, `["set", object]`, `["map", object]`, `["list", object]` or `["object", object]` "#)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match v {
                    "string" => Type::String,
                    "bool" => Type::Bool,
                    "number" => Type::Number,
                    "dynamic" => Type::Dynamic,
                    _ => return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self)),
                })
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
                    let attr: HashMap<String, Type> = seq
                        .next_element()?
                        .ok_or(serde::de::Error::invalid_length(1, &self))?;
                    return Ok(Type::Object(attr));
                }
                let attr: Type = seq
                    .next_element()?
                    .ok_or(serde::de::Error::invalid_length(1, &self))?;
                Ok(match name.as_str() {
                    "set" => Type::Set(Box::new(attr)),
                    "map" => Type::Map(Box::new(attr)),
                    "list" => Type::List(Box::new(attr)),
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
                    _ => return Err(serde::de::Error::invalid_value(Unexpected::Str(&v), &self)),
                })
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
    use super::{NestedTypeNesting, Type};

    #[test]
    fn serialize_attr_type_string() {
        let v = serde_json::json!("string");
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::String = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_bool() {
        let v = serde_json::json!("bool");
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::Bool = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_number() {
        let v = serde_json::json!("number");
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::Number = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_dynamic() {
        let v = serde_json::json!("dynamic");
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::Dynamic = attr {
            return;
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_set() {
        let v = serde_json::json!(["set", "string"]);
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::Set(attr) = attr {
            if let Type::String = *attr {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_map() {
        let v = serde_json::json!(["map", "string"]);
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::Map(attr) = attr {
            if let Type::String = *attr {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_list() {
        let v = serde_json::json!(["list", "string"]);
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::List(attr) = attr {
            if let Type::String = *attr {
                return;
            }
        }
        panic!()
    }

    #[test]
    fn serialize_attr_type_object() {
        let v = serde_json::json!(["object", { "name": "string" }]);
        let attr: Type = serde_json::from_value(v).unwrap();
        if let Type::Object(attr) = attr {
            if let Some(Type::String) = attr.get("name") {
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
