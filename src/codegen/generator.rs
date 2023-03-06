use std::collections::HashMap;

use heck::{ToSnakeCase, ToUpperCamelCase};
use semver::{Comparator, Op, VersionReq};
use tf_schema::provider::v1_0::{Attribute, Block, BlockType, Provider, Type};
use tf_schema::provider::Schema;

pub struct Generator {
    provider: Vec<ProviderInfo>,
}

/// Stores information used to generate provider bindings.
pub struct ProviderInfo {
    pub provider_type: String,
    version: String,
    info: StructInfo,
    sub_types: Vec<StructInfo>,
    pub resource_constructs: Vec<ConstructInfo>,
    pub data_source_constructs: Vec<ConstructInfo>,
}

/// Stores information used to generate resource and data source bindings.
pub struct ConstructInfo {
    pub resource_type: String,
    name: String,
    fields: Vec<FieldInfo>,
    sub_types: Vec<StructInfo>,
}

/// Stores information required to generate blocks and complex types of constructs.
pub struct StructInfo {
    name: String,
    fields: Vec<FieldInfo>,
}

/// Stores information used to generate attributes and blocks of constructs.
pub struct FieldInfo {
    name: String,
    type_info: String,
    description: Option<String>,
    optional: bool,
    computed: bool,
}

pub struct Fields(Vec<FieldInfo>);
pub struct Structs(Vec<StructInfo>);

impl Generator {
    pub fn provider(&self) -> impl Iterator<Item = &ProviderInfo> {
        self.provider.iter()
    }
}

impl ProviderInfo {
    pub fn as_binding(&self) -> String {
        let struct_binding = self.as_struct_binding();
        let builder_binding = self.as_builder_binding();
        let extra: String = self.sub_types.iter().map(StructInfo::as_binding).collect();
        format!("{struct_binding}\n{builder_binding}\n{extra}")
    }

    pub fn as_struct_binding(&self) -> String {
        let name = &self.info.name;
        let fields: String = self
            .info
            .fields
            .iter()
            .filter_map(FieldInfo::as_field_binding)
            .map(|f| f + ",\n")
            .collect();
        let builder_init: String = self
            .info
            .fields
            .iter()
            .map(|field| format!("{}: None,\n", fix_ident(&field.name)))
            .collect();
        format!(
            r#"pub struct {name} {{
				__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>,
				{fields}
			}}

			impl {name} {{
				pub fn create<C: ::tf_bindgen::Construct + 'static>(scope: &::std::rc::Rc<C>) -> {name}Builder {{
					{name}Builder {{
						__m_scope: scope.clone(),
						{builder_init}
					}}
				}}
			}}"#
        )
    }

    pub fn as_builder_binding(&self) -> String {
        let provider_type = &self.provider_type;
        let name = &self.info.name;
        let version = &self.version;
        let fields: String = self
            .info
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_field_binding)
            .map(|field| field + ",\n")
            .collect();
        let impls: String = self
            .info
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_binding)
            .map(|field| field + "\n")
            .collect();
        let builder_assign: String = self
            .info
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_assign)
            .map(|a| a + ",\n")
            .collect();
        let config_assign: String = self
            .info
            .fields
            .iter()
            .filter(|field| !field.computed || field.optional)
            .map(|field| {
                let name = fix_ident(&field.name);
                format!(
                    r#"let value = ::tf_bindgen::json::to_value(&self.{name}).unwrap();
					config.insert("{name}".to_string(), value);
					"#
                )
            })
            .collect();
        format!(
            r#"pub struct {name}Builder {{
				__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>,
				{fields}
			}}

			impl {name}Builder {{
				{impls}

				pub fn build(&mut self) -> {name} {{
					let result = {name} {{
						__m_scope: self.__m_scope.clone(),
						{builder_assign}
					}};
					let app = result.__m_scope.app();
					let mut config = ::tf_bindgen::json::Map::new();
					{config_assign}
					app.add_provider(
						result.__m_scope.stack(),
						"{provider_type}",
						"{version}",
						config
					);
					result
				}}
			}}"#
        )
    }
}

