use construct::Construct;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::DeriveInput;

mod construct;
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
///		#[construct(scope)]
///		__m_scope: Rc<dyn ::tf_bindgen::Construct>,
///		#[construct(id)]
///		__m_name: String
/// }
/// ```
#[proc_macro_derive(Construct, attributes(construct))]
pub fn construct_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let construct = Construct::from_derive_input(&input).unwrap();
    quote::quote!(#construct).into()
}
