use std::borrow::Cow;

use crate::wrap_expression_in_some;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn;

/// Field for the builder struct, implementing `quote::ToTokens`.
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust,ignore
/// # extern crate proc_macro2;
/// # #[macro_use]
/// # extern crate quote;
/// # #[macro_use]
/// # extern crate syn;
/// # #[macro_use]
/// # extern crate derive_builder_core;
/// # use derive_builder_core::{BuilderField, BuilderPattern};
/// # fn main() {
/// #    let attrs = vec![parse_quote!(#[some_attr])];
/// #    let mut field = default_builder_field!();
/// #    field.attrs = attrs.as_slice();
/// #
/// #    assert_eq!(quote!(#field).to_string(), quote!(
/// #[some_attr] pub foo: ::derive_builder::export::core::option::Option<String>,
/// #    ).to_string());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuilderField<'a> {
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Type of the target field.
    pub field_type: BuilderFieldType<'a>,
    /// Visibility of this builder field, e.g. `syn::Visibility::Public`.
    pub field_visibility: Cow<'a, syn::Visibility>,
    /// Attributes which will be attached to this builder field.
    pub attrs: &'a [syn::Attribute],
}

/// The type of a field in the builder struct
#[derive(Debug, Clone)]
pub enum BuilderFieldType<'a> {
    /// The corresonding builder field will be `Option<field_type>`.
    Optional(&'a syn::Type),
    /// The corresponding builder field will be just this type
    Precise(&'a syn::Type),
    /// The corresponding builder field will be a PhantomData
    ///
    /// We do this if if the field is disabled
    /// to hack around issues with unused generic type parameters - at
    /// least for now.
    Phantom(&'a syn::Type),
}

impl<'a> BuilderFieldType<'a> {
    /// Some call sites want the target field type
    pub fn target_type(&'a self) -> &'a syn::Type {
        match self {
            BuilderFieldType::Optional(ty) => ty,
            BuilderFieldType::Precise(ty) => ty,
            BuilderFieldType::Phantom(ty) => ty,
        }
    }

    /// Returns expression wrapping `bare_value` in Some, if appropriate
    pub fn wrap_some(&'a self, bare_value: TokenStream) -> TokenStream {
        match self {
            BuilderFieldType::Optional(_) => wrap_expression_in_some(bare_value),
            BuilderFieldType::Precise(_) => bare_value,
            BuilderFieldType::Phantom(_) => panic!("setter_enabled but BFT::PHantom"),
        }
    }
}

impl<'a> ToTokens for BuilderFieldType<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BuilderFieldType::Optional(ty) => tokens.append_all(quote!(
                ::derive_builder::export::core::option::Option<#ty>
            )),
            BuilderFieldType::Precise(ty) => ty.to_tokens(tokens),
            BuilderFieldType::Phantom(ty) => tokens.append_all(quote!(
                ::derive_builder::export::core::marker::PhantomData<#ty>
            )),
        }
    }
}

impl<'a> ToTokens for BuilderField<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.field_ident;
        let vis = &self.field_visibility;
        let ty = &self.field_type;
        let attrs = self.attrs;
        tokens.append_all(quote!(
            #(#attrs)* #vis #ident: #ty,
        ));
        // K let ty = self.field_type.target_type();
    }
}

impl<'a> BuilderField<'a> {
    /// Emits a struct field initializer that initializes the field to `Default::default`.
    pub fn default_initializer_tokens(&self) -> TokenStream {
        let ident = self.field_ident;
        quote! { #ident : ::derive_builder::export::core::default::Default::default(), }
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[cfg(test)] // This contains a Box::leak, so is suitable only for tests
#[doc(hidden)]
#[macro_export]
macro_rules! default_builder_field {
    () => {{
        BuilderField {
            field_ident: &syn::Ident::new("foo", ::proc_macro2::Span::call_site()),
            field_type: BuilderFieldType::Optional(Box::leak(Box::new(parse_quote!(String)))),
            field_visibility: ::std::borrow::Cow::Owned(parse_quote!(pub)),
            attrs: &[parse_quote!(#[some_attr])],
        }
    }};
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn setter_enabled() {
        let field = default_builder_field!();

        assert_eq!(
            quote!(#field).to_string(),
            quote!(
                #[some_attr] pub foo: ::derive_builder::export::core::option::Option<String>,
            )
            .to_string()
        );
    }

    #[test]
    fn setter_disabled() {
        let mut field = default_builder_field!();
        field.field_visibility = syn::Visibility::Inherited;
        field.field_type = match field.field_type {
            BuilderFieldType::Optional(ty) => BuilderFieldType::Phantom(ty),
            _ => panic!(),
        };

        assert_eq!(
            quote!(#field).to_string(),
            quote!(
                #[some_attr]
                foo: ::derive_builder::export::core::marker::PhantomData<String>,
            )
            .to_string()
        );
    }

    #[test]
    fn private_field() {
        let private = Cow::Owned(syn::Visibility::Inherited);
        let mut field = default_builder_field!();
        field.field_visibility = private;

        assert_eq!(
            quote!(#field).to_string(),
            quote!(
                #[some_attr]
                foo: ::derive_builder::export::core::option::Option<String>,
            )
            .to_string()
        );
    }
}