impl ConstructInfo {
    pub fn as_binding(&self) -> String {
        let construct = self.as_struct_binding();
        let builder = self.as_builder_binding();
        let others: String = self.sub_types.iter().map(StructInfo::as_binding).collect();
        format!("{construct}\n{builder}\n{others}")
    }

    pub fn as_struct_binding(&self) -> String {
        let name = &self.name;
        let fields: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_field_binding)
            .map(|field| field + ",\n")
            .collect();
        let builder_init: String = self
            .fields
            .iter()
            .filter(|field| !field.computed || field.optional)
            .map(|field| format!("{}: None,\n", fix_ident(&field.name)))
            .collect();
        format!(
            r#"pub struct {name} {{
				__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>,
				__m_name: ::std::string::String,
				{fields}
			}}

			impl {name} {{
				pub fn create<C: ::tf_bindgen::Construct + 'static>(
					scope: &::std::rc::Rc<C>,
					name: impl ::std::convert::Into<::std::string::String>
				) -> {name}Builder {{
					{name}Builder {{
						__m_scope: scope.clone(),
						__m_name: name.into(),
						{builder_init}
					}}
				}}
			}}
			
			impl ::tf_bindgen::Construct for {name} {{
				fn app(&self) -> ::tf_bindgen::app::App {{
					self.__m_scope.app()
				}}

				fn stack(&self) -> &str {{
					self.__m_scope.stack()
				}}

				fn name(&self) -> &str {{
					self.__m_scope.name()
				}}

				fn path(&self) -> ::std::string::String {{
					self.__m_scope.path() + "/" + &self.__m_name
				}}
			}}"#
        )
    }

    pub fn as_builder_binding(&self) -> String {
        let resource_type = &self.resource_type;
        let name = &self.name;
        let fields: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_field_binding)
            .map(|field| field + ",\n")
            .collect();
        let impls: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_binding)
            .map(|field| field + "\n")
            .collect();
        let builder_assign: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_assign)
            .map(|field| field + ",\n")
            .collect();
        let config_assign: String = self
            .fields
            .iter()
            .filter(|field| !field.computed || field.optional)
            .map(|field| {
                let name = fix_ident(&field.name);
                format!(
                    r#"let value = ::tf_bindgen::json::to_value(&self.{name}).unwrap();
					config.insert("{name}".to_string(), value);
					"#
                )
            })
            .collect();
        format!(
            r#"pub struct {name}Builder {{
				__m_scope: ::std::rc::Rc<dyn ::tf_bindgen::Construct>,
				__m_name: ::std::string::String,
				{fields}
			}}

			impl {name}Builder {{
				{impls}

				pub fn build(&mut self) -> {name} {{
					use ::tf_bindgen::Construct;

					let result = {name} {{
						__m_scope: self.__m_scope.clone(),
						__m_name: self.__m_name.clone(),
						{builder_assign}
					}};

					let mut config = ::std::collections::HashMap::new();
					{config_assign}
					let resource = ::tf_bindgen::schema::document::Resource {{
						meta: ::tf_bindgen::schema::document::ResourceMeta {{
							metadata: ::tf_bindgen::schema::document::ResourceMetadata {{
								path: result.path(),
								unique_id: result.name().to_string(),
							}},
						}},
						config
					}};
					let app = result.app();
					app.add_resource(result.stack(), "{resource_type}", result.name(), resource);
					result
				}}
			}}"#
        )
    }
}

