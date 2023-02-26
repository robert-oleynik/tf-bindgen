use std::collections::HashMap;

use heck::ToSnakeCase;
use quote::ToTokens;
use quote::__private::{Span, TokenStream};
use syn::{Attribute, Ident, Type};

/// All types of structs to create.
pub enum StructType {
    Construct,
    Provider(String, String),
    Inner,
}

/// Stores all information required to generate struct and associated builder.
pub struct StructInfo {
    pub struct_type: StructType,
    pub name: Ident,
    pub fields: HashMap<Ident, FieldInfo>,
}

/// Stores all information required to generate field and associated builder function.
pub struct FieldInfo {
    pub computed: bool,
    pub attributes: Vec<Attribute>,
    pub ty: Type,
}

impl StructInfo {
    pub fn struct_tokens(&self) -> TokenStream {
        let name = &self.name;
        let builder_ident = format!("{}Builder", self.name);
        let builder_ident = Ident::new(&builder_ident, Span::call_site());
        let fields = self
            .fields
            .iter()
            .filter(|(_, info)| !info.computed || info.is_optional())
            .map(|(ident, info)| (ident, &info.ty))
            .map(|(ident, ty)| quote::quote!(pub #ident: #ty));
        match &self.struct_type {
            StructType::Construct => quote::quote!(
                pub struct #name<C>
                where
                    C: ::tf_bindgen::Construct
                {
                    scope: ::std::rc::Rc<C>,
                    name: ::std::string::String,
                    #( #fields ),*
                }

                impl<C> #name<C>
                where
                    C: ::tf_bindgen::Construct,
                {
                    pub fn create(
                        scope: &::std::rc::Rc<C>,
                        name: impl ::std::convert::Into<::std::string::String>) -> #builder_ident<C>
                    {
                        #builder_ident::new(scope.clone(), name)
                    }
                }

                impl<C> ::tf_bindgen::Construct for #name<C>
                where
                    C: ::tf_bindgen::Construct,
                {
                    fn app(&self) -> ::tf_bindgen::app::App {
                        self.scope.app()
                    }

                    fn stack(&self) -> &str {
                        self.scope.stack()
                    }

                    fn name(&self) -> &str {
                        &self.name
                    }

                    fn path(&self) -> ::std::string::String {
                        format!("{}/{}", self.scope.path(), self.name)
                    }
                }
            ),
            StructType::Provider(_, _) => quote::quote!(
                pub struct #name<C>
                where
                    C: ::tf_bindgen::Construct
                {
                    scope: ::std::rc::Rc<C>,
                    #( #fields ),*
                }

                impl<C> #name<C>
                where
                    C: ::tf_bindgen::Construct,
                {
                    pub fn create(scope: &::std::rc::Rc<C>) -> #builder_ident<C> {
                        #builder_ident::new(scope.clone())
                    }
                }
            ),
            StructType::Inner => quote::quote!(
                #[derive(::std::clone::Clone, ::tf_bindgen::serde::Serialize)]
                #[serde(crate = "::tf_bindgen::serde")]
                pub struct #name {
                    #( #fields ),*
                }

                impl #name {
                    pub fn builder() -> #builder_ident {
                        #builder_ident::new()
                    }
                }
            ),
        }
    }

    pub fn builder_tokens(&self) -> TokenStream {
        let name = &self.name;
        let resource_type = self.name.to_string().to_snake_case();
        let builder_ident = format!("{}Builder", self.name);
        let builder_ident = Ident::new(&builder_ident, Span::call_site());
        let fields = self
            .fields
            .iter()
            .filter(|(_, info)| !info.computed || info.is_optional());
        let builder_fields = fields
            .clone()
            .map(|(ident, info)| (ident, info.as_builder_field_type()))
            .map(|(ident, ty)| quote::quote!(#ident: #ty));
        let builder_field_names = fields.clone().map(|(ident, _)| ident);
        let builder_setters = fields
            .clone()
            .filter(|(_, info)| !info.computed || info.is_optional())
            .map(FieldInfo::as_builder_setter);
        let builder_fields_assign = fields
            .clone()
            .filter(|(_, info)| !info.computed || info.is_optional())
            .map(FieldInfo::into_builder_assign);
        let builder_fields_value =
            fields
                .clone()
                .filter(|(_, info)| !info.computed)
                .map(|(ident, _)| {
                    let ident_str = ident.to_string();
                    quote::quote!(
                        let value = ::tf_bindgen::json::to_value(&self.#ident).unwrap();
                        config.insert(#ident_str.to_string(), value);
                    )
                });
        match &self.struct_type {
            StructType::Construct => quote::quote!(
                pub struct #builder_ident<C>
                where
                    C: ::tf_bindgen::Construct
                {
                    scope: ::std::rc::Rc<C>,
                    name: ::std::string::String,
                    #( #builder_fields ),*
                }

                impl<C> #builder_ident<C>
                where
                    C: ::tf_bindgen::Construct
                {
                    #( #builder_setters )*

                    pub fn new(
                        scope: ::std::rc::Rc<C>,
                        name: impl ::std::convert::Into<::std::string::String>
                    ) -> Self {
                        Self {
                            scope,
                            name: name.into(),
                            #( #builder_field_names: None ),*
                        }
                    }

                    pub fn build(&mut self) -> #name<C> {
                        use ::tf_bindgen::Construct;
                        let result = #name {
                            scope: self.scope.clone(),
                            name: self.name.clone(),
                            #( #builder_fields_assign ),*
                        };
                        let mut config = ::std::collections::HashMap::new();
                        #( #builder_fields_value )*
                        let resource = ::tf_bindgen::schema::document::Resource {
                            meta: ::tf_bindgen::schema::document::ResourceMeta {
                                metadata: ::tf_bindgen::schema::document::ResourceMetadata {
                                    path: result.path(),
                                    unique_id: result.name().to_string()
                                },
                            },
                            config
                        };
                        let app = result.app();
                        app.add_resource(result.stack(), #resource_type, result.name(), resource);
                        result
                    }
                }
            ),
            StructType::Provider(url, version) => {
                quote::quote!(
                    pub struct #builder_ident<C>
                    where
                        C: ::tf_bindgen::Construct
                    {
                        scope: ::std::rc::Rc<C>,
                        #( #builder_fields ),*
                    }

                    impl<C> #builder_ident<C>
                    where
                        C: ::tf_bindgen::Construct
                    {
                        #( #builder_setters )*

                        pub fn new(
                            scope: ::std::rc::Rc<C>,
                        ) -> Self {
                            Self {
                                scope,
                                #( #builder_field_names: None ),*
                            }
                        }

                        pub fn build(&mut self) -> #name<C> {
                            use ::tf_bindgen::Construct;
                            let result = #name {
                                scope: self.scope.clone(),
                                #( #builder_fields_assign ),*
                            };
                            let mut config = ::tf_bindgen::json::Map::new();
                            #( #builder_fields_value )*
                            let app = self.scope.app();
                            app.add_provider(self.scope.stack(), #url, #version, config);
                            result
                        }
                    }
                )
            }
            StructType::Inner => quote::quote!(
                pub struct #builder_ident {
                    #( #builder_fields ),*
                }

                impl #builder_ident {
                    #( #builder_setters )*

                    pub fn new() -> Self {
                        Self {
                            #( #builder_field_names: None ),*
                        }
                    }

                    pub fn build(&mut self) -> #name {
                        #name {
                            #( #builder_fields_assign ),*
                        }
                    }
                }
            ),
        }
    }
}

