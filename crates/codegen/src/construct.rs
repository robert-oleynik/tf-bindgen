use darling::ast::Data;
use darling::{FromDeriveInput, FromField, FromMeta, ToTokens};
use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments};

#[derive(FromDeriveInput)]
#[darling(
    attributes(construct),
    forward_attrs(allow, doc, cfg),
    supports(struct_named)
)]
pub struct Construct {
    #[darling(default)]
    builder: bool,
    #[darling(rename = "crate")]
    crate_path: Option<String>,
    ident: syn::Ident,
    data: Data<darling::util::Ignored, ConstructField>,
}

#[derive(FromField)]
#[darling(attributes(construct))]
pub struct ConstructField {
    #[darling(default)]
    id: bool,
    #[darling(default)]
    scope: bool,
    #[darling(default)]
    skip: bool,
    ty: syn::Type,
    ident: Option<syn::Ident>,
    #[darling(default)]
    setter: Setter,
}

#[derive(Clone)]
pub enum SetterMode {
    Into,
    IntoValue,
    IntoValueSet,
    IntoValueMap,
    IntoValueList,
    Default,
}

#[derive(FromMeta, Clone, Default)]
pub struct Setter {
    #[darling(default)]
    into: bool,
    #[darling(default)]
    into_value: bool,
    #[darling(default)]
    into_value_set: bool,
    #[darling(default)]
    into_value_map: bool,
    #[darling(default)]
    into_value_list: bool,
}

impl ToTokens for Construct {
    fn to_tokens(&self, tokens: &mut syn::__private::TokenStream2) {
        let base_path = self
            .crate_path
            .clone()
            .unwrap_or(String::from("::tf_bindgen"));
        let base_path: syn::Path = syn::parse_str(&base_path).unwrap();
        let ident = &self.ident;
        let fields = self.data.as_ref().take_struct().unwrap().fields;
        let id_field = fields
            .iter()
            .find(|field| field.id)
            .expect("Missing id field. Use `construct(id)` to select a field.");
        let scope_field = fields
            .iter()
            .find(|field| field.scope)
            .expect("Missing id field. Use `construct(scope)` to select a field.");
        let id_field_ident = &id_field.ident;
        let scope_field_ident = &scope_field.ident;
        let extra = if self.builder {
            let builder = syn::Ident::new(&format!("{}Builder", self.ident), self.ident.span());
            let fields_iter = fields
                .iter()
                .filter(|field| !field.id && !field.scope && !field.skip)
                .map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    quote::quote!( #ident: Option<#ty> )
                });
            let setter = fields
                .iter()
                .filter(|field| !field.id && !field.scope && !field.skip)
                .map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let setter_mode: SetterMode = field.setter.clone().into();
                    let ty = unwrap_type(&field.ty, &setter_mode);
                    let impl_type = match &setter_mode {
                        SetterMode::Into => quote::quote!(::std::convert::Into<#ty>),
                        SetterMode::IntoValue => quote::quote!(#base_path::value::IntoValue<#ty>),
                        SetterMode::IntoValueSet => quote::quote!(
                            #base_path::value::IntoValueSet<#ty>
                        ),
                        SetterMode::IntoValueMap => quote::quote!(
                            #base_path::value::IntoValueMap<#ty>
                        ),
                        SetterMode::IntoValueList => quote::quote!(
                            #base_path::value::IntoValueList<#ty>
                        ),
                        SetterMode::Default => quote::quote!(#ty),
                    };
                    let convert = match &setter_mode {
                        SetterMode::Into => quote::quote!(value.into()),
                        SetterMode::IntoValue => quote::quote!(value.into_value()),
                        SetterMode::IntoValueList => quote::quote!(value.into_value_list()),
                        SetterMode::IntoValueSet => quote::quote!(value.into_value_set()),
                        SetterMode::IntoValueMap => quote::quote!(value.into_value_map()),
                        SetterMode::Default => quote::quote!(value),
                    };
                    quote::quote!(
                        pub fn #ident(&mut self, value: impl #impl_type) -> &mut Self {
                            #[allow(unused_imports)]
                            use #base_path::value::*;
                            self.#ident = Some(#convert);
                            self
                        }
                    )
                });
            let fields_ident = fields
                .iter()
                .filter(|field| !field.id && !field.scope && !field.skip)
                .map(|field| &field.ident);
            let id_ty = &id_field.ty;
            let scope_ty = &scope_field.ty;
            quote::quote!(
                pub struct #builder {
                    #scope_field_ident: #scope_ty,
                    #id_field_ident: #id_ty,
                    #( #fields_iter ),*
                }
                impl #ident {
                    pub fn create<C: #base_path::Scope + 'static>(
                        scope: &::std::rc::Rc<C>,
                        name: impl ::std::convert::Into<#id_ty>
                    ) -> #builder {
                        #builder {
                            #scope_field_ident: scope.clone(),
                            #id_field_ident: name.into(),
                            #( #fields_ident: None ),*
                        }
                    }
                }
                impl #builder {
                    #( #setter )*
                }
            )
        } else {
            quote::quote!()
        };
        tokens.extend(quote::quote!(
            impl #base_path::Scope for #ident {
                fn stack(&self) -> #base_path::Stack {
                    self.#scope_field_ident.stack()
                }
                fn path(&self) -> #base_path::Path {
                    let mut path = self.#scope_field_ident.path();
                    path.push(&self.#id_field_ident);
                    path
                }
            }
            #extra
        ));
    }
}

