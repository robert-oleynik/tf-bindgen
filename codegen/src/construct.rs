use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;

pub struct Construct {
    pub pub_token: Token![pub],
    pub name: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

pub struct Field {
    pub name: syn::Ident,
    pub colon_token: Token![:],
}

pub enum FieldType {}

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
        let name = input.parse()?;
        let colon_token = input.parse()?;
        Ok(Self { name, colon_token })
    }
}

impl Parse for FieldType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}
