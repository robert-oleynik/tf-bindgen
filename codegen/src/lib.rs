use proc_macro::TokenStream;
use syn::parse_macro_input;

mod construct;

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
    todo!()
}