impl ToTokens for StructInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.struct_tokens());
        tokens.extend(self.builder_tokens());
    }
}

impl FieldInfo {
    /// Converts this field information into the builder's field type.
    pub fn as_builder_field_type(&self) -> TokenStream {
        let ty = &self.ty;
        if self.is_optional() {
            quote::quote!(#ty)
        } else {
            quote::quote!(::std::option::Option<#ty>)
        }
    }

    /// Converts a pair of field identifier and field information into a builder setter function.
    pub fn as_builder_setter((name, this): (&Ident, &FieldInfo)) -> TokenStream {
        let attributes = &this.attributes;
        let ty = &this.ty;
        if this.is_optional() {
            let ty = match ty {
                Type::Path(path) => path
                    .path
                    .segments
                    .iter()
                    .last()
                    .map(|seg| match &seg.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            args.args.iter().next().unwrap()
                        }
                        _ => unimplemented!(),
                    })
                    .unwrap(),
                _ => unimplemented!(),
            };
            quote::quote!(
                #( #attributes )*
                pub fn #name(&mut self, value: impl Into<#ty>) -> &mut Self {
                    self.#name = Some(value.into());
                    self
                }
            )
        } else {
            quote::quote!(
                #( #attributes )*
                pub fn #name(&mut self, value: impl Into<#ty>) -> &mut Self {
                    self.#name = Some(value.into());
                    self
                }
            )
        }
    }

    /// Converts a pair of field identifier and field information into an assigned. Will implement
    /// option unwraps if necessary.
    pub fn into_builder_assign((name, this): (&Ident, &FieldInfo)) -> TokenStream {
        if this.is_optional() {
            quote::quote!(#name: self.#name.clone())
        } else {
            let message = format!("expected missing field `{name}`");
            quote::quote!(#name: self.#name.clone().expect(#message))
        }
    }

    /// Returns true if `self.ty` is of type [`::std::option::Option`].
    pub fn is_optional(&self) -> bool {
        match &self.ty {
            Type::Path(path) => path
                .path
                .segments
                .iter()
                .last()
                .iter()
                .any(|seg| seg.ident == "Option"),
            _ => false,
        }
    }
}
