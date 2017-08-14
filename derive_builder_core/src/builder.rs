use quote::{Tokens, ToTokens};
use syn;

use Bindings;
use BuildMethod;
use BuilderField;
use BuilderPattern;
use Setter;
use doc_comment::doc_comment_from;
use DeprecationNotes;

/// Builder, implementing `quote::ToTokens`.
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
/// # use derive_builder_core::{Builder, DeprecationNotes};
/// # fn main() {
/// #    let builder = default_builder!();
/// #
/// #    assert_eq!(quote!(#builder), quote!(
/// #[derive(Default, Clone)]
/// pub struct FooBuilder {
///     foo: u32,
/// }
///
/// #[allow(dead_code)]
/// impl FooBuilder {
///     fn bar () -> {
///         unimplemented!()
///     }
/// }
/// #    ));
/// # }
/// ```
#[derive(Debug)]
pub struct Builder<'a> {
    /// Enables code generation for this builder struct.
    pub enabled: bool,
    /// Name of this builder struct.
    pub ident: &'a syn::Ident,
    /// Pattern of this builder struct.
    pub pattern: BuilderPattern,
    /// Traits to automatically derive on the builder type.
    pub derives: &'a [syn::Ident],
    /// Type parameters and lifetimes attached to this builder's struct
    /// definition.
    pub generics: Option<&'a syn::Generics>,
    /// Visibility of the builder struct, e.g. `syn::Visibility::Public`.
    pub visibility: &'a syn::Visibility,
    /// Fields of the builder struct, e.g. `foo: u32,`
    ///
    /// Expects each entry to be terminated by a comma.
    pub fields: Vec<Tokens>,
    /// Functions of the builder struct, e.g. `fn bar() -> { unimplemented!() }`
    pub functions: Vec<Tokens>,
    /// Doc-comment of the builder struct.
    pub doc_comment: Option<syn::Attribute>,
    /// Emit deprecation notes to the user.
    pub deprecation_notes: DeprecationNotes,
    /// Library bindings to use in emitted builder.
    pub bindings: Bindings,
}

impl<'a> ToTokens for Builder<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if self.enabled {
            trace!("Deriving builder `{}`.", self.ident);
            let builder_vis = self.visibility;
            let builder_ident = self.ident;
            let derives = self.derives;
            let bounded_generics = self.compute_impl_bounds();
            let (impl_generics, _, _) = bounded_generics.split_for_impl();
            let (struct_generics, ty_generics, where_clause) = self.generics
                .map(syn::Generics::split_for_impl)
                .map(|(i, t, w)| (Some(i), Some(t), Some(w)))
                .unwrap_or((None, None, None));
            let builder_fields = &self.fields;
            let functions = &self.functions;
            let builder_doc_comment = &self.doc_comment;
            let deprecation_notes = &self.deprecation_notes.as_item();

            debug!(
                "ty_generics={:?}, where_clause={:?}, struct_generics={:?}",
                ty_generics,
                where_clause,
                struct_generics
            );

            tokens.append(quote!(
                #[derive(Default, Clone #( , #derives)* )]
                #builder_doc_comment
                #builder_vis struct #builder_ident #struct_generics #where_clause {
                    #(#builder_fields)*
                }

                #[allow(dead_code)]
                impl #impl_generics #builder_ident #ty_generics #where_clause {
                    #(#functions)*
                    #deprecation_notes
                }
            ));
        } else {
            trace!("Skipping builder `{}`.", self.ident);
        }
    }
}

impl<'a> Builder<'a> {
    /// Set a doc-comment for this item.
    pub fn doc_comment(&mut self, s: String) -> &mut Self {
        self.doc_comment = Some(doc_comment_from(s));
        self
    }

