use heck::ToUpperCamelCase;
use terraform_schema::provider::attribute::{NestedType, Type};
use terraform_schema::provider::{Attribute, Block};

/// Generate all structs necessary from a given Terraform [`Block`] schema.
pub fn generate_structs_from_block(name: &str, schema: &Block) -> String {
    let structs = tf_block_to_rust_struct(name, schema);

    structs
}

/// Converts a given Block into a rust struct.
fn tf_block_to_rust_struct(name: &str, schema: &Block) -> String {
    let name = name.to_upper_camel_case();
    let mut result = format!("pub struct {name} {{\n");
    if let Some(attrs) = &schema.attributes {
        result += &attrs
            .iter()
            .filter_map(filter_attr_type)
            .filter_map(|(name, _, ty, opt, comp)| if comp { None } else { Some((name, ty, opt)) })
            .map(|(aname, ty, opt)| (aname, to_rust_type(&name, aname, ty), opt))
            .map(|(name, ty, opt)| format!("{name}: {},\n", wrap_optional(ty, opt)))
            .collect::<String>();
        result += &attrs
            .iter()
            .filter_map(filter_attr_nested_type)
            .filter_map(|(name, _, ty, opt, comp)| if comp { None } else { Some((name, ty, opt)) })
            .map(|(aname, ty, opt)| (aname, todo!("{ty:#?}"), opt))
            .map(|(name, ty, opt)| format!("{name}: {},\n", wrap_optional(ty, opt)))
            .collect::<String>();
    }
    if let Some(_blocks) = &schema.block_types {
        todo!()
    }
    result + "}"
}

/// Converts a Terraform type of an attribute to a rust type using `prefix` and `name` to determine
/// sub type names.
fn to_rust_type(prefix: &str, name: &str, ty: &Type) -> String {
    match ty {
        Type::String => "String".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Number => "isize".to_string(),
        Type::Dynamic => todo!(),
        Type::Set(ty) => format!(
            "::std::collections::HashSet<{}>",
            to_rust_type(prefix, name, ty)
        ),
        Type::Map(ty) => format!(
            "::std::collections::HashMap<String, {}>",
            to_rust_type(prefix, name, ty)
        ),
        Type::List(ty) => format!("::std::vec::Vec<{}>", to_rust_type(prefix, name, ty)),
        Type::Object(_) => format!("{prefix}{}", name.to_upper_camel_case()),
    }
}

/// Wrap a type `ty` into an [`Option`] if `opt` is `true`.
fn wrap_optional(ty: String, opt: bool) -> String {
    if opt {
        return format!("::std::option::Option<{ty}>");
    }
    ty
}

/// Used to filter all [`Attribute::NestedTypee`] from a stream of name, attribute pairs. Will also
/// reduce the required and optional parameter to a single optional one.
fn filter_attr_type<'a, 'b>(
    (name, attr): (&'a String, &'b Attribute),
) -> Option<(&'a String, &'b Option<String>, &'b Type, bool, bool)> {
    match attr {
        Attribute::Type {
            r#type,
            description,
            required,
            optional,
            computed,
            ..
        } => {
            let comp = computed.unwrap_or(false);
            let opt = optional.unwrap_or(false);
            let req = required.unwrap_or(comp && !opt);
            assert_ne!(req, opt, "attribute must be optional xor required");
            Some((name, description, r#type, opt, comp))
        }
        Attribute::NestedType { .. } => None,
    }
}

/// Used to filter all [`Attribute::Type`] from a stream of name, attribute pairs.
fn filter_attr_nested_type<'a, 'b>(
    (name, attr): (&'a String, &'b Attribute),
) -> Option<(&'a String, &'b Option<String>, &'b NestedType, bool, bool)> {
    match attr {
        Attribute::NestedType {
            nested_type,
            description,
            required,
            optional,
            computed,
            ..
        } => {
            let comp = computed.unwrap_or(false);
            let opt = optional.unwrap_or(false);
            let req = required.unwrap_or(comp && !opt);
            assert_ne!(req, opt, "attribute must be optional xor required");
            Some((name, description, nested_type, opt, comp))
        }
        Attribute::Type { .. } => None,
    }
}
