use std::collections::HashMap;

use heck::ToUpperCamelCase;
use terraform_schema::provider::attribute::{NestedType, Type};
use terraform_schema::provider::{self, Attribute, Block};

/// Generate all structs necessary from a given Terraform [`Block`] schema.
pub fn generate_structs_from_block(name: &str, schema: &Block) -> String {
    let name = name.to_upper_camel_case();
    let mut result = tf_block_to_rust_struct(&name, schema);

    if let Some(attrs) = &schema.attributes {
        result += &attrs
            .iter()
            .filter_map(filter_attr_type)
            .filter(|(_, _, _, _, comp)| !comp)
            .filter_map(|(name, _, ty, _, _)| Some((name, find_custom_type(ty)?)))
            .map(|(aname, mapping)| generate_structs_from_mapping(&name, aname, mapping))
            .collect::<String>();
        result += &attrs
            .iter()
            .filter_map(filter_attr_nested_type)
            .filter(|(_, _, _, _, comp)| !comp)
            .inspect(|_| todo!())
            .map(|_| String::new()) // TODO
            .collect::<String>();
    }
    if let Some(blocks) = &schema.block_types {
        result += &blocks
            .iter()
            .map(|(block_name, block)| {
                let block_name = block_name.to_upper_camel_case();
                let name = format!("{name}{block_name}");
                match block {
                    provider::Type::Single { block } => generate_structs_from_block(&name, block),
                    provider::Type::List { block, .. } => generate_structs_from_block(&name, block),
                }
            })
            .collect::<String>();
    }

    result
}

pub fn generate_structs_from_mapping(
    prefix: &str,
    name: &str,
    mapping: &HashMap<String, Type>,
) -> String {
    let name = format!("{prefix}{}", name.to_upper_camel_case());
    let mut result = tf_mapping_to_rust_struct(&name, mapping);

    result += &mapping
        .iter()
        .filter_map(|(fname, ty)| Some((fname, find_custom_type(ty)?)))
        .map(|(fname, mapping)| generate_structs_from_mapping(&name, fname, mapping))
        .collect::<String>();

    result
}

/// Converts a given Terrasform [`Block`] into a rust struct.
fn tf_block_to_rust_struct(name: &str, schema: &Block) -> String {
    let name = name.to_upper_camel_case();
    let mut result = format!("pub struct {name} {{\n");
    if let Some(attrs) = &schema.attributes {
        result += &attrs
            .iter()
            .filter_map(filter_attr_type)
            .filter_map(|(name, _, ty, opt, comp)| if comp { None } else { Some((name, ty, opt)) })
            .map(|(aname, ty, opt)| (fix_ident(aname), to_rust_type(&name, aname, ty), opt))
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
    if let Some(blocks) = &schema.block_types {
        result += &blocks
            .iter()
            .map(|(bname, ty)| (fix_ident(bname), bname.to_upper_camel_case(), ty))
            .map(|(n, bname, ty)| match ty {
                provider::Type::Single { .. } => format!("{n}: {name}{bname},\n"),
                provider::Type::List { .. } => {
                    format!("{n}: ::std::vec::Vec<{name}{bname}>,\n")
                }
            })
            .collect::<String>();
    }
    result + "}\n"
}

fn tf_mapping_to_rust_struct(name: &str, mapping: &HashMap<String, Type>) -> String {
    let name = name.to_upper_camel_case();
    let mut result = format!("pub struct {name} {{\n");
    result += &mapping
        .iter()
        .map(|(fname, field)| (fix_ident(fname), to_rust_type(&name, fname, field)))
        .map(|(fname, ty)| format!("{fname}: {ty},\n"))
        .collect::<String>();
    result + "}\n"
}

/// Converts a Terraform type of an attribute to a rust type using `prefix` and `name` to determine
/// sub type names.
fn to_rust_type(prefix: &str, name: &str, ty: &Type) -> String {
    match ty {
        Type::String => "String".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Number => "isize".to_string(),
        Type::Dynamic => "::terraform_bindgen_core::json::Value".to_string(),
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
pub fn filter_attr_type<'a, 'b>(
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

/// Will search type tree until [`Type::Object`] was found. Will return `None` if non such type exists.
pub fn find_custom_type(ty: &Type) -> Option<&HashMap<String, Type>> {
    match ty {
        Type::Set(ty) => find_custom_type(ty),
        Type::Map(ty) => find_custom_type(ty),
        Type::List(ty) => find_custom_type(ty),
        Type::Object(mapping) => Some(mapping),
        _ => None,
    }
}

/// Replace rust keywords with raw names.
pub fn fix_ident(input: &str) -> &str {
    match input {
        "type" => "r#type",
        _ => input,
    }
}
