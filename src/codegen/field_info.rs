use itertools::Itertools;

use super::{
    path::Path,
    type_info::{TypeInfo, Wrapper},
};

#[derive(derive_builder::Builder, Clone, Debug)]
pub struct FieldInfo {
    path: Path,
    name: String,
    type_info: TypeInfo,
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
        if self.is_computed() && !self.is_optional() {
            let type_name = self.type_info.source();
            return format!(
                "#[serde(skip_serializing)] pub {name}: ::tf_bindgen::value::Cell<::tf_bindgen::value::Computed<{type_name}>>"
            );
        }
        let type_name = self.field_type();
        format!("pub {name}: {type_name}")
    }

    pub fn gen_builder_field(&self) -> String {
        let name = self.name();
        let type_name = self.type_info.source();
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
        let type_name = self.type_info.source();
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
        format!("::tf_bindgen::value::Cell<{type_name}>")
    }

    /// Generated the builder's setter function.
    pub fn builder_setter_impl(&self) -> String {
        let name = self.name();
        let fn_name = match name {
            "build" => "build_",
            _ => name,
        };
        let type_name = self.type_info.type_name();
        let convert = match self.type_info.wrapper() {
            Wrapper::List => "into_value_list()",
            Wrapper::Map => "into_value_map()",
            Wrapper::Type => "into_value()",
            Wrapper::Set => "into_value_set()",
        };
        let impl_type = match self.type_info.wrapper() {
            Wrapper::List => "IntoValueList",
            Wrapper::Map => "IntoValueMap",
            Wrapper::Type => "IntoValue",
            Wrapper::Set => "IntoValueSet",
        };
        format!(
            r#"pub fn {fn_name}(&mut self, value: impl ::tf_bindgen::value::{impl_type}<{type_name}>) -> &mut Self {{
				self.{name} = Some(value.{convert});
				self
			}}"#
        )
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
