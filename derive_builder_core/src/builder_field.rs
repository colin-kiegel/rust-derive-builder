use quote::{Tokens, ToTokens};
use syn;

/// Field for the builder struct, implementing `quote::ToTokens`.
///
/// # Example
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate syn;
/// # #[macro_use]
/// # extern crate derive_builder_core;
/// # use derive_builder_core::{BuilderField, BuilderPattern};
/// # fn main() {
/// #    let attrs = vec![syn::parse_outer_attr("#[some_attr]").unwrap()];
/// #    let mut field = default_builder_field!();
/// #    field.attrs = attrs.as_slice();
/// #
/// #    assert_eq!(quote!(#field), quote!(
/// #[some_attr] pub foo: ::std::option::Option<String>,
/// #    ));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuilderField<'a> {
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub field_type: &'a syn::Ty,
    /// Wether the builder implements a setter for this field.
    ///
    /// Note: We will fallback to `PhantomData` if the setter is disabled
    ///       to hack around issues with unused generic type parameters - at least for now.
    pub setter_enabled: bool,
    /// Visibility of this builder field, e.g. `syn::Visibility::Public`.
    pub setter_visibility: &'a syn::Visibility,
    /// Attributes which will be attached to this builder field.
    pub attrs: &'a [syn::Attribute],
}

impl<'a> ToTokens for BuilderField<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if self.setter_enabled {
            trace!("Deriving builder field for `{}`.", self.field_ident);
            let vis = self.setter_visibility;
            let ident = self.field_ident;
            let ty = self.field_type;
            let attrs = self.attrs;

            tokens.append(quote!(
                #(#attrs)* #vis #ident: ::std::option::Option<#ty>,
            ));
        } else {
            trace!("Skipping builder field for `{}`, fallback to PhantomData.",
                   self.field_ident);
            let ident = self.field_ident;
            let ty = self.field_type;
            let attrs = self.attrs;

            tokens.append(quote!(
                #(#attrs)* #ident: ::std::marker::PhantomData<#ty>,
            ));
        }
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_builder_field {
    () => {
        BuilderField {
            field_ident: &syn::Ident::new("foo"),
            field_type: &syn::parse_type("String").unwrap(),
            setter_enabled: true,
            setter_visibility: &syn::Visibility::Public,
            attrs: &vec![syn::parse_outer_attr("#[some_attr]").unwrap()]
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn setter_enabled() {
        let field = default_builder_field!();

        assert_eq!(quote!(#field), quote!(
            #[some_attr] pub foo: ::std::option::Option<String>,
        ));
    }

    #[test]
    fn setter_disabled() {
        let mut field = default_builder_field!();
        field.setter_enabled = false;

        assert_eq!(quote!(#field), quote!(
            #[some_attr] foo: ::std::marker::PhantomData<String>,
        ));
    }
}
