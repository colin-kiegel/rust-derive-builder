use syn;
use derive_builder_core::{DeprecationNotes, BuilderPattern, Builder, BuildMethod};

/// These struct options define how the builder is generated.
#[derive(Debug, Clone)]
pub struct StructOptions {
    /// Name of the builder struct, e.g. `FooBuilder`.
    pub builder_ident: syn::Ident,
    /// Visibility of the builder struct, e.g. `syn::Visibility::Public`.
    pub builder_visibility: syn::Visibility,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// Target struct name.
    pub build_target_ident: syn::Ident,
    /// Represents lifetimes and type parameters attached to the declaration of items.
    ///
    /// We assume that this is identical for the builder and its build target.
    pub generics: syn::Generics,
    /// Emit deprecation notes to the user,
    /// e.g. if a deprecated attribute was used in `derive_builder`.
    pub deprecation_notes: DeprecationNotes,
    /// Number of fields on the target struct.
    pub struct_len: usize,
}

impl StructOptions {
    /// Returns a `Builder` according to the options.
    pub fn as_builder<'a>(&'a self) -> Builder<'a> {
        Builder {
            enabled: true,
            ident: &self.builder_ident,
            generics: Some(&self.generics),
            visibility: &self.builder_visibility,
            fields: Vec::with_capacity(self.struct_len),
            functions: Vec::with_capacity(self.struct_len),
            doc_comment: None,
            deprecation_notes: DeprecationNotes::default(),
        }
    }
    /// Returns a `BuildMethod` according to the options.
    pub fn as_build_method<'a>(&'a self) -> BuildMethod<'a> {
        let (_impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();
        BuildMethod {
            enabled: true,
            ident: syn::Ident::new("build"),
            visibility: &self.builder_visibility,
            pattern: self.builder_pattern,
            target_ty: &self.build_target_ident,
            target_ty_generics: Some(ty_generics),
            initializers: Vec::with_capacity(self.struct_len),
            doc_comment: None,
        }
    }
}
