use quote::{Tokens, ToTokens};
use syn;
use BuilderPattern;
use Block;
use Bindings;
use DEFAULT_STRUCT_NAME;

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
/// #    initializer.default_value = Some("42".parse().unwrap());
/// #    initializer.builder_pattern = BuilderPattern::Owned;
/// #
/// #    assert_eq!(quote!(#initializer), quote!(
/// foo: match self.foo {
///     Some(value) => value,
///     None => { 42 },
/// },
/// #    ));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Initializer<'a> {
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Whether the builder implements a setter for this field.
    pub setter_enabled: bool,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// Default value for the target field.
    ///
    /// This takes precedence over a default struct identifier.
    pub default_value: Option<Block>,
    /// Whether the build_method defines a default struct.
    pub use_default_struct: bool,
    /// Bindings to libstd or libcore.
    pub bindings: Bindings,
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
            let default = self.default();
            tokens.append(quote!(
                #struct_field: #default,
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
            BuilderPattern::Immutable => {
                if self.bindings.no_std {
                    MatchSome::CloneNoStd
                } else {
                    MatchSome::Clone
                }
            },
        }
    }

    /// To be used inside of `#struct_field: match self.#builder_field { ... }`
    fn match_none(&'a self) -> MatchNone<'a> {
        match self.default_value {
            Some(ref expr) => MatchNone::DefaultTo(expr),
            None => {
                if self.use_default_struct {
                    MatchNone::UseDefaultStructField(self.field_ident)
                } else if self.bindings.no_std {
                    MatchNone::ReturnErrorNoStd(
                        format!("`{}` must be initialized", self.field_ident),
                    )
                } else {
                    MatchNone::ReturnError(format!("`{}` must be initialized", self.field_ident))
                }
            },
        }
    }

    fn default(&'a self) -> Tokens {
        match self.default_value {
            Some(ref expr) => quote!(#expr),
            None if self.use_default_struct => {
                let struct_ident = syn::Ident::new(DEFAULT_STRUCT_NAME);
                let field_ident = self.field_ident;
                quote!(#struct_ident.#field_ident)
            },
            None => {
                let default = self.bindings.default_trait();
                quote!(#default::default())
            },
        }
    }
}

/// To be used inside of `#struct_field: match self.#builder_field { ... }`
enum MatchNone<'a> {
    /// Inner value must be a valid Rust expression
    DefaultTo(&'a Block),
    /// Inner value must be the field identifier
    ///
    /// The default struct must be in scope in the build_method.
    UseDefaultStructField(&'a syn::Ident),
    /// Inner value must be the field name
    ReturnError(String),
    /// Inner value must be the field name
    ReturnErrorNoStd(String),
}

impl<'a> ToTokens for MatchNone<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            MatchNone::DefaultTo(expr) => {
                tokens.append(quote!(
                None => #expr
            ))
            },
            MatchNone::UseDefaultStructField(field_ident) => {
                let struct_ident = syn::Ident::new(DEFAULT_STRUCT_NAME);
                tokens.append(quote!(
                    None => #struct_ident.#field_ident
                ))
            },
            MatchNone::ReturnError(ref err) => {
                tokens.append(quote!(
                None => return ::std::result::Result::Err(::std::string::String::from(#err))
            ))
            },
            MatchNone::ReturnErrorNoStd(ref err) => {
                tokens.append(quote!(
                None => return ::core::result::Result::Err(
                    ::alloc::string::String::from(#err))
            ))
            },
        }
    }
}

/// To be used inside of `#struct_field: match self.#builder_field { ... }`
enum MatchSome {
    Move,
    Clone,
    CloneNoStd,
}

impl<'a> ToTokens for MatchSome {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            MatchSome::Move => {
                tokens.append(quote!(
                Some(value) => value
            ))
            },
            MatchSome::Clone => {
                tokens.append(quote!(
                Some(ref value) => ::std::clone::Clone::clone(value)
            ))
            },
            MatchSome::CloneNoStd => {
                tokens.append(quote!(
                Some(ref value) => ::core::clone::Clone::clone(value)
            ))
            },
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
            default_value: None,
            use_default_struct: false,
            bindings: Default::default(),
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
    fn default_value() {
        let mut initializer = default_initializer!();
        initializer.default_value = Some("42".parse().unwrap());

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(ref value) => ::std::clone::Clone::clone(value),
                None => { 42 },
            },
        ));
    }

    #[test]
    fn default_struct() {
        let mut initializer = default_initializer!();
        initializer.use_default_struct = true;

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(ref value) => ::std::clone::Clone::clone(value),
                None => __default.foo,
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

    #[test]
    fn no_std() {
        let mut initializer = default_initializer!();
        initializer.bindings.no_std = true;

        assert_eq!(quote!(#initializer), quote!(
            foo: match self.foo {
                Some(ref value) => ::core::clone::Clone::clone(value),
                None => return ::core::result::Result::Err(::alloc::string::String::from(
                    "`foo` must be initialized"
                )),
            },
        ));
    }

    #[test]
    fn no_std_setter_disabled() {
        let mut initializer = default_initializer!();
        initializer.bindings.no_std = true;
        initializer.setter_enabled = false;

        assert_eq!(quote!(#initializer), quote!(
            foo: ::core::default::Default::default(),
        ));
    }
}