    /// Add a field to the builder
    pub fn push_field(&mut self, f: BuilderField) -> &mut Self {
        self.fields.push(quote!(#f));
        self
    }

    /// Add a setter function to the builder
    pub fn push_setter_fn(&mut self, f: Setter) -> &mut Self {
        self.functions.push(quote!(#f));
        self
    }

    /// Add final build function to the builder
    pub fn push_build_fn(&mut self, f: BuildMethod) -> &mut Self {
        self.functions.push(quote!(#f));
        self
    }

    /// Add `Clone` trait bound to generic types for non-owned builders.
    /// This enables target types to declare generics without requiring a
    /// `Clone` impl. This is the same as how the built-in derives for
    /// `Clone`, `Default`, `PartialEq`, and other traits work.
    fn compute_impl_bounds(&self) -> syn::Generics {
        if let Some(type_gen) = self.generics {
            let mut generics = type_gen.clone();

            if !self.pattern.requires_clone() || type_gen.ty_params.is_empty() {
                return generics;
            }

            let clone_bound = syn::TyParamBound::Trait(
                syn::PolyTraitRef {
                    trait_ref: syn::parse_path(self.bindings.clone_trait().as_str()).unwrap(),
                    bound_lifetimes: vec![],
                },
                syn::TraitBoundModifier::None,
            );

            for typ in &mut generics.ty_params {
                typ.bounds.push(clone_bound.clone());
            }

            generics
        } else {
            Default::default()
        }
    }
}

/// Helper macro for unit tests. This is _only_ public in order to be accessible
/// from doc-tests too.
#[doc(hidden)]
#[macro_export]
macro_rules! default_builder {
    () => {
        Builder {
            enabled: true,
            ident: &syn::Ident::new("FooBuilder"),
            pattern: Default::default(),
            derives: &vec![],
            generics: None,
            visibility: &syn::Visibility::Public,
            fields: vec![quote!(foo: u32,)],
            functions: vec![quote!(fn bar() -> { unimplemented!() })],
            doc_comment: None,
            deprecation_notes: DeprecationNotes::default(),
            bindings: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn simple() {
        let builder = default_builder!();

        assert_eq!(
            quote!(#builder),
            quote!(
            #[derive(Default, Clone)]
            pub struct FooBuilder {
                foo: u32,
            }

            #[allow(dead_code)]
            impl FooBuilder {
                fn bar () -> {
                    unimplemented!()
                }
            }
        )
        );
    }

    #[test]
    fn generic() {
        let ast = syn::parse_macro_input(stringify!(
            struct Lorem<'a, T: Debug> where T: PartialEq { }
        )).expect("Couldn't parse item");
        let generics = ast.generics;
        let mut builder = default_builder!();
        builder.generics = Some(&generics);

        assert_eq!(
            quote!(#builder),
            quote!(
            #[derive(Default, Clone)]
            pub struct FooBuilder<'a, T: Debug> where T: PartialEq {
                foo: u32,
            }

            #[allow(dead_code)]
            impl<'a, T: Debug + ::std::clone::Clone> FooBuilder<'a, T> where T: PartialEq {
                fn bar() -> {
                    unimplemented!()
                }
            }
        )
        );
    }

    #[test]
    fn generic_reference() {
        let ast = syn::parse_macro_input(stringify!(
            struct Lorem<'a, T: 'a + Default> where T: PartialEq{ }
        )).expect("Couldn't parse item");

        let generics = ast.generics;
        let mut builder = default_builder!();
        builder.generics = Some(&generics);

        assert_eq!(
            quote!(#builder),
            quote!(
            #[derive(Default, Clone)]
            pub struct FooBuilder<'a, T: 'a + Default> where T: PartialEq {
                foo: u32,
            }

            #[allow(dead_code)]
            impl<'a, T: 'a + Default + ::std::clone::Clone> FooBuilder<'a, T> where T: PartialEq {
                fn bar() -> {
                    unimplemented!()
                }
            }
        )
        );
    }

    #[test]
    fn owned_generic() {
        let ast = syn::parse_macro_input(stringify!(
            struct Lorem<'a, T: Debug> where T: PartialEq { }
        )).expect("Couldn't parse item");
        let generics = ast.generics;
        let mut builder = default_builder!();
        builder.generics = Some(&generics);
        builder.pattern = BuilderPattern::Owned;

        assert_eq!(
            quote!(#builder),
            quote!(
            #[derive(Default, Clone)]
            pub struct FooBuilder<'a, T: Debug> where T: PartialEq {
                foo: u32,
            }

            #[allow(dead_code)]
            impl<'a, T: Debug> FooBuilder<'a, T> where T: PartialEq {
                fn bar() -> {
                    unimplemented!()
                }
            }
        )
        );
    }

    #[test]
    fn disabled() {
        let mut builder = default_builder!();
        builder.enabled = false;

        assert_eq!(quote!(#builder), quote!());
    }

    #[test]
    fn add_derives() {
        let derives = vec![syn::Ident::new("Serialize")];
        let mut builder = default_builder!();
        builder.derives = &derives;

        assert_eq!(
            quote!(#builder),
            quote!(
            #[derive(Default, Clone, Serialize)]
            pub struct FooBuilder {
                foo: u32,
            }

            #[allow(dead_code)]
            impl FooBuilder {
                fn bar () -> {
                    unimplemented!()
                }
            }
        )
        );
    }
}
