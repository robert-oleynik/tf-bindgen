use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, LitInt, Token, Type};

pub struct Construct {
    pub pub_token: Token![pub],
    pub name: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

pub struct Field {
    pub attributes: Vec<Attribute>,
    pub auto: Option<Token![auto]>,
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
