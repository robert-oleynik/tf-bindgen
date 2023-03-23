use proc_macro::TokenStream;

mod resource;

/// Used to generate a builder chain for a specified Terraform resource or data source.
///
/// # Usage
///
/// ```rust
/// use tf_codegen::resource;
///
/// resource! {
///		scope,
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
