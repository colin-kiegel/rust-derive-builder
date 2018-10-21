#![cfg_attr(feature = "cargo-clippy", allow(useless_let_if_seq))]
use quote::{ToTokens, TokenStreamExt};
use proc_macro2::{Span, TokenStream};
use syn;

use Bindings;
use BuilderPattern;
use DeprecationNotes;

/// Setter for the struct fields in the build method, implementing
/// `quote::ToTokens`.
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust
/// # extern crate proc_macro2;
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate syn;
/// # #[macro_use]
/// # extern crate derive_builder_core;
/// # use derive_builder_core::{Setter, BuilderPattern};
/// # fn main() {
/// #     let mut setter = default_setter!();
/// #     setter.pattern = BuilderPattern::Mutable;
/// #
/// #     assert_eq!(quote!(#setter).to_string(), quote!(
/// # #[allow(unused_mut)]
/// pub fn foo(&mut self, value: Foo) -> &mut Self {
///     let mut new = self;
///     new.foo = ::std::option::Option::Some(value);
///     new
/// }
/// #     ).to_string());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Setter<'a> {
    /// Enables code generation for this setter fn.
    pub enabled: bool,
    /// Enables code generation for the `try_` variant of this setter fn.
    pub try_setter: bool,
    /// Visibility of the setter, e.g. `syn::Visibility::Public`.
    pub visibility: syn::Visibility,
    /// How the setter method takes and returns `self` (e.g. mutably).
    pub pattern: BuilderPattern,
    /// Attributes which will be attached to this setter fn.
    pub attrs: &'a [syn::Attribute],
    /// Name of this setter fn.
    pub ident: syn::Ident,
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub field_type: &'a syn::Type,
    /// Make the setter generic over `Into<T>`, where `T` is the field type.
    pub generic_into: bool,
    /// Emit deprecation notes to the user.
    pub deprecation_notes: &'a DeprecationNotes,
    /// Bindings to libstd or libcore.
    pub bindings: Bindings,
}