impl From<Setter> for SetterMode {
    fn from(value: Setter) -> Self {
        match value {
            Setter { into: true, .. } => SetterMode::Into,
            Setter {
                into_value: true, ..
            } => SetterMode::IntoValue,
            Setter {
                into_value_list: true,
                ..
            } => SetterMode::IntoValueList,
            Setter {
                into_value_set: true,
                ..
            } => SetterMode::IntoValueSet,
            Setter {
                into_value_map: true,
                ..
            } => SetterMode::IntoValueMap,
            _ => SetterMode::Default,
        }
    }
}

pub fn unwrap_type(ty: &syn::Type, setter: &SetterMode) -> syn::Type {
    match setter {
        SetterMode::IntoValue => match ty {
            syn::Type::Path(path) if is_ident(&path.path, "Value") => {
                get_nth_generic_argument(&path.path, 0)
            }
            _ => unimplemented!("Cannot use IntoValue for non Value types."),
        },
        SetterMode::IntoValueList => match ty {
            syn::Type::Path(path) if is_ident(&path.path, "Vec") => {
                let generic = get_nth_generic_argument(&path.path, 0);
                unwrap_type(&generic, &SetterMode::IntoValue)
            }
            _ => unimplemented!("Cannot use IntoValue for non Value types."),
        },
        SetterMode::IntoValueSet => match ty {
            syn::Type::Path(path) if is_ident(&path.path, "HashSet") => {
                let generic = get_nth_generic_argument(&path.path, 0);
                unwrap_type(&generic, &SetterMode::IntoValue)
            }
            _ => unimplemented!("Cannot use IntoValue for non Value types."),
        },
        SetterMode::IntoValueMap => match ty {
            syn::Type::Path(path) if is_ident(&path.path, "HashMap") => {
                let generic = get_nth_generic_argument(&path.path, 1);
                unwrap_type(&generic, &SetterMode::IntoValueMap)
            }
            _ => unimplemented!("Cannot use IntoValue for non Value types."),
        },
        _ => ty.clone(),
    }
}

pub fn get_nth_generic_argument(path: &syn::Path, n: usize) -> syn::Type {
    path.segments
        .last()
        .map(|segment| match &segment.arguments {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => args
                .iter()
                .filter_map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
                .skip(n)
                .next()
                .expect("missing generic argument"),
            _ => unimplemented!("Missing generic argument"),
        })
        .unwrap()
        .clone()
}

pub fn is_ident(path: &syn::Path, ident: &str) -> bool {
    if let Some(segment) = path.segments.iter().next() {
        return segment.ident == ident;
    }
    false
}