impl StructInfo {
    pub fn as_binding(&self) -> String {
        let struct_binding = self.as_struct_binding();
        let builder_binding = self.as_builder_binding();
        format!("{struct_binding}\n{builder_binding}")
    }
    pub fn as_struct_binding(&self) -> String {
        let name = &self.name;
        let fields: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_field_binding)
            .map(|field| field + ",\n")
            .collect();
        let builder_init: String = self
            .fields
            .iter()
            .filter(|field| !field.computed || field.optional)
            .map(|field| format!("{}: None,\n", fix_ident(&field.name)))
            .collect();
        format!(
            r#"#[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
			#[serde(crate = "::tf_bindgen::serde")]
			pub struct {name} {{
				{fields}
			}}

			impl {name} {{
				pub fn builder() -> {name}Builder {{
					{name}Builder {{
						{builder_init}
					}}
				}}
			}}"#
        )
    }

    pub fn as_builder_binding(&self) -> String {
        let name = &self.name;
        let fields: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_field_binding)
            .map(|field| field + ",\n")
            .collect();
        let impls: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_binding)
            .map(|field| field + "\n")
            .collect();
        let builder_assign: String = self
            .fields
            .iter()
            .filter_map(FieldInfo::as_builder_assign)
            .map(|field| field + ",\n")
            .collect();
        format!(
            r#"pub struct {name}Builder {{
				{fields}
			}}

			impl {name}Builder {{
				{impls}

				pub fn build(&mut self) -> {name} {{
					{name} {{
						{builder_assign}
					}}
				}}
			}}"#
        )
    }
}

impl FieldInfo {
    pub fn as_field_binding(&self) -> Option<String> {
        let name = fix_ident(&self.name);
        let ty = &self.type_info;
        match (self.computed, self.optional) {
            (true, false) => None,
            (_, true) => Some(format!("{name}: ::std::option::Option<{ty}>")),
            (_, false) => Some(format!("{name}: {ty}")),
        }
    }

    pub fn as_builder_field_binding(&self) -> Option<String> {
        let name = fix_ident(&self.name);
        let ty = &self.type_info;
        match (self.computed, self.optional) {
            (true, false) => None,
            (_, _) => Some(format!("{name}: ::std::option::Option<{ty}>")),
        }
    }

