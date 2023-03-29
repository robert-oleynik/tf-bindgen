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
        let fields = self.fields.iter().map(FieldInfo::gen_field).join(",\n");
        match self.ty {
            StructType::Provider { .. } => format!(
                r#"#[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
				#[serde(crate = "::tf_bindgen::serde")]
				pub struct {prefix}{name} {{
					#[serde(skip_serializing)]
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Scope>, 
					{fields}
				}}"#
            ),
            StructType::Construct { .. } => format!(
                r#"#[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
				#[serde(crate = "::tf_bindgen::serde")]
				pub struct {prefix}{name} {{
					#[serde(skip_serializing)]
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Scope>, 
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
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Scope>, 
					{fields}
				}}"#
            ),
            StructType::Construct { .. } => format!(
                r#"pub struct {prefix}{name}Builder {{
					__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Scope>, 
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
        let prepare_fields = self
            .fields
            .iter()
            .map(|field| {
                let name = field.name();
                format!(
                    r#"{name}: {{
						let path = format!("{{prefix}}.{name}");
						self.{name}.prepare(path)
					}}"#
                )
            })
            .join(",\n");
        match &self.ty {
            StructType::Provider { ty, .. } => format!(
                r#"impl {prefix}{name} {{
					pub fn create<C: ::tf_bindgen::Scope + 'static>(
						scope: &::std::rc::Rc<C>,
					) -> {prefix}{name}Builder {{
						{prefix}{name}Builder {{
							__m_scope: scope.clone(),
							{fields}
						}}
					}}

				}}

				impl ::tf_bindgen::Scope for {prefix}{name} {{
					fn stack(&self) -> ::tf_bindgen::Stack {{
						self.__m_scope.stack()
					}}

					fn path(&self) -> ::tf_bindgen::Path {{
						let mut path = self.__m_scope.path();
						path.push("{ty}");
						path
					}}
				}}"#
            ),
            StructType::Construct { .. } => format!(
                r#"impl {prefix}{name} {{
					pub fn create<C: ::tf_bindgen::Scope + 'static>(
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

				impl ::tf_bindgen::Scope for {prefix}{name} {{
					fn stack(&self) -> ::tf_bindgen::Stack {{
						self.__m_scope.stack()
					}}

					fn path(&self) -> ::tf_bindgen::Path {{
						let mut path = self.__m_scope.path();
						path.push(&self.__m_name);
						path
					}}
				}}
 
				impl ::tf_bindgen::value::Prepare for {prefix}{name} {{
					fn prepare(self, prefix: impl Into<::std::string::String>) -> Self {{
						let prefix = prefix.into();
						Self {{
							__m_scope: self.__m_scope,
							__m_name: self.__m_name,
							{prepare_fields}
						}}
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

				impl ::tf_bindgen::value::Prepare for {prefix}{name} {{
					fn prepare(self, prefix: impl Into<::std::string::String>) -> Self {{
						let prefix = prefix.into();
						Self {{
							{prepare_fields}
						}}
					}}
				}}

				impl From<{prefix}{name}> for ::tf_bindgen::value::Value<{prefix}{name}> {{
					fn from(value: {prefix}{name}) -> Self {{
						Self::Value {{ value: ::std::rc::Rc::new(value) }}
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
                let fn_name = if name == "build" { "build_" } else { name };
                let desc = field.doc_str();
                let ty = field.builder_type();
                format!(
                    r#"{desc}
					pub fn {fn_name}(&mut self, value: impl ::std::convert::Into<{ty}>) -> &mut Self {{
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
            // .filter(|field| !field.is_computed() || field.is_optional())
            .map(|field| {
                let name = field.name();
                if field.is_optional() {
                    format!(r#"{name}: ::tf_bindgen::value::Cell::new("{name}", self.{name}.clone())"#)
                } else if field.is_computed() {
					format!(r#"{name}: ::tf_bindgen::value::Cell::new("{name}", ::tf_bindgen::value::Value::Computed)"#)
				} else {
                    format!(r#"{name}: ::tf_bindgen::value::Cell::new("{name}", self.{name}.clone().expect("field `{name}`"))"#)
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
						let value = ::tf_bindgen::json::to_value(&self.{name}).unwrap();
						config.insert("{name}".to_string(), value);
					"#
                )
            })
            .collect();
        match &self.ty {
            StructType::Provider { ver, .. } => format!(
                r#"impl ::tf_bindgen::Provider for {prefix}{name} {{
					fn to_schema(&self) -> (::std::string::String, ::tf_bindgen::schema::document::Provider) {{
						let mut config = ::tf_bindgen::schema::document::Provider::new();
						{config}
						("{ver}".to_string(), config)
					}}
				}}

				impl {prefix}{name}Builder {{
					pub fn build(&mut self) -> ::std::rc::Rc<{prefix}{name}> {{
						use tf_bindgen::Scope;
						let this = ::std::rc::Rc::new({prefix}{name} {{
							__m_scope: self.__m_scope.clone(),
							{assign}
						}});
						this.stack().add_provider(this.clone());
						this
					}}
				}}"#
            ),
            StructType::Construct { ty, .. } => format!(
                r#"impl ::tf_bindgen::L1Construct for {prefix}{name} {{
					fn to_schema(&self) -> (::std::string::String, ::tf_bindgen::schema::document::Resource) {{
						use tf_bindgen::Scope;
						let mut config = ::std::collections::HashMap::new();
						{config}
						let path = self.path();
						let resource = ::tf_bindgen::schema::document::Resource {{
							meta: ::tf_bindgen::schema::document::ResourceMeta {{
								metadata: ::tf_bindgen::schema::document::ResourceMetadata {{
									path: path.to_string(),
									unique_id: path.name().to_string(),
								}},
							}},
							config
						}};
						("{ty}".to_string(), resource)
					}}
				}}


				impl {prefix}{name}Builder {{
					pub fn build(&mut self) -> ::std::rc::Rc<{prefix}{name}> {{
						use tf_bindgen::Scope;
						use ::tf_bindgen::value::Prepare;
						let this = {prefix}{name} {{
							__m_scope: self.__m_scope.clone(),
							__m_name: self.__m_name.clone(),
							{assign}
						}};
						let path = format!("{ty}.{{}}", this.path().name());
						let this = ::std::rc::Rc::new(this.prepare(path));
						this.stack().add_resource(this.clone());
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
