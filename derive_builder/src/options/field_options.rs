use syn;
use derive_builder_core::{DeprecationNotes, BuilderPattern, Setter, Initializer, BuilderField};

use options::DefaultExpression;

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
    /// e.g. `#[builder(default="42u32")]` (default to None)
    pub default_expression: Option<DefaultExpression>,
    /// The field name, may deviate from `setter_ident`.
    pub field_ident: syn::Ident,
    /// The field type.
    pub field_type: syn::Ty,
    /// Make the setter generic over `Into<_>`.
    pub setter_into: bool,
    /// Emit deprecation notes to the user,
    /// e.g. if a deprecated attribute was used in `derive_builder`.
    pub deprecation_notes: DeprecationNotes,
    pub attrs: Vec<syn::Attribute>
}

impl FieldOptions {
    /// Returns a `Setter` according to the options.
    pub fn as_setter<'a>(&'a self) -> Setter<'a> {
        Setter {
            enabled: self.setter_enabled,
            visibility: &self.setter_visibility,
            pattern: self.builder_pattern,
            attrs: &self.attrs,
            ident: &self.setter_ident,
            field_ident: &self.field_ident,
            field_type: &self.field_type,
            generic_into: self.setter_into,
            deprecation_notes: &self.deprecation_notes,
        }
    }

    /// Returns an `Initializer` according to the options.
    ///
    /// # Panics
    ///
    /// if `default_expression` can not be parsed as `Block`.
    pub fn as_initializer<'a>(&'a self) -> Initializer<'a> {
        Initializer {
            setter_enabled: self.setter_enabled,
            field_ident: &self.field_ident,
            builder_pattern: self.builder_pattern,
            explicit_default: self.default_expression.as_ref().map(|x| {
                match *x {
                    DefaultExpression::Explicit(ref s) => {
                        if s.is_empty() {
                            panic!(r#"Empty default expressions `default=""` are not supported."#);
                        }
                        s.parse()
                    },
                    // Use the struct level default only if the feature is enabled.
                    DefaultExpression::Struct if cfg!(feature = "struct_default") => format!("__default.{}", self.field_ident).parse(),
                    
                    // ... otherwise, fall back to the old style of generating defaults 
                    // based on the field's type.
                    DefaultExpression::Trait 
                    | DefaultExpression::Struct => "::std::default::Default::default()".parse(),
                }.expect(&format!("Couldn't parse default expression `{:?}`", x))
            }),
        }
    }

    /// Returns a `BuilderField` according to the options.
    pub fn as_builder_field<'a>(&'a self) -> BuilderField<'a> {
        BuilderField {
            field_ident: &self.field_ident,
            field_type: &self.field_type,
            setter_enabled: self.setter_enabled,
            setter_visibility: &self.setter_visibility,
            attrs: &self.attrs,
        }
    }
}
