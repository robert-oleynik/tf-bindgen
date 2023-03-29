use proc_macro::TokenStream;
use quote::__private::Span;
use syn::{ItemStruct, LitStr};

mod resource;

/// Used to generate a builder chain for a specified Terraform resource or data source. To
/// describe the resource this macro will use a simplified version of Terraform's HCL syntax.
///
/// # Usage
///
/// ```rust
/// use tf_codegen::resource;
///
/// resource! {
///		&scope,
///		resource "kubernetes_pod" "nginx" {
///			metadata {
///				name = "nginx"
///			}
///			spec {
///				container {
///					image = "nginx"
///					port {
///						container_port = 80
///					}
///				}
///			}
///		}
/// }
/// ```
#[proc_macro]
pub fn resource(item: TokenStream) -> TokenStream {
    let block = syn::parse_macro_input!(item as resource::Block);
    quote::quote!(#block).into()
}

/// Used to generate an implementation of [`tf_bindgen::Construct`]. Requires to fields to be
/// annotated with `#[id]` and `#[scope]`.
///
/// # Usage
///
/// ```rust
/// #[derive(tf_codegen::Construct)]
/// #[construct(crate = "::tf_codegen")] // optional
/// pub struct Custom {
///		#[scope]
///		__m_scope: Rc<dyn ::tf_bindgen::Construct>,
///		#[id]
///		__m_name: String
/// }
/// ```
#[proc_macro_derive(Construct, attributes(construct, scope, id))]
pub fn construct_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemStruct);

    let crate_path = match input
        .attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::List(list) => Some(list),
            _ => None,
        })
        .filter(|list| list.path.is_ident("construct"))
        .map(|list| syn::parse2::<syn::LitStr>(list.tokens.clone()))
        .last()
        .unwrap_or(Ok(LitStr::new("::tf_bindgen", Span::call_site())))
    {
        Ok(path) => path,
        Err(err) => return err.into_compile_error().into(),
    };
    let crate_path: syn::Path = match syn::parse_str(&crate_path.value()) {
        Ok(path) => path,
        Err(err) => return err.into_compile_error().into(),
    };

    let id_field = input
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| match &attr.meta {
                syn::Meta::Path(path) => path.is_ident("id"),
                _ => false,
            })
        })
        .next()
        .expect("missing `#[id]` annotation")
        .ident
        .as_ref()
        .expect("expected named fields");
    let scope_field = input
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| match &attr.meta {
                syn::Meta::Path(path) => path.is_ident("scope"),
                _ => false,
            })
        })
        .next()
        .expect("missing `#[scope]` annotation")
        .ident
        .as_ref()
        .expect("expected named fields");
    let name = &input.ident;

    quote::quote!(
        impl #crate_path::Scope for #name {
            fn stack(&self) -> #crate_path::Stack {
                self.#scope_field.stack()
            }

            fn path(&self) -> #crate_path::Path {
                let mut path = self.#scope_field.path();
                path.push(&self.#id_field);
                path
            }
        }
    )
    .into()
}
