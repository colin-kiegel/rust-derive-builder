use quote::{Tokens, ToTokens};
use syn;
use BuilderPattern;
use DeprecationNotes;

/// Setter for the struct fields in the build method, implementing `quote::ToTokens`.
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust
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
/// #     assert_eq!(quote!(#setter), quote!(
/// pub fn foo <VALUE: ::std::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
///     let mut new = self;
///     new.foo = ::std::option::Option::Some(value.into());
///     new
/// }
/// #     ));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Setter<'a> {
    /// Enables code generation for this setter fn.
    pub enabled: bool,
    /// Visibility of the setter, e.g. `syn::Visibility::Public`.
    pub visibility: &'a syn::Visibility,
    /// How the setter method takes and returns `self` (e.g. mutably).
    pub pattern: BuilderPattern,
    /// Attributes which will be attached to this setter fn.
    pub attrs: &'a [syn::Attribute],
    /// Name of this setter fn.
    pub ident: &'a syn::Ident,
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub field_type: &'a syn::Ty,
    /// Emit deprecation notes to the user.
    pub deprecation_notes: &'a DeprecationNotes,
}

impl<'a> ToTokens for Setter<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if self.enabled {
            trace!("Deriving setter for `{}`.", self.field_ident);
            let ty = self.field_type;
            let pattern = self.pattern;
            let vis = self.visibility;
            let field_ident = self.field_ident;
            let ident = self.ident;
            let attrs = self.attrs;
            let deprecation_notes = self.deprecation_notes;

            let self_param: Tokens;
            let return_ty: Tokens;
            let self_into_return_ty: Tokens;

            match pattern {
                BuilderPattern::Owned => {
                    self_param = quote!(self);
                    return_ty = quote!(Self);
                    self_into_return_ty = quote!(self);
                },
                BuilderPattern::Mutable => {
                    self_param = quote!(&mut self);
                    return_ty = quote!(&mut Self);
                    self_into_return_ty = quote!(self);
                },
                BuilderPattern::Immutable => {
                    self_param = quote!(&self);
                    return_ty = quote!(Self);
                    self_into_return_ty = quote!(::std::clone::Clone::clone(self));
                }
            };

            tokens.append(quote!(
                #(#attrs)*
                #vis fn #ident<VALUE: ::std::convert::Into<#ty>>(#self_param, value: VALUE)
                    -> #return_ty
                {
                    #deprecation_notes
                    let mut new = #self_into_return_ty;
                    new.#field_ident = ::std::option::Option::Some(value.into());
                    new
            }));
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
            visibility: &syn::Visibility::Public,
            pattern: BuilderPattern::Mutable,
            attrs: &vec![],
            ident: &syn::Ident::new("foo"),
            field_ident: &syn::Ident::new("foo"),
            field_type: &syn::parse_type("Foo").unwrap(),
            deprecation_notes: &Default::default(),
        };
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn immutable() {
        let mut setter = default_setter!();
        setter.pattern = BuilderPattern::Immutable;

        assert_eq!(quote!(#setter), quote!(
            pub fn foo <VALUE: ::std::convert::Into<Foo>>(&self, value: VALUE) -> Self {
                let mut new = ::std::clone::Clone::clone(self);
                new.foo = ::std::option::Option::Some(value.into());
                new
            }
        ));
    }

    #[test]
    fn mutable() {
        let mut setter = default_setter!();
        setter.pattern = BuilderPattern::Mutable;

        assert_eq!(quote!(#setter), quote!(
            pub fn foo <VALUE: ::std::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value.into());
                new
            }
        ));
    }

    #[test]
    fn owned() {
        let mut setter = default_setter!();
        setter.pattern = BuilderPattern::Owned;

        assert_eq!(quote!(#setter), quote!(
            pub fn foo <VALUE: ::std::convert::Into<Foo>>(self, value: VALUE) -> Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value.into());
                new
            }
        ));
    }

    #[test]
    fn private() {
        let vis = syn::Visibility::Inherited;

        let mut setter = default_setter!();
        setter.visibility = &vis;

        assert_eq!(quote!(#setter), quote!(
            fn foo <VALUE: ::std::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
                let mut new = self;
                new.foo = ::std::option::Option::Some(value.into());
                new
            }
        ));
    }

    // including
    #[test]
    fn full() {
        let attrs = vec![syn::parse_outer_attr("#[some_attr]").unwrap()];

        let mut deprecated = DeprecationNotes::default();
        deprecated.push(String::from("Some example."));

        let mut setter = default_setter!();
        setter.attrs = attrs.as_slice();
        setter.deprecation_notes = &deprecated;

        assert_eq!(quote!(#setter), quote!(
            #[some_attr]
            pub fn foo <VALUE: ::std::convert::Into<Foo>>(&mut self, value: VALUE) -> &mut Self {
                #deprecated
                let mut new = self;
                new.foo = ::std::option::Option::Some(value.into());
                new
            }
        ));
    }

    #[test]
    fn setter_disabled() {
        let mut setter = default_setter!();
        setter.enabled = false;

        assert_eq!(quote!(#setter), quote!());
    }
}
