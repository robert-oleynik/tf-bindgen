use std::collections::HashMap;

use heck::ToUpperCamelCase;
use itertools::Itertools;
use semver::{Comparator, Op, VersionReq};
use tf_schema::provider::v1_0::{Attribute, Block, BlockType, Type};
use tf_schema::provider::Schema;

use crate::codegen::type_info::TypeInfo;

use self::field_info::FieldInfo;
use self::path::Path;
use self::struct_info::{StructInfo, StructType};
use self::type_info::Wrapper;

pub mod field_info;
pub mod path;
pub mod struct_info;
pub mod type_info;

pub use tf_codegen::resource;
pub use tf_codegen::Construct;

pub struct Generator {
    pub providers: Vec<Provider>,
}

#[derive(Debug)]
pub struct Provider {
    pub provider: StructInfo,
    pub resources: Vec<StructInfo>,
    pub data_sources: Vec<StructInfo>,
}

pub struct Nested(Vec<StructInfo>);
pub struct Constructs(Vec<StructInfo>);
pub struct Fields(Vec<FieldInfo>);

impl Generator {
    pub fn from_schema(schema: Schema, versions: HashMap<String, VersionReq>) -> Self {
        let schemas = match schema {
            Schema::V1_0 { provider_schemas } => provider_schemas
                .iter()
                .map(|(url, schema)| {
                    let name = url.split("/").last().unwrap();
                    let version = versions
                        .iter()
                        .find(|(n, _)| n.split("/").last().unwrap() == name)
                        .unwrap()
                        .1
                        .comparators
                        .iter()
                        .map(|comp| cargo_simplify_version(comp.clone()))
                        .join(",");
                    let provider =
                        StructInfo::from_provider(name, version, url, &schema.provider.block);
                    let resources = schema
                        .resource_schemas
                        .iter()
                        .map(|(name, schema)| {
                            let path = Path::empty();
                            let this_path = Path::new(vec![name.to_string()]);
                            let fields = Fields::from_schema(&this_path, &schema.block).0;
                            let nested = Nested::from_schema(&this_path, &schema.block).0;
                            let ty = StructType::Construct {
                                ty: name.clone(),
                                nested,
                            };
                            StructInfo::builder()
                                .ty(ty)
                                .name(name.clone())
                                .path(path)
                                .fields(fields)
                                .build()
                                .unwrap()
                        })
                        .collect();
                    let data_sources = schema
                        .data_source_schemas
                        .iter()
                        .map(|(name, schema)| {
                            let path = Path::new(vec!["data".to_string()]);
                            let this_path = Path::new(vec!["data".to_string(), name.to_string()]);
                            let nested = Nested::from_schema(&this_path, &schema.block).0;
                            let ty = StructType::Construct {
                                ty: name.clone(),
                                nested,
                            };
                            let fields = Fields::from_schema(&this_path, &schema.block).0;
                            StructInfo::builder()
                                .ty(ty)
                                .name(name.clone())
                                .path(path)
                                .fields(fields)
                                .build()
                                .unwrap()
                        })
                        .collect();
                    Provider {
                        provider,
                        resources,
                        data_sources,
                    }
                })
                .collect(),
            Schema::Unknown => unimplemented!("unsupported provider version"),
        };
        Generator { providers: schemas }
    }
}

impl StructInfo {
    pub fn from_provider(
        name: impl Into<String>,
        version: impl Into<String>,
        url: impl Into<String>,
        schema: &Block,
    ) -> Self {
        let name = name.into();
        let path = Path::empty();
        let nested = Nested::from_schema(&path, schema);
        let ty = StructType::Provider {
            ty: url.into(),
            ver: version.into(),
            nested: nested.0,
        };
        let fields = Fields::from_schema(&path, schema);
        StructInfo::builder()
            .ty(ty)
            .path(path)
            .name(name)
            .fields(fields.0)
            .build()
            .unwrap()
    }
}

fn get_fields(ty: &BlockType) -> Option<&HashMap<String, BlockType>> {
    match ty {
        BlockType::Set(inner) | BlockType::Map(inner) | BlockType::List(inner) => get_fields(inner),
        BlockType::Object(fields) => Some(fields),
        _ => None,
    }
}