impl<'a> ToTokens for Setter<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.enabled {
            trace!("Deriving setter for `{}`.", self.field_ident);
            let ty = self.field_type;
            let pattern = self.pattern;
            let vis = &self.visibility;
            let field_ident = self.field_ident;
            let ident = &self.ident;
            let attrs = self.attrs;
            let deprecation_notes = self.deprecation_notes;
            let clone = self.bindings.clone_trait();
            let option = self.bindings.option_ty();
            let into = self.bindings.into_trait();

            let self_param: TokenStream;
            let return_ty: TokenStream;
            let self_into_return_ty: TokenStream;

            match pattern {
                BuilderPattern::Owned => {
                    self_param = quote!(self);
                    return_ty = quote!(Self);
                    self_into_return_ty = quote!(self);
                }
                BuilderPattern::Mutable => {
                    self_param = quote!(&mut self);
                    return_ty = quote!(&mut Self);
                    self_into_return_ty = quote!(self);
                }
                BuilderPattern::Immutable => {
                    self_param = quote!(&self);
                    return_ty = quote!(Self);
                    self_into_return_ty = quote!(#clone::clone(self));
                }
            };

            let ty_params: TokenStream;
            let param_ty: TokenStream;
            let into_value: TokenStream;

            if self.generic_into {
                ty_params = quote!(<VALUE: #into<#ty>>);
                param_ty = quote!(VALUE);
                into_value = quote!(value.into());
            } else {
                ty_params = quote!();
                param_ty = quote!(#ty);
                into_value = quote!(value);
            }

            tokens.append_all(quote!(
                #(#attrs)*
                #[allow(unused_mut)]
                #vis fn #ident #ty_params (#self_param, value: #param_ty)
                    -> #return_ty
                {
                    #deprecation_notes
                    let mut new = #self_into_return_ty;
                    new.#field_ident = #option::Some(#into_value);
                    new
            }));

            if self.try_setter {
                let try_into = self.bindings.try_into_trait();
                let try_ty_params = quote!(<VALUE: #try_into<#ty>>);
                let try_ident = syn::Ident::new(&format!("try_{}", ident), Span::call_site());
                let result = self.bindings.result_ty();

                tokens.append_all(quote!(
                    #(#attrs)*
                    #vis fn #try_ident #try_ty_params (#self_param, value: VALUE)
                        -> #result<#return_ty, VALUE::Error>
                    {
                        let converted : #ty = value.try_into()?;
                        let mut new = #self_into_return_ty;
                        new.#field_ident = #option::Some(converted);
                        Ok(new)
                }));
            } else {
                trace!("Skipping try_setter for `{}`.", self.field_ident);
            }
        } else {
            trace!("Skipping setter for `{}`.", self.field_ident);
        }
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_setter {
    () => {
        Setter {
            enabled: true,
            try_setter: false,
            visibility: syn::parse_str("pub").unwrap(),
            pattern: BuilderPattern::Mutable,
            attrs: &vec![],
            ident: syn::Ident::new("foo", ::proc_macro2::Span::call_site()),
            field_ident: &syn::Ident::new("foo", ::proc_macro2::Span::call_site()),
            field_type: &syn::parse_str("Foo").unwrap(),
            generic_into: false,
            deprecation_notes: &Default::default(),
            bindings: Default::default(),
        };
    };
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn immutable() {
        let mut setter = default_setter!();
        setter.pattern = BuilderPattern::Immutable;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo(&self, value: Foo) -> Self {
                let mut new = ::std::clone::Clone::clone(self);
                new.foo = ::std::option::Option::Some(value);
                new
            }
        ).to_string()
        );
    }

    #[test]
    fn mutable() {
        let mut setter = default_setter!();
        setter.pattern = BuilderPattern::Mutable;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo(&mut self, value: Foo) -> &mut Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value);
                new
            }
        ).to_string()
        );
    }

    #[test]
    fn owned() {
        let mut setter = default_setter!();
        setter.pattern = BuilderPattern::Owned;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo(self, value: Foo) -> Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value);
                new
            }
        ).to_string()
        );
    }

    #[test]
    fn private() {
        let vis = syn::Visibility::Inherited;

        let mut setter = default_setter!();
        setter.visibility = vis;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            fn foo(&mut self, value: Foo) -> &mut Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value);
                new
            }
        ).to_string()
        );
    }

    #[test]
    fn generic() {
        let mut setter = default_setter!();
        setter.generic_into = true;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo <VALUE: ::std::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value.into());
                new
            }
        ).to_string()
        );
    }

    // including try_setter
    #[test]
    fn full() {
        //named!(outer_attrs -> Vec<syn::Attribute>, many0!(syn::Attribute::parse_outer));
        //let attrs = outer_attrs.parse_str("#[some_attr]").unwrap();
        let attrs: Vec<syn::Attribute> = vec![parse_quote!(#[some_attr])];

        let mut deprecated = DeprecationNotes::default();
        deprecated.push("Some example.".to_string());

        let mut setter = default_setter!();
        setter.attrs = attrs.as_slice();
        setter.generic_into = true;
        setter.deprecation_notes = &deprecated;
        setter.try_setter = true;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[some_attr]
            #[allow(unused_mut)]
            pub fn foo <VALUE: ::std::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
                #deprecated
                let mut new = self;
                new.foo = ::std::option::Option::Some(value.into());
                new
            }

            #[some_attr]
            pub fn try_foo<VALUE: ::std::convert::TryInto<Foo>>(&mut self, value: VALUE)
                -> ::std::result::Result<&mut Self, VALUE::Error> {
                let converted : Foo = value.try_into()?;
                let mut new = self;
                new.foo = ::std::option::Option::Some(converted);
                Ok(new)
            }
        ).to_string()
        );
    }

    #[test]
    fn no_std() {
        let mut setter = default_setter!();
        setter.bindings.no_std = true;
        setter.pattern = BuilderPattern::Immutable;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo(&self, value: Foo) -> Self {
                let mut new = ::core::clone::Clone::clone(self);
                new.foo = ::core::option::Option::Some(value);
                new
            }
        ).to_string()
        );
    }

    #[test]
    fn no_std_generic() {
        let mut setter = default_setter!();
        setter.bindings.no_std = true;
        setter.generic_into = true;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo <VALUE: ::core::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
                let mut new = self;
                new.foo = ::core::option::Option::Some(value.into());
                new
            }
        ).to_string()
        );
    }

    #[test]
    fn setter_disabled() {
        let mut setter = default_setter!();
        setter.enabled = false;

        assert_eq!(quote!(#setter).to_string(), quote!().to_string());
    }

    #[test]
    fn try_setter() {
        let mut setter: Setter = default_setter!();
        setter.pattern = BuilderPattern::Mutable;
        setter.try_setter = true;

        assert_eq!(
            quote!(#setter).to_string(),
            quote!(
            #[allow(unused_mut)]
            pub fn foo(&mut self, value: Foo) -> &mut Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value);
                new
            }

            pub fn try_foo<VALUE: ::std::convert::TryInto<Foo>>(&mut self, value: VALUE)
                -> ::std::result::Result<&mut Self, VALUE::Error> {
                let converted : Foo = value.try_into()?;
                let mut new = self;
                new.foo = ::std::option::Option::Some(converted);
                Ok(new)
            }
        ).to_string()
        );
    }
}
