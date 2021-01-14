use crate::macro_options::FieldWithDefaults;
use doc_comment_from;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn;
use Block;
use BuilderPattern;
use DEFAULT_STRUCT_NAME;

/// Initializer for the struct fields in the build method, implementing
/// `quote::ToTokens`.
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust,ignore
/// # extern crate proc_macro2;
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate syn;
/// # #[macro_use(default_build_method)]
/// # extern crate derive_builder_core;
/// # use derive_builder_core::{BuildMethod, BuilderPattern};
/// # fn main() {
/// #    let build_method = default_build_method!();
/// #
/// #    assert_eq!(quote!(#build_method).to_string(), quote!(
/// pub fn build(&self) -> ::derive_builder::export::core::result::Result<Foo, FooBuilderError> {
///     Ok(Foo {
///         foo: self.foo,
///     })
/// }
/// #    ).to_string());
/// # }
/// ```
#[derive(Debug)]
pub struct BuildMethod<'a> {
    /// Enables code generation for this build method.
    pub enabled: bool,
    /// Name of this build fn.
    pub ident: &'a syn::Ident,
    /// Visibility of the build method, e.g. `syn::Visibility::Public`.
    pub visibility: syn::Visibility,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub pattern: BuilderPattern,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub target_ty: &'a syn::Ident,
    /// Type parameters and lifetimes attached to this builder struct.
    pub target_ty_generics: Option<syn::TypeGenerics<'a>>,
    /// Type of error.
    pub error_ty: syn::Ident,
    /// Fields for the target type.
    pub fields: Vec<FieldWithDefaults<'a>>,
    /// Doc-comment of the builder struct.
    pub doc_comment: Option<syn::Attribute>,
    /// Default value for the whole struct.
    ///
    /// This will be in scope for all initializers as `__default`.
    pub default_struct: Option<Block>,
    /// Validation function with signature `&FooBuilder -> Result<(), String>`
    /// to call before the macro-provided struct buildout.
    pub validate_fn: Option<&'a syn::Path>,
}

impl<'a> ToTokens for BuildMethod<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let vis = &self.visibility;
        let target_ty = &self.target_ty;
        let target_ty_generics = &self.target_ty_generics;
        let error_ty = &self.error_ty;
        let error_constructor = quote!(#error_ty::UninitializedField);
        let initializers = &self
            .fields
            .iter()
            .map(|field| field.as_initializer(&error_constructor))
            .collect::<Vec<_>>();
        let self_param = match self.pattern {
            BuilderPattern::Owned => quote!(self),
            BuilderPattern::Mutable | BuilderPattern::Immutable => quote!(&self),
        };
        let doc_comment = &self.doc_comment;
        let default_struct = self.default_struct.as_ref().map(|default_expr| {
            let ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
            quote!(let #ident: #target_ty #target_ty_generics = #default_expr;)
        });
        let validate_fn = self.validate_fn.as_ref().map(|vfn| quote!(#vfn(&self)?;));

        if self.enabled {
            tokens.append_all(quote!(
                #doc_comment
                #vis fn #ident(#self_param)
                    -> ::derive_builder::export::core::result::Result<#target_ty #target_ty_generics, #error_ty>
                {
                    #validate_fn
                    #default_struct
                    Ok(#target_ty {
                        #(#initializers)*
                    })
                }
            ))
        }
    }
}

impl<'a> BuildMethod<'a> {
    /// Set a doc-comment for this item.
    pub fn doc_comment(&mut self, s: String) -> &mut Self {
        self.doc_comment = Some(doc_comment_from(s));
        self
    }

    /// Set fields for this item.
    pub fn fields(&mut self, fields: &[FieldWithDefaults<'a>]) -> &mut Self {
        self.fields = fields.to_vec();
        self
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_build_method {
    () => {
        BuildMethod {
            enabled: true,
            ident: &syn::Ident::new("build", ::proc_macro2::Span::call_site()),
            visibility: syn::parse_str("pub").unwrap(),
            pattern: BuilderPattern::Mutable,
            target_ty: &syn::Ident::new("Foo", ::proc_macro2::Span::call_site()),
            target_ty_generics: None,
            error_ty: syn::Ident::new("FooBuilderError", ::proc_macro2::Span::call_site()),
            fields: vec![],
            doc_comment: None,
            default_struct: None,
            validate_fn: None,
        }
    };
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn std() {
        let build_method = default_build_method!();

        #[rustfmt::skip]
        assert_eq!(
            quote!(#build_method).to_string(),
            quote!(
                pub fn build(&self) -> ::derive_builder::export::core::result::Result<Foo, FooBuilderError> {
                    Ok(Foo {})
                }
            )
            .to_string()
        );
    }

    #[test]
    fn default_struct() {
        let mut build_method = default_build_method!();
        build_method.default_struct = Some("Default::default()".parse().unwrap());

        #[rustfmt::skip]
        assert_eq!(
            quote!(#build_method).to_string(),
            quote!(
                pub fn build(&self) -> ::derive_builder::export::core::result::Result<Foo, FooBuilderError> {
                    let __default: Foo = { Default::default() };
                    Ok(Foo {})
                }
            )
            .to_string()
        );
    }

    #[test]
    fn skip() {
        let mut build_method = default_build_method!();
        build_method.enabled = false;
        build_method.enabled = false;

        assert_eq!(quote!(#build_method).to_string(), quote!().to_string());
    }

    #[test]
    fn rename() {
        let ident = syn::Ident::new("finish", Span::call_site());
        let mut build_method: BuildMethod = default_build_method!();
        build_method.ident = &ident;

        #[rustfmt::skip]
        assert_eq!(
            quote!(#build_method).to_string(),
            quote!(
                pub fn finish(&self) -> ::derive_builder::export::core::result::Result<Foo, FooBuilderError> {
                    Ok(Foo {})
                }
            )
            .to_string()
        );
    }

    #[test]
    fn validation() {
        let validate_path: syn::Path = syn::parse_str("IpsumBuilder::validate")
            .expect("Statically-entered path should be valid");

        let mut build_method: BuildMethod = default_build_method!();
        build_method.validate_fn = Some(&validate_path);

        #[rustfmt::skip]
        assert_eq!(
            quote!(#build_method).to_string(),
            quote!(
                pub fn build(&self) -> ::derive_builder::export::core::result::Result<Foo, FooBuilderError> {
                    IpsumBuilder::validate(&self)?;
                    Ok(Foo {})
                }
            )
            .to_string()
        );
    }
}
