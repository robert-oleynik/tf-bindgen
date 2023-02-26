use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::struct_info::StructType;

mod construct;
mod struct_info;

/// Used to generate a construct from declaration.
///
/// # Examples
///
/// ```rust
/// terraform_bindgen_codegen::construct! {
///		pub KubernetesService {
///			metadata: {
///				name: String
///			}
///		}
/// }
/// ```
#[proc_macro]
pub fn construct(tokens: TokenStream) -> TokenStream {
    use construct::Construct;

    let construct = parse_macro_input!(tokens as Construct);
    let struct_info: Vec<_> = construct.into();

    quote::quote!(
        #(
            #struct_info
        )*
    )
    .into()
}

/// Used to generate a provider from Terraform schema.
///
/// # Examples
///
/// ```rust
/// terraform_bindgen_codegen::provider! {
///		"registry.terraform.io/hashicorp/kubernetes"
///		pub Provider {
///			config_path: String
///		}
/// }
/// ```
#[proc_macro]
pub fn provider(tokens: TokenStream) -> TokenStream {
    use construct::Provider;

    let provider = parse_macro_input!(tokens as Provider);
    let url = provider.provider.value();
    let version = provider.version.value();
    let mut struct_info: Vec<_> = provider.construct.into();
    struct_info[0].struct_type = StructType::Provider(url, version);

    quote::quote!(
        #(
            #struct_info
        )*
    )
    .into()
}
