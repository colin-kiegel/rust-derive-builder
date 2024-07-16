use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};

use crate::{BlockContents, BuilderPattern, DEFAULT_FIELD_NAME_PREFIX};

/// Initializer for the target struct fields, implementing `quote::ToTokens`.
///
/// Lives in the body of `BuildMethod`.
///
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
/// #    initializer.builder_pattern = BuilderPattern::Owned;
/// #
/// #    assert_eq!(quote!(#initializer).to_string(), quote!(
///        foo: self.foo.or(__default_foo).unwrap(),
/// #    ).to_string());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Initializer<'a> {
    /// Path to the root of the derive_builder crate.
    pub crate_root: &'a syn::Path,
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Whether the builder implements a setter for this field.
    pub field_enabled: bool,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// Method to use to to convert the builder's field to the target field
    ///
    /// For sub-builder fields, this will be `build` (or similar)
    /// If the `conversion` is `FieldConversion::OptionOrDefault` this will
    /// use the default value calculated in `FieldDefaultValue`. Otherwise
    /// the default value is calculated based on `default_value` and
    /// `use_default_struct`.
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
                tokens.append(default_value);
            } else {
                match &self.conversion {
                    FieldConversion::Move => tokens.append_all(quote!(self.#builder_field)),
                    FieldConversion::OptionOrDefault => {
                        let moved_or_cloned =
                            self.move_or_clone_option(quote!(self.#builder_field));
                        tokens.append_all(quote!( #moved_or_cloned.or(#default_value).unwrap()))
                    }
                    FieldConversion::Block(content) => content.to_tokens(tokens),
                }
            }
        };

        tokens.append_all(quote!(#struct_field:));
        append_rhs(tokens);
        tokens.append_all(quote!(,));
    }
}

impl<'a> Initializer<'a> {
    fn move_or_clone_option(&'a self, value_in_option: TokenStream) -> TokenStream {
        let crate_root = self.crate_root;

        match self.builder_pattern {
            BuilderPattern::Owned => value_in_option,
            BuilderPattern::Mutable | BuilderPattern::Immutable => {
                quote!(
                    #value_in_option.as_ref()
                        .map(|value| #crate_root::export::core::clone::Clone::clone(value))
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldConversion<'a> {
    /// Usual conversion: unwrap the Option from the builder, or (hope to) use a default value
    OptionOrDefault,
    /// Custom conversion is a block contents expression
    Block(&'a BlockContents),
    /// Custom conversion is just to move the field from the builder
    Move,
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
            conversion: FieldConversion::OptionOrDefault,
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
               foo: self.foo.as_ref()
                   .map(|value| ::db::export::core::clone::Clone::clone(value))
                   .or(__default_foo).unwrap(),
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
                foo: self.foo.as_ref()
                    .map(|value| ::db::export::core::clone::Clone::clone(value))
                    .or(__default_foo).unwrap(),
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
                foo: self.foo.or(__default_foo).unwrap(),
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
            quote!(
                foo: __default_foo,
            )
            .to_string()
        );
    }

    #[test]
    fn no_std() {
        let initializer = default_initializer!();

        assert_eq!(
            quote!(#initializer).to_string(),
            quote!(
                foo: self.foo.as_ref()
                    .map(|value| ::db::export::core::clone::Clone::clone(value))
                    .or(__default_foo).unwrap(),
            )
            .to_string()
        );
    }
}
