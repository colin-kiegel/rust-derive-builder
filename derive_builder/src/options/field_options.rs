use syn;
use derive_builder_core::{DeprecationNotes, BuilderPattern, Setter, Initializer, BuilderField,
                          Block, Bindings};
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
    /// Visibility of the field, e.g. `syn::Visibility::Public`.
    pub field_visibility: syn::Visibility,
    /// Default expression for the field, e.g. `#[builder(default="42u32")]` (default to None).
    pub default_expression: Option<DefaultExpression>,
    /// Whether the build_method defines a default struct.
    pub use_default_struct: bool,
    /// The field name, may deviate from `setter_ident`.
    pub field_ident: syn::Ident,
    /// The field type.
    pub field_type: syn::Ty,
    /// Make the setter generic over `Into<_>`.
    pub setter_into: bool,
    /// Emit deprecation notes to the user,
    /// e.g. if a deprecated attribute was used in `derive_builder`.
    pub deprecation_notes: DeprecationNotes,
    /// Setter attributes, e.g. `#[allow(non_snake_case)]`.
    pub attrs: Vec<syn::Attribute>,
    /// Bindings to libstd or libcore.
    pub bindings: Bindings,
    /// Enables code generation for the TryInto setter.
    pub try_setter: bool,
}

impl DefaultExpression {
    pub fn parse_block(&self, no_std: bool) -> Block {
        let expr = match *self {
            DefaultExpression::Explicit(ref s) => {
                if s.is_empty() {
                    panic!(r#"Empty default expressions `default=""` are not supported."#);
                }
                s
            },
            DefaultExpression::Trait => if no_std {
                "::core::default::Default::default()"
            } else {
                "::std::default::Default::default()"
            },
        };

        expr.parse().expect(&format!("Couldn't parse default expression `{:?}`", self))
    }
}

impl FieldOptions {
    /// Returns a `Setter` according to the options.
    pub fn as_setter<'a>(&'a self) -> Setter<'a> {
        Setter {
            enabled: self.setter_enabled,
            try_setter: self.try_setter,
            visibility: &self.setter_visibility,
            pattern: self.builder_pattern,
            attrs: &self.attrs,
            ident: &self.setter_ident,
            field_ident: &self.field_ident,
            field_type: &self.field_type,
            generic_into: self.setter_into,
            deprecation_notes: &self.deprecation_notes,
            bindings: self.bindings,
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
            default_value: self.default_expression
                .as_ref()
                .map(|x| { x.parse_block(self.bindings.no_std) }),
            use_default_struct: self.use_default_struct,
            bindings: self.bindings,
        }
    }

    /// Returns a `BuilderField` according to the options.
    pub fn as_builder_field<'a>(&'a self) -> BuilderField<'a> {
        BuilderField {
            field_ident: &self.field_ident,
            field_type: &self.field_type,
            setter_enabled: self.setter_enabled,
            field_visibility: &self.field_visibility,
            attrs: &self.attrs,
            bindings: self.bindings,
        }
    }
}
