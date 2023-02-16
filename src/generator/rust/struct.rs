use std::collections::HashMap;

use terraform_schema::provider::attribute::Type;
use terraform_schema::provider::BlockSchema;

use super::field::Field;

pub struct Struct {
    name: String,
    fields: Vec<Field>,
}

impl Struct {
    /// Create a struct from a given [`BlockSchema`].
    pub fn from_schema(name: impl Into<String>, schema: &BlockSchema) -> Self {
        let name = name.into();
        let mut fields = Vec::new();
        if let Some(attributes) = &schema.block.attributes {
            let iter = attributes
                .iter()
                .map(|(name, attribute)| Field::from_attribute(name, attribute));
            fields.extend(iter)
        }
        if let Some(block_types) = &schema.block.block_types {
            todo!("{block_types:#?}")
        }
        Self { name, fields }
    }

    /// Create a struct from a given name type mapping.
    pub fn from_mapping(name: impl Into<String>, mapping: &HashMap<String, Type>) -> Self {
        let name = name.into();
        let fields = mapping
            .iter()
            .map(|(field_name, ty)| Field::from_type(field_name, ty))
            .collect();
        Self { name, fields }
    }

    pub fn to_rust_code(&self) -> String {
        let name = &self.name;
        let fields = self
            .fields
            .iter()
            .filter_map(Field::to_rust_field)
            .fold(String::new(), |mut text, field| {
                text + "\t" + &field + ",\n"
            });
        format!("pub struct {name} {{\n{fields}}}")
    }
}