impl Nested {
    pub fn from_schema(path: &Path, block: &Block) -> Self {
        let iter = block
            .block_types
            .iter()
            .flatten()
            .flat_map(|(name, field)| {
                let mut this_path = path.clone();
                this_path.push(name);
                let schema = match field {
                    Type::Single { block } => block,
                    Type::List { block, .. } => block,
                };
                let this = StructInfo::from_type(path, name, field);
                Nested::from_schema(&this_path, schema)
                    .0
                    .into_iter()
                    .chain(vec![this])
                    .collect::<Vec<_>>()
            });
        let nested = block
            .attributes
            .iter()
            .flatten()
            .flat_map(|(name, field)| {
                let mut this_path = path.clone();
                this_path.push(name);
                let fields = get_fields(&field.r#type);
                let this = fields
                    .iter()
                    .map(|fields| StructInfo::from_fields(path, name, fields));
                fields
                    .iter()
                    .flat_map(move |fields| Nested::from_fields(&this_path, fields).0)
                    .chain(this)
                    .collect::<Vec<_>>()
            })
            .chain(iter)
            .collect();
        Nested(nested)
    }

    pub fn from_fields(path: &Path, fields: &HashMap<String, BlockType>) -> Self {
        let nested = fields
            .iter()
            .filter_map(|(name, ty)| Some((name, get_fields(ty)?)))
            .flat_map(|(name, fields)| {
                let this = StructInfo::from_fields(path, name, fields);
                let mut this_path = path.clone();
                this_path.push(name);
                Nested::from_fields(&this_path, fields)
                    .0
                    .into_iter()
                    .chain(vec![this])
                    .collect::<Vec<_>>()
            })
            .collect();
        Nested(nested)
    }
}

impl Fields {
    pub fn from_schema(path: &Path, schema: &Block) -> Self {
        let attr_fields = schema
            .attributes
            .iter()
            .flatten()
            .map(|(name, field)| FieldInfo::from_field(path, name, field));
        let fields = schema
            .block_types
            .iter()
            .flatten()
            .map(|(name, field)| FieldInfo::from_type(path, name, field))
            .chain(attr_fields)
            .collect();
        Fields(fields)
    }

    pub fn from_fields(path: &Path, fields: &HashMap<String, BlockType>) -> Self {
        let fields = fields
            .iter()
            .map(|(name, ty)| {
                FieldInfo::builder()
                    .path(path.clone())
                    .name(name.clone())
                    .type_info(TypeInfo::from_schema(path, name, ty))
                    .optional(true)
                    .computed(false)
                    .description(None)
                    .build()
                    .unwrap()
            })
            .collect();
        Fields(fields)
    }
}

impl StructInfo {
    pub fn from_type(path: &Path, name: impl Into<String>, ty: &Type) -> Self {
        let name = name.into();
        let mut this_path = path.clone();
        this_path.push(&name);
        let ty: &Block = match ty {
            Type::Single { block } => block,
            Type::List { block, .. } => block,
        };
        let fields = Fields::from_schema(&this_path, ty).0;
        StructInfo::builder()
            .ty(StructType::Nested)
            .path(path.clone())
            .name(name)
            .fields(fields)
            .build()
            .unwrap()
    }

    pub fn from_fields(
        path: &Path,
        name: impl Into<String>,
        fields: &HashMap<String, BlockType>,
    ) -> Self {
        let name = name.into();
        let mut this_path = path.clone();
        this_path.push(&name);
        let fields = Fields::from_fields(&this_path, fields);
        StructInfo::builder()
            .ty(StructType::Nested)
            .path(path.clone())
            .name(name)
            .fields(fields.0)
            .build()
            .unwrap()
    }
}

impl FieldInfo {
    pub fn from_field(path: &Path, name: impl Into<String>, field: &Attribute) -> Self {
        let opt = field.optional.unwrap_or(false);
        let comp = field.computed.unwrap_or(false);
        let req = field.required.unwrap_or(comp && !opt);
        assert_ne!(opt, req, "Field must be optional and required not both.");
        let name = name.into();
        let type_info = TypeInfo::from_schema(path, &name, &field.r#type);
        FieldInfo::builder()
            .path(path.clone())
            .name(name)
            .type_info(type_info)
            .description(field.description.clone())
            .optional(opt)
            .computed(comp)
            .build()
            .unwrap()
    }

    pub fn from_type(path: &Path, name: impl Into<String>, field: &Type) -> Self {
        let name = name.into();
        let req = match field {
            Type::Single { .. } => false,
            Type::List {
                min_items: Some(1),
                max_items: Some(1),
                ..
            } => true,
            Type::List { .. } => false,
        };
        let type_name = path.type_name() + &name.to_upper_camel_case();
        let type_wrapper = match field {
            Type::Single { .. } => Wrapper::Type,
            Type::List { .. } => Wrapper::List,
        };
        FieldInfo::builder()
            .path(path.clone())
            .name(name)
            .type_info(TypeInfo::new(type_wrapper, type_name))
            .optional(!req)
            .computed(false)
            .description(None)
            .build()
            .unwrap()
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
