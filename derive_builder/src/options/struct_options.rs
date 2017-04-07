use syn;
use derive_builder_core::{Block, DeprecationNotes, BuilderPattern, Builder, BuildMethod};

use options::field_options::DefaultExpression;

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
    pub struct_size_hint: usize,
    /// Whether the generated code should comply with `#![no_std]`.
    pub no_std: bool,
    /// Whether or not to use the struct's `Default` impl for missing fields.
    pub struct_default: Option<DefaultExpression>,
}

impl StructOptions {
    /// Returns a `Builder` according to the options.
    pub fn as_builder<'a>(&'a self) -> Builder<'a> {
        Builder {
            enabled: true,
            ident: &self.builder_ident,
            generics: Some(&self.generics),
            visibility: &self.builder_visibility,
            fields: Vec::with_capacity(self.struct_size_hint),
            functions: Vec::with_capacity(self.struct_size_hint),
            doc_comment: None,
            deprecation_notes: self.deprecation_notes.clone(),
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
            initializers: Vec::with_capacity(self.struct_size_hint),
            doc_comment: None,
            no_std: self.no_std,
            struct_default: if cfg!(feature = "struct_default") {
                self.struct_default.as_ref().map(struct_default)
            } else {
                None
            },
        }
    }
}

/// Converts a DefaultExpression into the correct code block for the initializer.
fn struct_default(expr: &DefaultExpression) -> Block {
    (match *expr {
        DefaultExpression::Struct 
        | DefaultExpression::Trait => "::std::default::Default::default()".parse(),
        DefaultExpression::Explicit(ref body) => format!("{}()", body).parse()
    }).expect("An explicit struct default must be a valid path to an expression")
}