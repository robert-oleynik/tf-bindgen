use std::collections::HashMap;

use quote::ToTokens;
use quote::__private::{Span, TokenStream};
use syn::{Attribute, Ident, Type};

/// All types of structs to create.
pub enum StructType {
    Construct,
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
            .filter(|(_, info)| !info.computed)
            .map(|(ident, info)| (ident, &info.ty))
            .map(|(ident, ty)| quote::quote!(pub #ident: #ty));
        match &self.struct_type {
            StructType::Construct => quote::quote!(
                pub struct #name<C>
                where
                    C: ::terraform_bindgen_core::Construct
                {
                    scope: ::std::rc::Rc<C>,
                    name: ::std::string::String,
                    #( #fields ),*
                }

                impl<C> #name<C>
                where
                    C: ::terraform_bindgen_core::Construct,
                {
                    pub fn create(
                        scope: impl ::std::convert::AsRef<::std::rc::Rc<C>>,
                        name: impl ::std::convert::Into<::std::string::String>) -> #builder_ident<C>
                    {
                        #builder_ident::new(scope.as_ref().clone(), name)
                    }
                }

                impl<C> ::terraform_bindgen_core::Construct for #name<C>
                where
                    C: ::terraform_bindgen_core::Construct,
                {
                    fn app(&self) -> ::std::rc::Rc<::terraform_bindgen_core::app::App> {
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
            StructType::Inner => quote::quote!(
                #[derive(Clone)]
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
        let builder_ident = format!("{}Builder", self.name);
        let builder_ident = Ident::new(&builder_ident, Span::call_site());
        let builder_fields = self
            .fields
            .iter()
            .filter(|(_, info)| !info.computed)
            .map(|(ident, info)| (ident, info.as_builder_field_type()))
            .map(|(ident, ty)| quote::quote!(#ident: #ty));
        let builder_field_names = self
            .fields
            .iter()
            .filter(|(_, info)| !info.computed)
            .map(|(ident, _)| ident);
        let builder_setters = self
            .fields
            .iter()
            .filter(|(_, info)| !info.computed)
            .map(FieldInfo::as_builder_setter);
        let builder_fields_assign = self
            .fields
            .iter()
            .filter(|(_, info)| !info.computed)
            .map(FieldInfo::into_builder_assign);
        match &self.struct_type {
            StructType::Construct => quote::quote!(
                pub struct #builder_ident<C>
                where
                    C: ::terraform_bindgen_core::Construct
                {
                    scope: ::std::rc::Rc<C>,
                    name: ::std::string::String,
                    #( #builder_fields ),*
                }

                impl<C> #builder_ident<C>
                where
                    C: ::terraform_bindgen_core::Construct
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
                        let result = #name {
                            scope: self.scope.clone(),
                            name: self.name.clone(),
                            #( #builder_fields_assign ),*
                        };
                        todo!()
                    }
                }
            ),
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
            quote::quote!(
                #( #attributes )*
                pub fn #name(&mut self, value: impl Into<#ty>) -> &mut Self {
                    self.#name = value.into();
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