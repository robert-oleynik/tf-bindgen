use heck::ToUpperCamelCase;
use quote::ToTokens;
use quote::__private::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::Token;

mod keyword {
    syn::custom_keyword!(resource);
    syn::custom_keyword!(scope);
}

pub struct Block {
    scope: syn::Expr,
    ty: syn::LitStr,
    name: syn::LitStr,
    body: Body,
}

pub struct Body {
    attributes: Vec<Attribute>,
}

pub enum Attribute {
    Block { name: syn::Ident, body: Body },
    Field { name: syn::Ident, assign: syn::Expr },
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.ty.value().to_upper_camel_case();
        let name = syn::Ident::new(&name, self.ty.span());
        let module = self.ty.value();
        let module = syn::Ident::new(&module, self.ty.span());
        let setter = self.body.to_setter_tokens();
        let blocks = self.body.to_block_tokens(&name, &module);
        let scope = &self.scope;
        let name_str = &self.name;
        tokens.extend(quote::quote!(
            {
                #( #blocks )*
                #module::#name::create(#scope, #name_str)
                    #( #setter )*
                    .build()
            }
        ))
    }
}

impl Body {
    fn to_setter_tokens(&self) -> impl Iterator<Item = TokenStream> + '_ {
        use Attribute::*;
        self.attributes
            .iter()
            .enumerate()
            .map(|(i, attr)| match attr {
                Block { name, .. } => {
                    let field_name = format!("{name}{i}");
                    let field_name = syn::Ident::new(&field_name, name.span());
                    quote::quote!(.#name(#field_name))
                }
                Field { name, assign } => quote::quote!(.#name(#assign)),
            })
    }

    fn to_block_tokens<'a>(
        &'a self,
        name: &'a syn::Ident,
        module: &'a syn::Ident,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        use Attribute::*;
        self.attributes
            .iter()
            .enumerate()
            .filter_map(move |(i, attr)| match attr {
                Block { name: aname, body } => {
                    let n = format!("{name}{}", aname.to_string().to_upper_camel_case());
                    let n = syn::Ident::new(&n, aname.span());
                    let aname = syn::Ident::new(&format!("{aname}{i}"), aname.span());
                    Some((aname, body.to_tokens(n, module)))
                }
                _ => None,
            })
            .map(|(name, body)| quote::quote!( let #name = #body; ))
    }

    fn to_tokens(&self, name: syn::Ident, module: &syn::Ident) -> TokenStream {
        let setter = self.to_setter_tokens();
        let blocks = self.to_block_tokens(&name, module);
        quote::quote!(
            {
                #( #blocks )*
                #module::#name::builder()
                    #( #setter )*
                    .build()
            }
        )
    }
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let scope = input.parse()?;
        let _: Token![,] = input.parse()?;
        let _: keyword::resource = input.parse()?;
        let ty = input.parse()?;
        let name = input.parse()?;
        let body = input.parse()?;
        Ok(Self {
            scope,
            ty,
            name,
            body,
        })
    }
}

impl Parse for Body {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::braced!(content in input);
        let attributes = (0..)
            .take_while(|_| !content.is_empty())
            .map(|_| content.parse())
            .collect::<Result<_, _>>()?;
        Ok(Self { attributes })
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let assign = input.parse()?;
            return Ok(Attribute::Field { name, assign });
        }
        let body = input.parse()?;
        Ok(Attribute::Block { name, body })
    }
}
