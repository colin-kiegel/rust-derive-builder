use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::Type;

use crate::{
    change_span, BlockContents, DefaultExpression, DEFAULT_FIELD_NAME_PREFIX, DEFAULT_STRUCT_NAME,
};

// TODO: remove no longer needed code from Initializer
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
    /// work for generated error types, but if the caller specified an error type to use instead
    /// they may have forgotten the conversion from `UninitializedFieldError` into their specified
    /// error type.
    pub custom_error_type_span: Option<Span>,
    /// Method to use to to convert the builder's field to the target field
    ///
    /// For sub-builder fields, this will be `build` (or similar)
    pub conversion: FieldConversion<'a>,
}

impl<'a> ToTokens for FieldDefaultValue<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // should be:
        // ```
        // let $prefix$fieldname = match self.$field.as_ref() {
        //      Some(_) => None,
        //      None => Some($default_value)
        //  }
        //  ```

        let struct_field = &self.field_ident;
        let builder_field = struct_field;

        let field_type = &self.field_type;

        let default_value = Ident::new(
            &format!("{}{}", DEFAULT_FIELD_NAME_PREFIX, struct_field),
            Span::call_site(),
        );

        // token stream to generate the calculation of the default value
        let default_calculation = (|| {
            let mut tokens = TokenStream::new();
            if !self.field_enabled {
                // If the field is disabled we just calculate the default here.
                // It is later set in the initializer
                let value = self.disabled_field_value();
                tokens.append_all(quote!(#value));
            } else {
                match &self.conversion {
                    FieldConversion::Block(_) | FieldConversion::Move => {
                        // value is directly accessed, therfor there is no default
                        tokens.append_all(quote!(None))
                    }
                    FieldConversion::OptionOrDefault => {
                        let default = self.default_value();
                        tokens.append_all(quote!(#default));
                    }
                }
            }
            tokens
        })();

        tokens.append_all(quote!(
            let #default_value: Option<#field_type> = match self.#builder_field.as_ref() {
                Some(_) => None,
                None => #default_calculation,
            };
        ));
    }
}

impl<'a> FieldDefaultValue<'a> {
    fn disabled_field_value(&'a self) -> TokenStream {
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
    fn default_value(&'a self) -> DefaultValue<'a> {
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

#[derive(Debug, Clone)]
pub enum FieldConversion<'a> {
    /// Usual conversion: unwrap the Option from the builder, or (hope to) use a default value
    OptionOrDefault,
    /// Custom conversion is a block contents expression
    Block(&'a BlockContents),
    /// Custom conversion is just to move the field from the builder
    Move,
}
