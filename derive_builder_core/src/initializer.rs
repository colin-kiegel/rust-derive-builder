use quote::{Tokens, ToTokens};
use syn;
use BuilderPattern;

/// Initializer for the target struct fields, implementing `quote::ToTokens`.
///
/// Lives in the body of `BuildMethod`.
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
/// # use derive_builder_core::{DeprecationNotes, Initializer, BuilderPattern};
/// # fn main() {
/// #    let mut initializer = default_initializer!();
/// #    initializer.default_expr = Some(String::from("lorem ipsum"));
/// #    initializer.builder_pattern = BuilderPattern::Owned;
/// #
/// #    assert_eq!(quote!(#initializer), quote!(
/// foo: match self.foo {
///     Some(value) => value,
///     None => "lorem ipsum",
/// },
/// #    ));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Initializer<'a> {
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Wether the builder implements a setter for this field.
    pub setter_enabled: bool,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// Default value for the target field.
    pub default_expr: Option<String>,
}

impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        trace!("Deriving initializer for `{}`.", self.field_ident);

        let struct_field = &self.field_ident;

        if self.setter_enabled {
            let match_some = self.match_some();
            let match_none = self.match_none();
            let builder_field = &*struct_field;
            tokens.append(quote!(
                #struct_field: match self.#builder_field {
                    #match_some,
                    #match_none,
                },
            ));
        } else {
            tokens.append(quote!(
                #struct_field: ::std::default::Default::default(),
            ));
        }
    }
}

impl<'a> Initializer<'a> {
    /// To be used inside of `#struct_field: match self.#builder_field { ... }`
    fn match_some(&'a self) -> MatchSome {
        match self.builder_pattern {
            BuilderPattern::Owned => MatchSome::Move,
            BuilderPattern::Mutable |
            BuilderPattern::Immutable => MatchSome::Clone,
        }
    }

    /// To be used inside of `#struct_field: match self.#builder_field { ... }`
    fn match_none(&'a self) -> MatchNone<'a> {
        match self.default_expr {
            Some(ref expr) => MatchNone::DefaultTo(expr),
            None => MatchNone::ReturnError(format!("`{}` must be initialized", self.field_ident)),
        }
    }
}

/// To be used inside of `#struct_field: match self.#builder_field { ... }`
enum MatchNone<'a> {
    /// Inner value must be a valid Rust expression
    DefaultTo(&'a str),
    /// Inner value must be the field name
    ReturnError(String),
}

impl<'a> ToTokens for MatchNone<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            MatchNone::DefaultTo(expr) => tokens.append(quote!(
                None => #expr
            )),
            MatchNone::ReturnError(ref err) => tokens.append(quote!(
                None => return ::std::result::Result::Err(::std::string::String::from(#err))
            )),
        }
    }
}

/// To be used inside of `#struct_field: match self.#builder_field { ... }`
enum MatchSome {
    Move,
    Clone,
}

impl<'a> ToTokens for MatchSome {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            MatchSome::Move => tokens.append(quote!(
                Some(value) => value
            )),
            MatchSome::Clone => tokens.append(quote!(
                Some(ref value) => ::std::clone::Clone::clone(value)
            )),
        }
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_initializer {
    () => {
        Initializer {
            field_ident: &syn::Ident::new("foo"),
            setter_enabled: true,
            builder_pattern: BuilderPattern::Mutable,
            default_expr: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn immutable() {
        let mut initializer = default_initializer!();
        initializer.builder_pattern = BuilderPattern::Immutable;

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(ref value) => ::std::clone::Clone::clone(value),
                None => return ::std::result::Result::Err(::std::string::String::from(
                    "`foo` must be initialized"
                )),
            },
        ));
    }

    #[test]
    fn mutable() {
        let mut initializer = default_initializer!();
        initializer.builder_pattern = BuilderPattern::Mutable;

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(ref value) => ::std::clone::Clone::clone(value),
                None => return ::std::result::Result::Err(::std::string::String::from(
                    "`foo` must be initialized"
                )),
            },
        ));
    }

    #[test]
    fn owned() {
        let mut initializer = default_initializer!();
        initializer.builder_pattern = BuilderPattern::Owned;

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(value) => value,
                None => return ::std::result::Result::Err(::std::string::String::from(
                    "`foo` must be initialized"
                )),
            },
        ));
    }

    #[test]
    fn custom_default() {
        let mut initializer = default_initializer!();
        initializer.default_expr = Some(String::from("lorem ipsum"));

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(ref value) => ::std::clone::Clone::clone(value),
                None => "lorem ipsum",
            },
        ));
    }

    #[test]
    fn setter_disabled() {
        let mut initializer = default_initializer!();
        initializer.setter_enabled = false;

        assert_eq!(quote!(#initializer), quote!(
            foo: ::std::default::Default::default(),
        ));
    }
}
