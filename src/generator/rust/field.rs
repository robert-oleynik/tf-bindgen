use heck::ToUpperCamelCase;
use terraform_schema::provider::{self, attribute::Type, Attribute};

pub struct Field {
    name: String,
    description: Option<String>,
    ty: String,
    required: bool,
    optional: bool,
    computed: bool,
}

pub fn tf_type_to_rs_type(ty_name: &str, ty: &Type) -> String {
    match ty {
        Type::String => "String".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Number => "usize".to_string(),
        Type::Dynamic => "::serde_json::Value".to_string(),
        Type::Set(ty) => format!(
            "::std::collections::HashSet<{}>",
            tf_type_to_rs_type(ty_name, ty)
        ),
        Type::Map(ty) => format!(
            "::std::collections::HashMap<String, {}>",
            tf_type_to_rs_type(ty_name, ty)
        ),
        Type::List(ty) => format!("::std::vec::Vec<{}>", tf_type_to_rs_type(ty_name, ty)),
        Type::Object(_) => format!("{ty_name}"),
    }
}

macro_rules! attr_bool_param {
    ($attr:ident, $id:ident) => {
        match $attr {
            Attribute::Type {
                $id: Some(true), ..
            } => true,
            Attribute::NestedType {
                $id: Some(true), ..
            } => true,
            _ => false,
        }
    };
}

impl Field {
    pub fn from_attribute(prefix: &str, name: impl Into<String>, attribute: &Attribute) -> Self {
        let name = name.into();
        let ty_name = format!("{prefix}{}", name.to_upper_camel_case());
        let ty = match attribute {
            Attribute::Type { r#type, .. } => tf_type_to_rs_type(&ty_name, r#type),
            Attribute::NestedType { .. } => todo!("attribute:#?"),
        };
        let description = match attribute {
            Attribute::Type { description, .. } => description.clone(),
            Attribute::NestedType { description, .. } => description.clone(),
        };
        let required = attr_bool_param!(attribute, required);
        let optional = attr_bool_param!(attribute, optional);
        let computed = attr_bool_param!(attribute, computed);
        Self {
            name: name.into(),
            description,
            ty,
            required,
            optional,
            computed,
        }
    }

    pub fn from_type(prefix: &str, name: impl Into<String>, ty: &Type) -> Self {
        let name = name.into();
        let ty_name = format!("{prefix}{}", name.to_upper_camel_case());
        let ty = tf_type_to_rs_type(&ty_name, ty);
        Self {
            name,
            ty,
            description: None,
            required: true,
            optional: false,
            computed: false,
        }
    }

    pub fn from_block_type(prefix: &str, name: impl Into<String>, ty: &provider::Type) -> Self {
        let name = name.into();
        let type_name = format!("{prefix}{}", name.to_upper_camel_case());
        let ty = match ty {
            provider::Type::Single { .. } => type_name,
            provider::Type::List { .. } => format!("::std::vec::Vec<{type_name}>"),
        };
        Self {
            name,
            ty,
            description: None,
            required: true,
            optional: false,
            computed: false,
        }
    }

    /// Will return `None` if variable is computed.
    pub fn to_rust_field(&self) -> Option<String> {
        let name = &self.name;
        let ty = &self.ty;
        match (self.required, self.optional, self.computed) {
            (_, _, true) => None,
            (true, false, _) => Some(format!("{name}: {ty}")),
            (false, true, _) => Some(format!("{name}: ::std::option::Option<{ty}>")),
            _ => unimplemented!(),
        }
    }
}
