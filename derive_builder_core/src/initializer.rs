use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};

use crate::{BuilderPattern, DefaultExpression, FieldConversion, DEFAULT_FIELD_NAME_PREFIX};

/// Initializer for the target struct fields, implementing `quote::ToTokens`.
///
/// Lives in the body of `BuildMethod`.
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
/// # #[macro_use]
/// # extern crate derive_builder_core;
/// # use derive_builder_core::{DeprecationNotes, Initializer, BuilderPattern};
/// # fn main() {
/// #    let mut initializer = default_initializer!();
/// #    initializer.default_value = Some("42".parse().unwrap());
/// #    initializer.builder_pattern = BuilderPattern::Owned;
/// #
/// #    assert_eq!(quote!(#initializer).to_string(), quote!(
/// foo: match self.foo {
///     Some(value) => value,
///     None => { 42 },
/// },
/// #    ).to_string());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Initializer<'a> {
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Whether the builder implements a setter for this field.
    pub field_enabled: bool,

    // TODO delete fields below here
    //
    /// Path to the root of the derive_builder crate.
    pub crate_root: &'a syn::Path,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// Default value for the target field.
    ///
    /// This takes precedence over a default struct identifier.
    pub default_value: Option<&'a DefaultExpression>,
    /// Whether the build_method defines a default struct.
    pub use_default_struct: bool,
    /// Span where the macro was told to use a preexisting error type, instead of creating one,
    /// to represent failures of the `build` method.
    ///
    /// An initializer can force early-return if a field has no set value and no default is
    /// defined. In these cases, it will convert from `derive_builder::UninitializedFieldError`
    /// into thereturn type of its enclosing `build` method. That conversion is guaranteed to
    /// work for generated error types, but if the caller specified an error type to use instead
    /// they may have forgotten the conversion from `UninitializedFieldError` into their specified
    /// error type.
    pub custom_error_type_span: Option<Span>,
    /// Method to use to to convert the builder's field to the target field
    ///
    /// For sub-builder fields, this will be `build` (or similar)
    pub conversion: FieldConversion<'a>,
}

impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_field = &self.field_ident;
        let builder_field = struct_field;

        let default_value = Ident::new(
            &format!("{}{}", DEFAULT_FIELD_NAME_PREFIX, struct_field),
            Span::call_site(),
        );

        // This structure prevents accidental failure to add the trailing `,` due to incautious `return`
        let append_rhs = |tokens: &mut TokenStream| {
            if !self.field_enabled {
                tokens.append_all(quote!(#default_value));
            } else {
                tokens.append_all(quote!( self.#builder_field.or(#default_value).unwrap()));
            }
        };

        tokens.append_all(quote!(#struct_field:));
        append_rhs(tokens);
        tokens.append_all(quote!(,));
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_initializer {
    () => {
        Initializer {
            // Deliberately don't use the default value here - make sure
            // that all test cases are passing crate_root through properly.
            crate_root: &parse_quote!(::db),
            field_ident: &syn::Ident::new("foo", ::proc_macro2::Span::call_site()),
            field_enabled: true,
            builder_pattern: BuilderPattern::Mutable,
            default_value: None,
            use_default_struct: false,
            conversion: FieldConversion::OptionOrDefault,
            custom_error_type_span: None,
        }
    };
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn immutable() {
        let mut initializer = default_initializer!();
        initializer.builder_pattern = BuilderPattern::Immutable;

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: match self.foo {
                    Some(ref value) => ::db::export::core::clone::Clone::clone(value),
                    None => return ::db::export::core::result::Result::Err(::db::export::core::convert::Into::into(
                        ::db::UninitializedFieldError::from("foo")
                    )),
                },
            )
            .to_string()
        );
    }

    #[test]
    fn mutable() {
        let mut initializer = default_initializer!();
        initializer.builder_pattern = BuilderPattern::Mutable;

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: match self.foo {
                    Some(ref value) => ::db::export::core::clone::Clone::clone(value),
                    None => return ::db::export::core::result::Result::Err(::db::export::core::convert::Into::into(
                        ::db::UninitializedFieldError::from("foo")
                    )),
                },
            )
            .to_string()
        );
    }

    #[test]
    fn owned() {
        let mut initializer = default_initializer!();
        initializer.builder_pattern = BuilderPattern::Owned;

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: match self.foo {
                    Some(value) => value,
                    None => return ::db::export::core::result::Result::Err(::db::export::core::convert::Into::into(
                        ::db::UninitializedFieldError::from("foo")
                    )),
                },
            )
            .to_string()
        );
    }

    #[test]
    fn default_value() {
        let mut initializer = default_initializer!();
        let default_value = DefaultExpression::explicit::<syn::Expr>(parse_quote!(42));
        initializer.default_value = Some(&default_value);

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: match self.foo {
                    Some(ref value) => ::db::export::core::clone::Clone::clone(value),
                    None => { 42 },
                },
            )
            .to_string()
        );
    }

    #[test]
    fn default_struct() {
        let mut initializer = default_initializer!();
        initializer.use_default_struct = true;

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: match self.foo {
                    Some(ref value) => ::db::export::core::clone::Clone::clone(value),
                    None => __default.foo,
                },
            )
            .to_string()
        );
    }

    #[test]
    fn setter_disabled() {
        let mut initializer = default_initializer!();
        initializer.field_enabled = false;

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(foo: ::db::export::core::default::Default::default(),).to_string()
        );
    }

    #[test]
    fn no_std() {
        let initializer = default_initializer!();

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: match self.foo {
                    Some(ref value) => ::db::export::core::clone::Clone::clone(value),
                    None => return ::db::export::core::result::Result::Err(::db::export::core::convert::Into::into(
                        ::db::UninitializedFieldError::from("foo")
                    )),
                },
            )
            .to_string()
        );
    }
}
