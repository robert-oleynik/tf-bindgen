use itertools::Itertools;

use super::path::Path;

#[derive(derive_builder::Builder, Clone, Debug)]
pub struct FieldInfo {
    path: Path,
    name: String,
    type_name: String,
    description: Option<String>,
    optional: bool,
    computed: bool,
}

impl FieldInfo {
    pub fn builder() -> FieldInfoBuilder {
        FieldInfoBuilder::default()
    }

    pub fn gen_field(&self) -> String {
        let name = self.name();
        let type_name = self.field_type();
        format!(
            r#"#[serde(serialize_with = "::tf_bindgen::value::serialize_rc_cell")] pub {name}: {type_name}"#
        )
    }

    pub fn gen_builder_field(&self) -> String {
        let name = self.name();
        let type_name = self.builder_type();
        format!("{name}: ::std::option::Option<{type_name}>")
    }

    pub fn name(&self) -> &str {
        fix_ident(&self.name)
    }

    /// Returns `true` if this field is optional.
    pub fn is_optional(&self) -> bool {
        self.optional
    }

    /// Returns `true` if this field can be calculated by Terraform.
    pub fn is_computed(&self) -> bool {
        self.computed
    }

    /// Name of the reference used by terraform.
    pub fn path_ref(&self) -> String {
        self.path
            .segments()
            .chain(Some(&self.name).into_iter())
            .join(".")
    }

    fn ty(&self) -> String {
        let type_name = &self.type_name;
        if self.is_optional() {
            format!("::std::option::Option<{type_name}>")
        } else {
            type_name.to_string()
        }
    }

    /// Type of field used inside of resources and nested types. Will be wrapped inside of
    /// [`std::rc::Rc`] and [`crate::value::Cell`].
    pub fn field_type(&self) -> String {
        let type_name = self.ty();
        format!("::std::rc::Rc<::tf_bindgen::value::Cell<{type_name}>>")
    }

    /// Type of field used inside of a builder. Will be wrapped inside of [`crate::value::Value`].
    pub fn builder_type(&self) -> String {
        let type_name = self.ty();
        format!("::tf_bindgen::value::Value<{type_name}>")
    }

    /// Generate doc comment for field builder. Will be empty if node description was specified.
    pub fn doc_str(&self) -> String {
        self.description
            .iter()
            .flat_map(|desc| desc.split("\n"))
            .map(|desc| "///".to_owned() + desc)
            .join("\n")
    }
}

/// Replace rust keywords with raw names.
fn fix_ident(input: &str) -> &str {
    assert!(!input.is_empty(), "ident: '{input}' is empty");
    match input {
        "type" => "r#type",
        "as" => "r#as",
        "async" => "r#async",
        "await" => "r#await",
        "box" => "r#box",
        "break" => "r#break",
        "const" => "r#const",
        "continue" => "r#continue",
        "dyn" => "r#dyn",
        "else" => "r#else",
        "enum" => "r#enum",
        "extern" => "r#extern",
        "fn" => "r#final",
        "for" => "r#for",
        "if" => "r#if",
        "impl" => "r#impl",
        "in" => "r#in",
        "let" => "r#let",
        "loop" => "r#loop",
        "macro" => "r#macro",
        "match" => "r#match",
        "mod" => "r#mod",
        "move" => "r#move",
        "mut" => "r#mut",
        "pub" => "r#pub",
        "ref" => "r#ref",
        "return" => "r#return",
        "self" => "r#self",
        "static" => "r#static",
        "super" => "r#super",
        "trait" => "r#trait",
        "union" => "r#union",
        "unsafe" => "r#unsafe",
        "use" => "r#use",
        "where" => "r#where",
        "while" => "r#while",
        "yield" => "r#yield",
        _ => input,
    }
}
