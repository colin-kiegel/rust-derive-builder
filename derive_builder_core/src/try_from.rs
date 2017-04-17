use quote::{Tokens, ToTokens};
use syn;

use {Bindings, BuilderPattern};

/// Generator for `TryFrom<FooBuilder>`.
pub struct TryFromImpl<'a> {
    /// Enables code generation for this TryFrom impl.
    pub enabled: bool,
    /// Name of the builder type.
    pub builder_ty: &'a syn::Ident,
    /// Name of the build fn.
    pub fn_ident: syn::Ident,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub pattern: BuilderPattern,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub target_ty: &'a syn::Ident,
    /// Type parameters and lifetimes attached to this builder struct.
    pub generics: &'a syn::Generics,
    
    /// Bindings to libstd or libcore.
    pub bindings: Bindings,
}

impl<'a> ToTokens for TryFromImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if !self.enabled {
            return;
        }
        
        // For the generic declarations after impl, we need a
        // lifetime so that we can declare TryFrom<&'whatever FooBuilder>.
        // This should NOT appear in the where clause or target type declaration,
        // which is why we have to make the clone.
        let mut i_generics = self.generics.clone();
        i_generics.lifetimes.push(syn::LifetimeDef::new("'try_from"));
        let (impl_generics, _, _) = i_generics.split_for_impl();
        
        let builder_ty = &self.builder_ty;
        let target_ty = &self.target_ty;
        let (owned_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let fn_ident = &self.fn_ident;
        let try_from = self.bindings.try_from_trait();
        let result_ty = self.bindings.result_ty();
        let string_ty = self.bindings.string_ty();
        
        // Emit the conversion which matches the builder pattern. This is required,
        // as the borrow patterns don't work for owned-pattern builders whose fields
        // may not implement `Clone`.
        tokens.append(match self.pattern {
            BuilderPattern::Immutable => quote!(
                impl #impl_generics #try_from<&'try_from #builder_ty #ty_generics> for #target_ty #ty_generics 
                    #where_clause {
                    type Error = #string_ty;
                    
                    fn try_from(v: &#builder_ty #ty_generics) -> #result_ty<Self, Self::Error> {
                        v.#fn_ident()
                    }
                }
            ),
            BuilderPattern::Mutable => quote!(
                impl #impl_generics #try_from<&'try_from mut #builder_ty #ty_generics> for #target_ty #ty_generics 
                    #where_clause {
                    type Error = #string_ty;
                    
                    fn try_from(v: &mut #builder_ty #ty_generics) -> #result_ty<Self, Self::Error> {
                        v.#fn_ident()
                    }
                }
            ),
            BuilderPattern::Owned => quote!(
                impl #owned_generics #try_from<#builder_ty #ty_generics> for #target_ty #ty_generics 
                    #where_clause {
                    type Error = #string_ty;
                    
                    fn try_from(v: #builder_ty #ty_generics) -> #result_ty<Self, Self::Error> {
                        v.#fn_ident()
                    }
                }
            )
        });
    }
}

#[doc(hidden)]
macro_rules! default_try_from {
    () => {
        TryFromImpl {
            enabled: true,
            builder_ty: &syn::Ident::new("FooBuilder"),
            target_ty: &syn::Ident::new("Foo"),
            fn_ident: syn::Ident::new("build"),
            pattern: Default::default(),
            generics: &Default::default(),
            bindings: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn;
    
    use super::TryFromImpl;
    use BuilderPattern;
    
    #[test]
    fn simple() {
        let tf = default_try_from!();
        
        assert_eq!(quote!(#tf), quote!(
            impl<'try_from> ::std::convert::TryFrom<&'try_from mut FooBuilder> for Foo {
                type Error = ::std::string::String;
                fn try_from(v: &mut FooBuilder) -> ::std::result::Result<Self, Self::Error> {
                    v.build()
                }
            }
        ))
    }
    
    #[test]
    fn owned() {
        let mut tf = default_try_from!();
        tf.pattern = BuilderPattern::Owned;
        
        assert_eq!(quote!(#tf), quote!(
            impl ::std::convert::TryFrom<FooBuilder> for Foo {
                type Error = ::std::string::String;
                fn try_from(v: FooBuilder) -> ::std::result::Result<Self, Self::Error> {
                    v.build()
                }
            }
        ))
    }
    
    #[test]
    fn lifetime() {
        let new_generics = {
            let mut generics = syn::Generics::default();
            generics.lifetimes.push(syn::LifetimeDef::new("'a"));
            generics
        };
        
        let mut tf : TryFromImpl = default_try_from!();
        tf.generics = &new_generics;
        
        assert_eq!(quote!(#tf), quote!(
            impl<'a, 'try_from> ::std::convert::TryFrom<&'try_from mut FooBuilder<'a> > for Foo<'a> {
                type Error = ::std::string::String;
                fn try_from(v: &mut FooBuilder<'a>) -> ::std::result::Result<Self, Self::Error> {
                    v.build()
                }
            }
        ));
    }
}