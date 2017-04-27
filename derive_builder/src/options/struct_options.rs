use syn;
use derive_builder_core::{DeprecationNotes, BuilderPattern, Builder, BuildMethod, Bindings};
use options::DefaultExpression;

/// These struct options define how the builder is generated.
#[derive(Debug, Clone)]
pub struct StructOptions {
    /// Whether or not this struct should implement its own build method.
    pub build_fn_enabled: bool,
    /// The name of the emitted build method.
    pub build_fn_name: syn::Ident,
    /// Name of the builder struct, e.g. `FooBuilder`.
    pub builder_ident: syn::Ident,
    /// Visibility of the builder struct, e.g. `syn::Visibility::Public`.
    pub builder_visibility: syn::Visibility,
    /// The additional traits to derive on the builder.
    pub derives: Vec<syn::Ident>,
    /// How the build method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// Target struct name.
    pub build_target_ident: syn::Ident,
    /// Represents lifetimes and type parameters attached to the declaration of items.
    pub generics: syn::Generics,
    /// Emit deprecation notes to the user,
    /// e.g. if a deprecated attribute was used in `derive_builder`.
    pub deprecation_notes: DeprecationNotes,
    /// Number of fields on the target struct.
    pub struct_size_hint: usize,
    /// Bindings to libstd or libcore.
    pub bindings: Bindings,
    /// Default expression for the whole struct, e.g. `#[builder(default)]` (default to None).
    pub default_expression: Option<DefaultExpression>,
    /// Path to the optional validation function to invoke before the
    /// macro-generated `build` method executes.
    pub validate_fn: Option<syn::Path>,
}

impl StructOptions {
    /// Returns a `Builder` according to the options.
    pub fn as_builder<'a>(&'a self) -> Builder<'a> {
        Builder {
            enabled: true,
            ident: &self.builder_ident,
            pattern: self.builder_pattern,
            derives: &self.derives,
            generics: Some(&self.generics),
            visibility: &self.builder_visibility,
            fields: Vec::with_capacity(self.struct_size_hint),
            functions: Vec::with_capacity(self.struct_size_hint),
            doc_comment: None,
            deprecation_notes: self.deprecation_notes.clone(),
            bindings: self.bindings,
        }
    }
    /// Returns a `BuildMethod` according to the options.
    pub fn as_build_method<'a>(&'a self) -> BuildMethod<'a> {
        let (_impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();
        BuildMethod {
            enabled: self.build_fn_enabled,
            ident: &self.build_fn_name,
            visibility: &self.builder_visibility,
            pattern: self.builder_pattern,
            target_ty: &self.build_target_ident,
            target_ty_generics: Some(ty_generics),
            initializers: Vec::with_capacity(self.struct_size_hint),
            doc_comment: None,
            bindings: self.bindings,
            default_struct: self.default_expression
                .as_ref()
                .map(|x| { x.parse_block(self.bindings.no_std) }),
            validate_fn: self.validate_fn.as_ref(),
        }
    }
}