    pub fn as_builder_binding(&self) -> Option<String> {
        let name = fix_ident(&self.name);
        let ty = &self.type_info;
        let desc: String = self
            .description
            .clone()
            .unwrap_or(String::new())
            .lines()
            .map(|line| format!("/// {line}\n"))
            .collect();
        match (self.computed, self.optional) {
            (true, false) => None,
            (_, _) => Some(format!(
                r#"{desc}pub fn {name}(&mut self, value: impl ::std::convert::Into<{ty}>) -> &mut Self {{
					self.{name} = Some(value.into());
					self
				}}"#
            )),
        }
    }

    pub fn as_builder_assign(&self) -> Option<String> {
        let name = fix_ident(&self.name);
        match (self.computed, self.optional) {
            (true, false) => None,
            (_, true) => Some(format!(r#"{name}: self.{name}.clone()"#)),
            (_, false) => Some(format!(
                r#"{name}: self.{name}.clone().expect("missing field `{name}`")"#
            )),
        }
    }
}

impl From<(&Schema, &HashMap<String, VersionReq>)> for Generator {
    fn from((schema, versions): (&Schema, &HashMap<String, VersionReq>)) -> Self {
        let provider = match schema {
            Schema::V1_0 { provider_schemas } => provider_schemas
                .iter()
                .map(|(name, schema)| {
                    let n = name.split('/').last().unwrap();
                    let version = versions
                        .get(n)
                        .unwrap()
                        .comparators
                        .iter()
                        .cloned()
                        .map(cargo_simplify_version)
                        .fold(String::from(">=0.0.0"), |text, c| text + "," + &c);
                    (n, name, schema, version)
                })
                .map(ProviderInfo::from)
                .collect(),
            Schema::Unknown => unimplemented!("schema version not supported"),
        };
        Self { provider }
    }
}

impl From<(&str, &String, &Provider, String)> for ProviderInfo {
    fn from((name, provider_type, schema, version): (&str, &String, &Provider, String)) -> Self {
        let name = name.to_upper_camel_case();
        let fields = Fields::from((&name, &schema.provider.block)).0;
        let sub_types = Structs::from((&name, &schema.provider.block)).0;
        let resource_constructs = schema
            .resource_schemas
            .iter()
            .map(|(name, resource)| (name, &resource.block))
            .map(ConstructInfo::from)
            .collect::<Vec<_>>();
        let data_source_constructs = schema
            .data_source_schemas
            .iter()
            .map(|(name, data_source)| (name, &data_source.block))
            .map(ConstructInfo::from)
            .collect::<Vec<_>>();
        ProviderInfo {
            provider_type: provider_type.clone(),
            version: version.clone(),
            info: StructInfo {
                name: name.to_upper_camel_case(),
                fields,
            },
            sub_types,
            resource_constructs,
            data_source_constructs,
        }
    }
}

impl From<(&String, &Block)> for ConstructInfo {
    fn from((name, schema): (&String, &Block)) -> Self {
        let resource_type = name.clone();
        let name = name.to_upper_camel_case();
        let fields = Fields::from((&name, schema)).0;
        let structs = Structs::from((&name, schema)).0;
        ConstructInfo {
            resource_type,
            name,
            fields,
            sub_types: structs,
        }
    }
}

impl From<(&String, &Block)> for Fields {
    fn from((prefix, schema): (&String, &Block)) -> Self {
        let mut fields = schema
            .attributes
            .iter()
            .flatten()
            .map(|(name, attribute)| (prefix, name, attribute))
            .map(FieldInfo::from)
            .collect::<Vec<_>>();
        let iter = schema
            .block_types
            .iter()
            .flatten()
            .map(|(name, block)| (prefix, name, block))
            .map(FieldInfo::from);
        fields.extend(iter);
        Fields(fields)
    }
}

impl From<(&String, &Block)> for Structs {
    fn from((prefix, schema): (&String, &Block)) -> Self {
        let mut structs = schema
            .attributes
            .iter()
            .flatten()
            .map(|(name, attr)| (prefix, name, &attr.r#type))
            .map(Structs::from)
            .flat_map(|st| st.0)
            .collect::<Vec<_>>();
        structs.extend(
            schema
                .block_types
                .iter()
                .flatten()
                .flat_map(|(name, ty)| match ty {
                    Type::Single { block } | Type::List { block, .. } => {
                        let name = format!("{prefix}{}", name.to_upper_camel_case());
                        let mut structs = vec![StructInfo::from((&name, block.as_ref()))];
                        structs.extend(Structs::from((&name, block.as_ref())).0);
                        structs
                    }
                }),
        );
        Structs(structs)
    }
}

impl From<(&String, &String, &BlockType)> for Structs {
    fn from((prefix, name, schema): (&String, &String, &BlockType)) -> Self {
        let type_name = format!("{prefix}{}", name.to_upper_camel_case());
        match schema {
            BlockType::Set(ty) | BlockType::Map(ty) | BlockType::List(ty) => {
                let ty: &BlockType = &*ty;
                Structs::from((prefix, name, ty))
            }
            BlockType::Object(fields) => {
                let mut structs = vec![StructInfo::from((&type_name, fields))];
                let iter = fields
                    .iter()
                    .map(|(name, ty)| (&type_name, name, ty))
                    .map(Structs::from)
                    .flat_map(|s| s.0);
                structs.extend(iter);
                Structs(structs)
            }
            _ => Structs(Vec::new()),
        }
    }
}

impl From<(&String, &HashMap<String, BlockType>)> for StructInfo {
    fn from((name, fields): (&String, &HashMap<String, BlockType>)) -> Self {
        StructInfo {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(n, field)| (name, n, field))
                .map(FieldInfo::from)
                .collect(),
        }
    }
}

impl From<(&String, &Block)> for StructInfo {
    fn from((name, schema): (&String, &Block)) -> Self {
        StructInfo {
            name: name.to_string(),
            fields: Fields::from((name, schema)).0,
        }
    }
}

impl From<(&String, &String, &Attribute)> for FieldInfo {
    fn from((prefix, name, attribute): (&String, &String, &Attribute)) -> Self {
        let comp = attribute.computed.unwrap_or(false);
        let opt = attribute.optional.unwrap_or(false);
        let req = attribute.required.unwrap_or(comp && !opt);
        assert_ne!(opt, req, "Expected to be required or optional not both");
        Self {
            name: name.to_string(),
            type_info: from_typename(&prefix, &name, &attribute.r#type),
            description: attribute.description.clone(),
            optional: opt,
            computed: comp,
        }
    }
}

impl From<(&String, &String, &BlockType)> for FieldInfo {
    fn from((prefix, name, ty): (&String, &String, &BlockType)) -> Self {
        FieldInfo {
            name: name.clone(),
            type_info: from_typename(prefix, name, ty),
            description: None,
            optional: false,
            computed: false,
        }
    }
}

impl From<(&String, &String, &Type)> for FieldInfo {
    fn from((prefix, name, schema): (&String, &String, &Type)) -> Self {
        let optional = match schema {
            Type::Single { .. } => true,
            Type::List {
                min_items,
                max_items: Some(1),
                ..
            } => min_items.unwrap_or(0) != 1,
            Type::List { .. } => false,
        };
        let typename = format!("{prefix}{}", name.to_upper_camel_case());
        let type_info = match schema {
            Type::Single { .. } => typename,
            Type::List {
                max_items: Some(1), ..
            } => typename,
            Type::List { .. } => format!("::std::vec::Vec<{typename}>",),
        };
        FieldInfo {
            name: name.to_snake_case(),
            type_info,
            description: None,
            optional,
            computed: false,
        }
    }
}

fn from_typename(prefix: &str, name: &str, schema: &BlockType) -> String {
    match schema {
        BlockType::String => "::std::string::String".to_string(),
        BlockType::Bool => "bool".to_string(),
        BlockType::Number => "i64".to_string(),
        BlockType::Dynamic => "::tf_bindgen::json::Value".to_string(),
        BlockType::Set(ty) => format!(
            "::std::collections::HashSet<{}>",
            from_typename(prefix, name, &*ty)
        ),
        BlockType::Map(ty) => format!(
            "::std::collections::HashMap<::std::string::String, {}>",
            from_typename(prefix, name, &*ty)
        ),
        BlockType::List(ty) => format!("::std::vec::Vec<{}>", from_typename(prefix, name, &*ty)),
        BlockType::Object(_) => format!("{prefix}{}", name.to_upper_camel_case()),
    }
}

pub fn cargo_simplify_version(constraint: Comparator) -> String {
    let major = constraint.major;
    let minor = constraint.minor.unwrap_or(0);
    let patch = constraint.patch.unwrap_or(0);
    assert!(
        constraint.pre.is_empty(),
        "pre release constraints are not supported"
    );
    match constraint.op {
        Op::Tilde if constraint.minor.is_some() => {
            format!(">={major}{minor}{patch},<{major}{}.0", minor + 1)
        }
        Op::Caret if major == 0 && constraint.minor.is_none() => ">=0.0.0,<1.0.0".to_string(),
        Op::Caret if major == 0 && minor == 0 && constraint.patch.is_some() => {
            format!(">=0.0.{patch},<0.0.{}", patch + 1)
        }
        Op::Caret if major == 0 => {
            format!(">=0.{minor}.0,<0.{}.0", minor + 1)
        }
        Op::Wildcard if constraint.minor.is_some() => {
            format!(">={major}.{minor}.0,<{major}.{}.0", minor + 1)
        }
        Op::Tilde | Op::Caret | Op::Wildcard => {
            format!(">={major}.{minor}.{patch},<{}.0.0", major + 1)
        }
        Op::Exact => format!("={major}.{minor}.{patch}"),
        Op::Greater => format!(">{major}.{minor}.{patch}"),
        Op::GreaterEq => format!(">={major}.{minor}.{patch}"),
        Op::Less => format!("<{major}.{minor}.{patch}"),
        Op::LessEq => format!("<={major}.{minor}.{patch}"),
        _ => unimplemented!(),
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
