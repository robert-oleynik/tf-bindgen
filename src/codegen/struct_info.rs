use heck::ToUpperCamelCase;
use itertools::Itertools;

use super::field_info::FieldInfo;
use super::path::Path;

#[derive(Clone, Debug)]
pub enum StructType {
    Provider {
        ty: String,
        ver: String,
        nested: Vec<StructInfo>,
    },
    Construct {
        ty: String,
        nested: Vec<StructInfo>,
    },
    Nested,
}

#[derive(derive_builder::Builder, Clone, Debug)]
pub struct StructInfo {
    ty: StructType,
    path: Path,
    name: String,
    fields: Vec<FieldInfo>,
}

impl StructInfo {
    pub fn builder() -> StructInfoBuilder {
        StructInfoBuilder::default()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &str {
        match &self.ty {
            StructType::Provider { ty, .. } => &ty,
            StructType::Construct { ty, .. } => &ty,
            StructType::Nested => unimplemented!(),
        }
    }

    pub fn gen_rust(&self) -> String {
        let mut sources = vec![
            self.gen_struct(),
            self.gen_struct_impl(),
            self.gen_builder_struct(),
            self.gen_build_impl(),
            self.gen_builder_setter(),
        ];
        match &self.ty {
            StructType::Provider { nested, .. } | StructType::Construct { nested, .. } => {
                sources.extend(nested.iter().map(|nested| nested.gen_rust()))
            }
            StructType::Nested => {}
        }
        sources.join("\n")
    }

    pub fn gen_struct(&self) -> String {
        let prefix = self.path.type_name();
        let name = self.name.to_upper_camel_case();
        let fields = self
            .fields
            .iter()
            .filter(|field| !field.is_computed() || field.is_optional())
            .map(FieldInfo::gen_field)
            .join(",\n");
        match self.ty {
            StructType::Provider { .. } => format!(
                r#"#[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
				#[serde(crate = "::tf_bindgen::serde")]
				pub struct {prefix}{name} {{
					#[serde(skip_serializing)]
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>, 
					{fields}
				}}"#
            ),
            StructType::Construct { .. } => format!(
                r#"#[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
				#[serde(crate = "::tf_bindgen::serde")]
				pub struct {prefix}{name} {{
					#[serde(skip_serializing)]
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>, 
					#[serde(skip_serializing)]
					__m_name: ::std::string::String,
					{fields}
				}}"#
            ),
            StructType::Nested => format!(
                r#"#[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
				#[serde(crate = "::tf_bindgen::serde")]
				pub struct {prefix}{name} {{
					{fields}
				}}"#
            ),
        }
    }

    pub fn gen_builder_struct(&self) -> String {
        let prefix = self.path.type_name();
        let name = self.name.to_upper_camel_case();
        let fields = self
            .fields
            .iter()
            .filter(|field| !field.is_computed() || field.is_optional())
            .map(FieldInfo::gen_builder_field)
            .join(",\n");
        match self.ty {
            StructType::Provider { .. } => format!(
                r#"pub struct {prefix}{name}Builder {{
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>, 
					{fields}
				}}"#
            ),
            StructType::Construct { .. } => format!(
                r#"pub struct {prefix}{name}Builder {{
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>, 
					__m_name: ::std::string::String,
					{fields}
				}}"#
            ),
            StructType::Nested => format!(
                r#"pub struct {prefix}{name}Builder {{
					{fields}
				}}"#
            ),
        }
    }

    pub fn gen_struct_impl(&self) -> String {
        let prefix = self.path.type_name();
        let name = self.name.to_upper_camel_case();
        let fields = self
            .fields
            .iter()
            .filter(|field| !field.is_computed() || field.is_optional())
            .map(|field| {
                let name = field.name();
                format!("{name}: None")
            })
            .join(",\n");
        match &self.ty {
            StructType::Provider { .. } => format!(
                r#"impl {prefix}{name} {{
					pub fn create<C: ::tf_bindgen::Construct + 'static>(
						scope: &::std::rc::Rc<C>,
					) -> {prefix}{name}Builder {{
						{prefix}{name}Builder {{
							__m_scope: scope.clone(),
							{fields}
						}}
					}}
				}}"#
            ),
            StructType::Construct { .. } => format!(
                r#"impl {prefix}{name} {{
					pub fn create<C: ::tf_bindgen::Construct + 'static>(
						scope: &::std::rc::Rc<C>,
						name: impl ::std::convert::Into<::std::string::String>
					) -> {prefix}{name}Builder {{
						{prefix}{name}Builder {{
							__m_scope: scope.clone(),
							__m_name: name.into(),
							{fields}
						}}
					}}
				}}

				impl ::tf_bindgen::Construct for {prefix}{name} {{
					fn app(&self) -> ::tf_bindgen::app::App {{
						self.__m_scope.app()
					}}

					fn stack(&self) -> &str {{
						self.__m_scope.stack()
					}}

					fn name(&self) -> &str {{
						&self.__m_name
					}}

					fn path(&self) -> ::std::string::String {{
						self.__m_scope.path() + "/" + &self.__m_name
					}}
				}}"#
            ),
            StructType::Nested => format!(
                r#"impl {prefix}{name} {{
					pub fn builder() -> {prefix}{name}Builder {{
						{prefix}{name}Builder {{
							{fields}
						}}
					}}
				}}

				impl From<{prefix}{name}> for ::tf_bindgen::value::Value<{prefix}{name}> {{
					fn from(value: {prefix}{name}) -> Self {{
						Self::Value(value)
					}}
				}}"#
            ),
        }
    }

    pub fn gen_builder_setter(&self) -> String {
        let prefix = self.path.type_name();
        let name = self.name.to_upper_camel_case();
        let setter = self
            .fields
            .iter()
            .filter(|field| !field.is_computed() || field.is_optional())
            .map(|field| {
                let name = field.name();
                let desc = field.doc_str();
                let ty = field.builder_type();
                format!(
                    r#"{desc}
					pub fn {name}(&mut self, value: impl ::std::convert::Into<{ty}>) -> &mut Self {{
						self.{name} = Some(value.into());
						self
					}}"#
                )
            })
            .join("\n");
        format!(
            r#"impl {prefix}{name}Builder {{
				{setter}
			}}"#
        )
    }

    pub fn gen_build_impl(&self) -> String {
        let prefix = self.path.type_name();
        let name = self.name.to_upper_camel_case();
        let assign = self
            .fields
            .iter()
            .filter(|field| !field.is_computed() || field.is_optional())
            .map(|field| {
                let name = field.name();
				let id = field.path_ref();
                if field.is_optional() {
                    format!(r#"{name}: ::tf_bindgen::value::Cell::new("{id}", self.{name}.clone())"#)
                } else {
                    format!(r#"{name}: ::tf_bindgen::value::Cell::new("{id}", self.{name}.clone().expect("field `{name}`"))"#)
                }
            })
            .join(",\n");
        let config: String = self
            .fields
            .iter()
            .filter(|field| !field.is_computed() || field.is_optional())
            .map(|field| {
                let name = field.name();
                format!(
                    r#"
						let value = ::tf_bindgen::json::to_value(&*this.{name}).unwrap();
						config.insert("{name}".to_string(), value);
					"#
                )
            })
            .collect();
        match &self.ty {
            StructType::Provider { ty, ver, .. } => format!(
                r#"impl {prefix}{name}Builder {{
					pub fn build(&mut self) -> {prefix}{name} {{
						let this = {prefix}{name} {{
							__m_scope: self.__m_scope.clone(),
							{assign}
						}};
						let mut config = ::tf_bindgen::json::Map::new();
						{config}
						this.__m_scope
							.app()
							.add_provider(
								this.__m_scope.stack(),
								"{ty}",
								"{ver}",
								config
							);
						this
					}}
				}}"#
            ),
            StructType::Construct { ty, .. } => format!(
                r#"impl {prefix}{name}Builder {{
					pub fn build(&mut self) -> {prefix}{name} {{
						use ::tf_bindgen::Construct;
						let this = {prefix}{name} {{
							__m_scope: self.__m_scope.clone(),
							__m_name: self.__m_name.clone(),
							{assign}
						}};
						let mut config = ::std::collections::HashMap::new();
						{config}
						let resource = ::tf_bindgen::schema::document::Resource {{
							meta: ::tf_bindgen::schema::document::ResourceMeta {{
								metadata: ::tf_bindgen::schema::document::ResourceMetadata {{
									path: this.path(),
									unique_id: this.name().to_string(),
								}},
							}},
							config
						}};
						let app = this.app();
						app.add_resource(this.stack(), "{ty}", this.name(), resource);
						this
					}}
				}}"#
            ),
            StructType::Nested => format!(
                r#"impl {prefix}{name}Builder {{
					pub fn build(&mut self) -> {prefix}{name} {{
						{prefix}{name} {{
							{assign}
						}}
					}}
				}}"#
            ),
        }
    }
}
