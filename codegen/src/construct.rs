use heck::ToUpperCamelCase;
use quote::__private::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, LitInt, Token, Type};

use crate::struct_info::{FieldInfo, StructInfo, StructType};

pub struct Provider {
    pub provider: syn::LitStr,
    pub version: syn::LitStr,
    pub construct: Construct,
}

pub struct Construct {
    pub pub_token: Token![pub],
    pub name: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

pub struct Field {
    pub attributes: Vec<Attribute>,
    pub auto: bool,
    pub opt: bool,
    pub name: syn::Ident,
    pub ty: FieldType,
    pub colon_token: Token![:],
}

pub enum FieldType {
    Object {
        fields: Punctuated<Field, Token![,]>,
    },
    Map {
        key_ty: Type,
        nested: Box<FieldType>,
    },
    List {
        min: Option<LitInt>,
        max: Option<LitInt>,
        nested: Box<FieldType>,
    },
    Set {
        nested: Box<FieldType>,
    },
    Type {
        ty: syn::Type,
    },
}

impl Into<Vec<StructInfo>> for Construct {
    fn into(self) -> Vec<StructInfo> {
        let name_str = self.name.to_string();
        let this = StructInfo {
            struct_type: StructType::Construct,
            name: self.name,
            fields: self
                .fields
                .iter()
                .filter(|f| !f.auto || f.opt)
                .map(|f| f.as_field_info(&name_str))
                .collect(),
        };
        let mut result = vec![this];
        let iter = self
            .fields
            .iter()
            .filter(|f| !f.auto || f.opt)
            .flat_map(|f| f.as_struct_info(&name_str));
        result.extend(iter);
        result
    }
}

impl Field {
    pub fn as_struct_info(&self, type_prefix: &str) -> Vec<StructInfo> {
        let mut result = Vec::new();
        if let Some(fields) = self.ty.custom_type_fields() {
            let name_str = self.name.to_string().to_upper_camel_case();
            let ident_str = format!("{type_prefix}{name_str}");
            let ident = Ident::new(&ident_str, Span::call_site());
            let this = StructInfo {
                struct_type: StructType::Inner,
                name: ident,
                fields: fields
                    .clone()
                    .filter(|f| !f.auto || f.opt)
                    .map(|f| f.as_field_info(&ident_str))
                    .collect(),
            };
            result.push(this);
            result.extend(
                fields
                    .filter(|f| !f.auto || f.opt)
                    .flat_map(|f| f.as_struct_info(&ident_str)),
            );
        }
        result
    }

    fn as_field_info(&self, parent_type: &str) -> (Ident, FieldInfo) {
        let name = self.name.to_string().to_upper_camel_case();
        let name = format!("{parent_type}{name}");
        let ident = self.name.clone();
        let info = FieldInfo {
            computed: self.auto,
            attributes: self.attributes.clone(),
            ty: self.ty.as_type(&name, self.opt, false),
        };
        (ident, info)
    }
}

impl FieldType {
    fn as_type(&self, custom_type_name: &str, optional: bool, nested: bool) -> syn::Type {
        let tokens = match self {
            FieldType::Object { .. } => {
                let custom_type_ident = Ident::new(custom_type_name, Span::call_site());
                if nested {
                    quote::quote!(#custom_type_ident)
                } else {
                    quote::quote!(::std::option::Option<#custom_type_ident>)
                }
            }
            FieldType::Map { key_ty, nested } => {
                let ty = nested.as_type(custom_type_name, false, true);
                quote::quote!(::std::collections::HashMap<#key_ty, #ty>)
            }
            FieldType::List { nested, min, max } => {
                let ty = nested.as_type(custom_type_name, false, true);
                match (min, max) {
                    (Some(min), Some(max))
                        if min.base10_parse::<usize>().unwrap() == 1
                            && max.base10_parse::<usize>().unwrap() == 1 =>
                    {
                        quote::quote!(#ty)
                    }
                    (_, Some(max)) if max.base10_parse::<usize>().unwrap() == 1 => {
                        quote::quote!(::std::option::Option<#ty>)
                    }
                    _ => quote::quote!(::std::vec::Vec<#ty>),
                }
            }
            FieldType::Set { nested } => {
                let ty = nested.as_type(custom_type_name, false, true);
                quote::quote!(::std::collections::HashSet<#ty>)
            }
            FieldType::Type { ty } => quote::quote!(#ty),
        };
        let tokens = if optional && !nested {
            quote::quote!(::std::option::Option<#tokens>)
        } else {
            tokens
        };
        syn::parse2(tokens).unwrap()
    }
}

impl FieldType {
    pub fn custom_type_fields(&self) -> Option<impl Iterator<Item = &Field> + Clone> {
        match self {
            FieldType::Object { fields } => Some(fields.iter()),
            FieldType::Map { nested, .. } => nested.custom_type_fields(),
            FieldType::List { nested, .. } => nested.custom_type_fields(),
            FieldType::Set { nested } => nested.custom_type_fields(),
            FieldType::Type { .. } => None,
        }
    }
}

impl Parse for Provider {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let provider = input.parse()?;
        let _: Token![:] = input.parse()?;
        let version = input.parse()?;
        let _: Token![,] = input.parse()?;
        Ok(Self {
            provider,
            version,
            construct: input.parse()?,
        })
    }
}

impl Parse for Construct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pub_token = input.parse()?;
        let ident = input.parse()?;
        let content;
        Ok(Self {
            pub_token,
            name: ident,
            brace_token: syn::braced!(content in input),
            fields: content.parse_terminated(Field::parse)?,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let mut auto = false;
        let mut opt = false;
        while input.peek(Token![@]) {
            let _: Token![@] = input.parse()?;
            let i: Ident = input.parse()?;
            match format!("{i}").as_str() {
                "auto" => auto = true,
                "opt" => opt = true,
                _ => panic!("unknown modifier `@{i}`"),
            }
        }
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            attributes,
            auto,
            opt,
            name,
            colon_token,
            ty,
        })
    }
}

impl Parse for FieldType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            let fields = content.parse_terminated(Field::parse)?;
            Ok(FieldType::Object { fields })
        } else if lookahead.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            if content.is_empty() {
                input.parse::<Token![=>]>()?;
                let nested = input.parse()?;
                return Ok(FieldType::Set { nested });
            }

            let lookahead = content.lookahead1();
            if lookahead.peek(Token![..]) || lookahead.peek(syn::LitInt) {
                let min = content.parse()?;
                content.parse::<Token![..]>()?;
                let max = content.parse()?;
                input.parse::<Token![=>]>()?;
                let nested = input.parse()?;
                return Ok(FieldType::List { min, max, nested });
            } else if lookahead.peek(Token![::]) || lookahead.peek(Ident) {
                let key_ty = content.parse()?;
                input.parse::<Token![=>]>()?;
                let nested = input.parse()?;
                return Ok(FieldType::Map { key_ty, nested });
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(Token![::]) || lookahead.peek(Ident) {
            let ty = input.parse()?;
            Ok(FieldType::Type { ty })
        } else {
            Err(lookahead.error())
        }
    }
}
