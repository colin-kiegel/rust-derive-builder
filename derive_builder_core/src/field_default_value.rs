use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::Type;

use crate::{change_span, DefaultExpression, DEFAULT_FIELD_NAME_PREFIX, DEFAULT_STRUCT_NAME};

/// Calculates the default value or error for fields, implementing `quote::ToTokens
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
/// #    let mut default = default_field_default_value!();
/// #    let default_value = DefaultExpression::explicit::<syn::Expr>(parse_quote!(42));
/// #    default.default_value = Some(&default_value);
/// #    assert_eq!(quote!(#default).to_string(), quote!(
///        let __default_foo: Option<usize> = match self.foo.as_ref() {
///            Some(_) => None,
///            None => Some({ 42 }),
///        };
/// #    ).to_string());
/// # }
/// ```
/// In case there is no default value
/// (`default_value == None && use_default_struct == false`)
/// the `None` case will return an error.
#[derive(Debug, Clone)]
pub struct FieldDefaultValue<'a> {
    /// Path to the root of the derive_builder crate.
    pub crate_root: &'a syn::Path,
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Type of the builder field.
    pub field_type: &'a Type,
    /// Whether the builder implements a setter for this field.
    pub field_enabled: bool,
    /// Whether the builder uses the default value.
    pub enabled: bool,
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
    /// into the return type of its enclosing `build` method. That conversion is guaranteed to
    /// work fr generated error types, but if the caller specified an error type to use instead
    /// they may have forgotten the conversion from `UninitializedFieldError` into their specified
    /// error type.
    pub custom_error_type_span: Option<Span>,
}

impl<'a> ToTokens for FieldDefaultValue<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if !self.enabled {
            return;
        }

        let struct_field = &self.field_ident;
        let builder_field = struct_field;

        let field_type = &self.field_type;

        let default_value = Ident::new(
            &format!("{}{}", DEFAULT_FIELD_NAME_PREFIX, struct_field),
            Span::call_site(),
        );

        if self.field_enabled {
            let default_calculation = self.default_value_calculation();
            tokens.append_all(quote!(
                let #default_value: Option<#field_type> = match self.#builder_field.as_ref() {
                    Some(_) => None,
                    None => #default_calculation,
                };
            ));
        } else {
            let default_calculation = self.default_value_for_disabled();
            tokens.append_all(quote!(
                let #default_value: #field_type = #default_calculation;
            ));
        }
    }
}

impl<'a> FieldDefaultValue<'a> {
    fn default_value_for_disabled(&'a self) -> TokenStream {
        let crate_root = self.crate_root;
        match self.default_value {
            Some(expr) => expr.with_crate_root(crate_root).into_token_stream(),
            None if self.use_default_struct => {
                let struct_ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
                let field_ident = self.field_ident;
                quote!(#struct_ident.#field_ident)
            }
            None => {
                quote!(#crate_root::export::core::default::Default::default())
            }
        }
    }

    fn default_value_calculation(&'a self) -> DefaultValue<'a> {
        match self.default_value {
            Some(expr) => DefaultValue::DefaultTo {
                expr,
                crate_root: self.crate_root,
            },
            None => {
                if self.use_default_struct {
                    DefaultValue::UseDefaultStructField(self.field_ident)
                } else {
                    DefaultValue::ReturnError {
                        crate_root: self.crate_root,
                        field_name: self.field_ident.to_string(),
                        span: self.custom_error_type_span,
                    }
                }
            }
        }
    }
}

enum DefaultValue<'a> {
    /// Inner value must be a valid Rust expression
    DefaultTo {
        expr: &'a DefaultExpression,
        crate_root: &'a syn::Path,
    },
    /// Inner value must be the field identifier
    ///
    /// The default struct must be in scope in the build_method.
    UseDefaultStructField(&'a syn::Ident),
    /// Inner value must be the field name
    ReturnError {
        crate_root: &'a syn::Path,
        field_name: String,
        span: Option<Span>,
    },
}

impl<'a> ToTokens for DefaultValue<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            DefaultValue::DefaultTo { expr, crate_root } => {
                let expr = expr.with_crate_root(crate_root);
                tokens.append_all(quote!(Some(#expr)));
            }
            DefaultValue::UseDefaultStructField(field_ident) => {
                let struct_ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
                tokens.append_all(quote!(
                    Some(#struct_ident.#field_ident)
                ))
            }
            DefaultValue::ReturnError {
                ref field_name,
                ref span,
                crate_root,
            } => {
                let conv_span = span.unwrap_or_else(Span::call_site);
                // If the conversion fails, the compiler error should point to the error declaration
                // rather than the crate root declaration, but the compiler will see the span of #crate_root
                // and produce an undesired behavior (possibly because that's the first span in the bad expression?).
                // Creating a copy with deeply-rewritten spans preserves the desired error behavior.
                let crate_root = change_span(crate_root.into_token_stream(), conv_span);
                let err_conv = quote_spanned!(conv_span => #crate_root::export::core::convert::Into::into(
                    #crate_root::UninitializedFieldError::from(#field_name)
                ));
                tokens.append_all(quote!(
                    return #crate_root::export::core::result::Result::Err(#err_conv)
                ));
            }
        }
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_field_default_value {
    () => {
        FieldDefaultValue {
            // Deliberately don't use the default value here - make sure
            // that all test cases are passing crate_root through properly.
            crate_root: &parse_quote!(::db),
            field_ident: &syn::Ident::new("foo", ::proc_macro2::Span::call_site()),
            field_type: &Type::Verbatim(proc_macro2::TokenStream::from_str("usize").unwrap()),
            field_enabled: true,
            enabled: true,
            default_value: None,
            use_default_struct: false,
            custom_error_type_span: None,
        }
    };
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use super::*;

    use std::str::FromStr;

    #[test]
    fn disabled() {
        let mut default = default_field_default_value!();
        default.enabled = false;

        assert_eq!(quote!(#default).to_string(), quote!().to_string());
    }

    #[test]
    fn disabled_field() {
        let mut default = default_field_default_value!();
        default.field_enabled = false;
        let default_value = DefaultExpression::explicit::<syn::Expr>(parse_quote!(42));
        default.default_value = Some(&default_value);

        assert_eq!(
            quote!(#default).to_string(),
            quote!(
                let __default_foo: usize = { 42 };
            )
            .to_string()
        );
    }

    #[test]
    fn default_value() {
        let mut default = default_field_default_value!();
        let default_value = DefaultExpression::explicit::<syn::Expr>(parse_quote!(42));
        default.default_value = Some(&default_value);

        assert_eq!(
            quote!(#default).to_string(),
            quote!(
                let __default_foo: Option<usize> = match self.foo.as_ref() {
                    Some(_) => None,
                    None => Some({ 42 }),
                };
            )
            .to_string()
        );
    }

    #[test]
    fn default_struct() {
        let mut default = default_field_default_value!();
        default.use_default_struct = true;

        assert_eq!(
            quote!(#default).to_string(),
            quote!(
                let __default_foo: Option<usize> = match self.foo.as_ref() {
                    Some(_) => None,
                    None => Some(__default.foo),
                };
            )
            .to_string()
        );
    }

    #[test]
    fn no_default() {
        let default = default_field_default_value!();

        assert_eq!(
            quote!(#default).to_string(),
            quote!(
                let __default_foo: Option<usize> = match self.foo.as_ref() {
                    Some(_) => None,
                    None => return ::db::export::core::result::Result::Err(
                        ::db::export::core::convert::Into::into(
                            ::db::UninitializedFieldError::from("foo")
                            )
                        ),
                };
            )
            .to_string()
        );
    }
}
