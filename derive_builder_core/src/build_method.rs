use quote::{Tokens, ToTokens};
use syn;
use Block;
use BuilderPattern;
use Bindings;
use Initializer;
use doc_comment::doc_comment_from;
use DEFAULT_STRUCT_NAME;

/// Initializer for the struct fields in the build method, implementing
/// `quote::ToTokens`.
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate syn;
/// # #[macro_use(default_build_method)]
/// # extern crate derive_builder_core;
/// # use derive_builder_core::{BuildMethod, BuilderPattern};
/// # fn main() {
/// #    let build_method = default_build_method!();
/// #
/// #    assert_eq!(quote!(#build_method), quote!(
/// pub fn build(&self) -> ::std::result::Result<Foo, ::std::string::String> {
///     Ok(Foo {
///         foo: self.foo,
///     })
/// }
/// #    ));
/// # }
/// ```
#[derive(Debug)]
pub struct BuildMethod<'a> {
    /// Enables code generation for this build method.
    pub enabled: bool,
    /// Name of this build fn.
    pub ident: &'a syn::Ident,
    /// Visibility of the build method, e.g. `syn::Visibility::Public`.
    pub visibility: &'a syn::Visibility,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub pattern: BuilderPattern,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub target_ty: &'a syn::Ident,
    /// Type parameters and lifetimes attached to this builder struct.
    pub target_ty_generics: Option<syn::TyGenerics<'a>>,
    /// Field initializers for the target type.
    pub initializers: Vec<Tokens>,
    /// Doc-comment of the builder struct.
    pub doc_comment: Option<syn::Attribute>,
    /// Bindings to libstd or libcore.
    pub bindings: Bindings,
    /// Default value for the whole struct.
    ///
    /// This will be in scope for all initializers as `__default`.
    pub default_struct: Option<Block>,
    /// Validation function with signature `&FooBuilder -> Result<(), String>`
    /// to call before the macro-provided struct buildout.
    pub validate_fn: Option<&'a syn::Path>,
}

impl<'a> ToTokens for BuildMethod<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ident = &self.ident;
        let vis = self.visibility;
        let target_ty = &self.target_ty;
        let target_ty_generics = &self.target_ty_generics;
        let initializers = &self.initializers;
        let self_param = match self.pattern {
            BuilderPattern::Owned => quote!(self),
            BuilderPattern::Mutable |
            BuilderPattern::Immutable => quote!(&self),
        };
        let doc_comment = &self.doc_comment;
        let default_struct = self.default_struct
            .as_ref()
            .map(|default_expr| {
                     let ident = syn::Ident::new(DEFAULT_STRUCT_NAME);
                     quote!(let #ident: #target_ty = #default_expr;)
                 });
        let validate_fn = self.validate_fn
            .as_ref()
            .map(|vfn| quote!(#vfn(&self)?;));
        let result = self.bindings.result_ty();
        let string = self.bindings.string_ty();

        if self.enabled {
            trace!("Deriving build method `{}`.", self.ident.as_ref());
            tokens.append(quote!(
                #doc_comment
                #vis fn #ident(#self_param)
                    -> #result<#target_ty #target_ty_generics, #string>
                {
                    #validate_fn
                    #default_struct
                    Ok(#target_ty {
                        #(#initializers)*
                    })
                }
            ))
        } else {
            trace!("Skipping build method.");
        }
    }
}

impl<'a> BuildMethod<'a> {
    /// Set a doc-comment for this item.
    pub fn doc_comment(&mut self, s: String) -> &mut Self {
        self.doc_comment = Some(doc_comment_from(s));
        self
    }

    /// Populate the `BuildMethod` with appropriate initializers of the
    /// underlying struct.
    ///
    /// For each struct field this must be called with the appropriate
    /// initializer.
    pub fn push_initializer(&mut self, init: Initializer) -> &mut Self {
        self.initializers.push(quote!(#init));
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
            ident: &syn::Ident::new("build"),
            visibility: &syn::Visibility::Public,
            pattern: BuilderPattern::Mutable,
            target_ty: &syn::Ident::new("Foo"),
            target_ty_generics: None,
            initializers: vec![quote!(foo: self.foo,)],
            doc_comment: None,
            bindings: Default::default(),
            default_struct: None,
            validate_fn: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn std() {
        let build_method = default_build_method!();

        assert_eq!(quote!(#build_method), quote!(
            pub fn build(&self) -> ::std::result::Result<Foo, ::std::string::String> {
                Ok(Foo {
                    foo: self.foo,
                })
            }
        ));
    }

    #[test]
    fn no_std() {
        let mut build_method = default_build_method!();
        build_method.bindings.no_std = true;

        assert_eq!(quote!(#build_method), quote!(
            pub fn build(&self) -> ::core::result::Result<Foo, ::collections::string::String> {
                Ok(Foo {
                    foo: self.foo,
                })
            }
        ));
    }

    #[test]
    fn default_struct() {
        let mut build_method = default_build_method!();
        build_method.default_struct = Some("Default::default()".parse().unwrap());

        assert_eq!(quote!(#build_method), quote!(
            pub fn build(&self) -> ::std::result::Result<Foo, ::std::string::String> {
                let __default: Foo = {Default::default()};
                Ok(Foo {
                    foo: self.foo,
                })
            }
        ));
    }

    #[test]
    fn skip() {
        let mut build_method = default_build_method!();
        build_method.enabled = false;
        build_method.enabled = false;

        assert_eq!(quote!(#build_method), quote!());
    }

    #[test]
    fn rename() {
        let ident = syn::Ident::new("finish");
        let mut build_method: BuildMethod = default_build_method!();
        build_method.ident = &ident;

        assert_eq!(quote!(#build_method), quote!(
            pub fn finish(&self) -> ::std::result::Result<Foo, ::std::string::String> {
                Ok(Foo {
                    foo: self.foo,
                })
            }
        ))
    }

    #[test]
    fn validation() {
        let validate_path = syn::parse_path("IpsumBuilder::validate")
            .expect("Statically-entered path should be valid");

        let mut build_method: BuildMethod = default_build_method!();
        build_method.validate_fn = Some(&validate_path);

        assert_eq!(quote!(#build_method), quote!(
            pub fn build(&self) -> ::std::result::Result<Foo, ::std::string::String> {
                IpsumBuilder::validate(&self)?;

                Ok(Foo {
                    foo: self.foo,
                })
            }
        ));
    }
}
