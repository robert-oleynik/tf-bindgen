use heck::ToUpperCamelCase;
use quote::__private::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, LitInt, Token, Type};

pub struct Construct {
    pub pub_token: Token![pub],
    pub name: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

#[derive(Debug)]
pub struct Field {
    pub attributes: Vec<Attribute>,
    pub auto: Option<Token![auto]>,
    pub name: syn::Ident,
    pub ty: FieldType,
    pub colon_token: Token![:],
}

#[derive(Debug)]
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

impl Construct {
    pub fn to_token_stream(&self) -> TokenStream {
        let ident = &self.name;
        let fields = self
            .fields
            .iter()
            .filter(|field| field.auto.is_none())
            .map(|field| field.to_field_token_stream(format!("{ident}").as_str()));
        let others = self
            .fields
            .iter()
            .filter(|field| field.auto.is_none())
            .map(|field| field.to_struct_token_stream(ident.to_string().as_ref()));
        quote::quote!(
            pub struct #ident {
                #( #fields ),*
            }
            #( #others )*
        )
    }
}

impl Field {
    pub fn to_field_token_stream(&self, prefix: &str) -> TokenStream {
        let name_str = self.name.to_string().to_upper_camel_case();
        let name = &self.name;
        let ty = self
            .ty
            .to_type_token_stream(format!("{prefix}{name_str}").as_str());
        quote::quote!(#name: #ty)
    }

    /// Generate all structs necessary to create this field.
    pub fn to_struct_token_stream(&self, prefix: &str) -> TokenStream {
        let name_str = self.name.to_string().to_upper_camel_case();
        let prefix = format!("{prefix}{name_str}");
        let ident = Ident::new(&prefix, Span::call_site());
        println!("Type {prefix}");
        let fields = self
            .ty
            .custom_type_fields()
            .iter()
            .cloned()
            .flatten()
            .collect::<Vec<_>>();
        let field_decl = fields
            .iter()
            .filter(|field| field.auto.is_none())
            .map(|field| field.to_field_token_stream(&prefix));
        let impls = fields
            .iter()
            .filter(|field| field.auto.is_none())
            .map(|field| field.to_struct_token_stream(&prefix));
        quote::quote!(
            pub struct #ident {
                #( #field_decl ),*
            }
            #( #impls )*
        )
    }
}

impl FieldType {
    pub fn custom_type_fields(&self) -> Option<impl Iterator<Item = &Field> + Clone> {
        match self {
            FieldType::Object { fields } => Some(fields.iter()),
            FieldType::Map { nested, .. } => nested.custom_type_fields(),
            FieldType::List { nested, min, max } => {
                println!("List Config {min:?} {max:?}");
                nested.custom_type_fields()
            }
            FieldType::Set { nested } => nested.custom_type_fields(),
            FieldType::Type { .. } => None,
        }
    }

    pub fn to_type_token_stream(&self, ty: &str) -> TokenStream {
        match self {
            FieldType::Object { .. } => {
                let ident = syn::Ident::new(ty, Span::call_site());
                quote::quote!( #ident )
            }
            FieldType::Map { key_ty, nested } => {
                let nested = nested.to_type_token_stream(ty);
                quote::quote!( ::std::collections::HashMap<#key_ty, #nested> )
            }
            FieldType::List {
                nested,
                max: Some(lit),
                min,
            } if lit.base10_parse::<usize>().expect("expected usize") == 1 => {
                let nested = nested.to_type_token_stream(ty);
                if let Some(min) = min {
                    if min.base10_parse::<usize>().expect("expected usize") == 1 {
                        return quote::quote!( #nested );
                    }
                }
                quote::quote!( ::std::option::Option<#nested> )
            }
            FieldType::List { nested, .. } => {
                let nested = nested.to_type_token_stream(ty);
                quote::quote!( ::std::vec::Vec<#nested> )
            }
            FieldType::Set { nested } => {
                let nested = nested.to_type_token_stream(ty);
                quote::quote!( ::std::collections::HashSet<#nested> )
            }
            FieldType::Type { ty } => quote::quote!( #ty ),
        }
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
        let auto = input.parse()?;
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            attributes,
            auto,
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
