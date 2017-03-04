use syn;
use derive_builder_core::{DeprecationNotes, BuilderPattern, Setter, Initializer, BuilderField};

/// These field options define how the builder interacts with the field.
#[derive(Debug, Clone)]
pub struct FieldOptions {
    /// Enables code generation for this setter.
    pub setter_enabled: bool,
    /// How the setter method takes and returns `self` (e.g. mutably).
    pub builder_pattern: BuilderPattern,
    /// The setter name.
    pub setter_ident: syn::Ident,
    /// Visibility of the setter, e.g. `syn::Visibility::Public`.
    pub setter_visibility: syn::Visibility,
    /// The field name, may deviate from `setter_ident`.
    pub field_ident: syn::Ident,
    /// The field type.
    pub field_type: syn::Ty,
    /// Emit deprecation notes to the user,
    /// e.g. if a deprecated attribute was used in `derive_builder`.
    pub deprecation_notes: DeprecationNotes,
    pub attrs: Vec<syn::Attribute>
}

impl FieldOptions {
    pub fn to_setter<'a>(&'a self) -> Setter<'a> {
        Setter {
            enabled: self.setter_enabled,
            visibility: &self.setter_visibility,
            pattern: self.builder_pattern,
            attrs: &self.attrs,
            ident: &self.setter_ident,
            field_ident: &self.field_ident,
            field_type: &self.field_type,
            deprecation_notes: &self.deprecation_notes,
        }
    }

    pub fn to_initializer<'a>(&'a self) -> Initializer<'a> {
        Initializer {
            setter_enabled: self.setter_enabled,
            field_ident: &self.field_ident,
            builder_pattern: self.builder_pattern,
            default_expr: None,
        }
    }

    pub fn to_builder_field<'a>(&'a self) -> BuilderField<'a> {
        BuilderField {
            field_ident: &self.field_ident,
            field_type: &self.field_type,
            setter_enabled: self.setter_enabled,
            setter_visibility: &self.setter_visibility,
            attrs: &self.attrs,
        }
    }
}
