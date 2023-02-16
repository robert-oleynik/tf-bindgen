use std::collections::HashMap;

use terraform_schema::provider::attribute::Type;
use terraform_schema::provider::Block;

use super::field::Field;

pub struct Struct {
    name: String,
    fields: Vec<Field>,
}

impl Struct {
    /// Create a struct from a given [`BlockSchema`].
    pub fn from_schema(name: impl Into<String>, schema: &Block) -> Self {
        let name = name.into();
        let mut fields = Vec::new();
        if let Some(attributes) = &schema.attributes {
            let iter = attributes
                .iter()
                .map(|(attr_name, attribute)| Field::from_attribute(&name, attr_name, attribute));
            fields.extend(iter)
        }
        if let Some(block_types) = &schema.block_types {
            let iter = block_types
                .iter()
                .map(|(block_name, ty)| Field::from_block_type(&name, block_name, ty));
            fields.extend(iter)
        }
        Self { name, fields }
    }

    /// Create a struct from a given name type mapping.
    pub fn from_mapping(name: impl Into<String>, mapping: &HashMap<String, Type>) -> Self {
        let name = name.into();
        let fields = mapping
            .iter()
            .map(|(field_name, ty)| Field::from_type(&name, field_name, ty))
            .collect();
        Self { name, fields }
    }

    pub fn to_rust_code(&self) -> String {
        let name = &self.name;
        let fields = self
            .fields
            .iter()
            .filter_map(Field::to_rust_field)
            .fold(String::new(), |text, field| text + "\t" + &field + ",\n");
        format!("pub struct {name} {{\n{fields}}}")
    }
}
