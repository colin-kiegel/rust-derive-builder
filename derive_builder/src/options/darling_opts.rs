use derive_builder_core::BuildMethod;
use std::vec::IntoIter;

use darling;
use darling::util::{Flag, IdentList};
use syn::{self, Attribute, Generics, Ident, Path, Visibility};

use derive_builder_core::{Bindings, Builder, BuilderField, BuilderPattern, Initializer, Setter};
use options::DefaultExpression;

trait FlagVisibility {
    fn public(&self) -> &Flag;
    fn private(&self) -> &Flag;

    fn as_expressed_vis(&self) -> Option<Visibility> {
        match (self.public().is_some(), self.private().is_some()) {
            (true, true) => panic!("A field cannot be both public and private"),
            (true, false) => Some(syn::parse_str("pub").unwrap()),
            (false, true) => Some(Visibility::Inherited),
            (false, false) => None,
        }
    }
}

/// Options for the `build_fn` property in struct-level builder options.
#[derive(Debug, Clone, FromMeta)]
pub struct BuildFn {
    skip: bool,
    name: Ident,
    validate: Option<Path>,
}

impl Default for BuildFn {
    fn default() -> Self {
        BuildFn {
            skip: false,
            name: Ident::from("build"),
            validate: None,
        }
    }
}

/// Contents of the `field` meta in `builder` attributes.
#[derive(Debug, Clone, Default, FromMeta)]
pub struct FieldMeta {
    public: Flag,
    private: Flag,
}

impl FlagVisibility for FieldMeta {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

#[derive(Debug, Clone, Default, FromMeta)]
pub struct StructLevelSetter {
    prefix: Option<Ident>,
    into: Option<bool>,
    skip: Option<bool>,
}

#[derive(Debug, Clone, Default, FromMeta)]
pub struct FieldLevelSetter {
    name: Option<Ident>,
    into: Option<bool>,
    skip: Option<bool>,
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(builder), forward_attrs(doc, cfg, allow))]
pub struct Field {
    ident: Option<Ident>,
    attrs: Vec<Attribute>,
    vis: syn::Visibility,
    ty: syn::Type,
    #[darling(default)]
    public: Flag,
    #[darling(default)]
    private: Flag,
    #[darling(default)]
    setter: FieldLevelSetter,
    #[darling(default)]
    default: Option<DefaultExpression>,
    #[darling(default)]
    try_setter: Option<bool>,
    #[darling(default)]
    field: FieldMeta,
}

impl FlagVisibility for Field {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(builder), forward_attrs(doc, cfg, allow))]
pub struct Options {
    ident: Ident,

    attrs: Vec<Attribute>,

    vis: Visibility,

    generics: Generics,

    /// The name of the generated builder. Defaults to `#{ident}Builder`.
    #[darling(default)]
    name: Option<Ident>,

    #[darling(default)]
    pattern: BuilderPattern,

    #[darling(default)]
    build_fn: BuildFn,

    /// Additional traits to derive on the builder.
    #[darling(default)]
    derive: IdentList,

    /// Setter options applied to all field setters in the struct.
    #[darling(default)]
    setter: StructLevelSetter,

    /// Struct-level value to use in place of any unfilled fields
    default: Option<DefaultExpression>,

    #[darling(default)]
    public: Flag,

    #[darling(default)]
    private: Flag,

    /// The parsed body of the derived struct.
    data: darling::ast::Data<darling::util::Ignored, Field>,

    #[darling(default)]
    no_std: Flag,

    /// When present, emit additional fallible setters alongside each regular
    /// setter.
    #[darling(default)]
    try_setter: Option<bool>,

    #[darling(default)]
    field: FieldMeta,
}

impl FlagVisibility for Options {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

/// Accessors for parsed properties.
impl Options {
    pub fn builder_ident(&self) -> Ident {
        if let Some(ref custom) = self.name {
            return custom.clone();
        }

        syn::parse_str(&format!("{}Builder", self.ident))
            .expect("Struct name with Builder suffix should be an ident")
    }

    pub fn builder_vis(&self) -> Visibility {
        self.vis.clone()
    }

    pub fn raw_fields<'a>(&'a self) -> Vec<&'a Field> {
        self.data
            .as_ref()
            .take_struct()
            .expect("Only structs supported")
            .fields
    }

    /// Get an iterator over the input struct's fields which pulls fallback
    /// values from struct-level settings.
    pub fn fields<'a>(&'a self) -> FieldIter<'a> {
        FieldIter(self, self.raw_fields().into_iter())
    }

    pub fn field_count(&self) -> usize {
        self.raw_fields().len()
    }

    fn bindings(&self) -> Bindings {
        Bindings {
            no_std: self.no_std.is_some(),
        }
    }
}

/// Converters to codegen structs
impl Options {
    pub fn as_builder<'a>(&'a self) -> Builder<'a> {
        Builder {
            enabled: true,
            ident: &self.builder_ident(),
            pattern: self.pattern,
            derives: &self.derive,
            generics: Some(&self.generics),
            visibility: &self.builder_vis(),
            fields: Vec::with_capacity(self.field_count()),
            functions: Vec::with_capacity(self.field_count()),
            doc_comment: None,
            deprecation_notes: Default::default(),
            bindings: self.bindings(),
        }
    }

    pub fn as_build_method<'a>(&'a self) -> BuildMethod<'a> {
        let (_, ty_generics, _) = self.generics.split_for_impl();
        BuildMethod {
            enabled: !self.build_fn.skip,
            ident: &self.build_fn.name,
            visibility: &self.builder_vis(),
            pattern: self.pattern,
            target_ty: &self.ident,
            target_ty_generics: Some(ty_generics),
            initializers: Vec::with_capacity(self.field_count()),
            doc_comment: None,
            bindings: self.bindings(),
            default_struct: self.default
                .as_ref()
                .map(|x| x.parse_block(self.no_std.into())),
            validate_fn: self.build_fn.validate.as_ref(),
        }
    }
}

/// Accessor for field data which can pull through options from the parent
/// struct.
pub struct FieldWithDefaults<'a> {
    parent: &'a Options,
    field: &'a Field,
}

/// Accessors for parsed properties, with transparent pull-through from the
/// parent struct's configuration.
impl<'a> FieldWithDefaults<'a> {
    /// Check if this field should emit a setter.
    pub fn enabled(&self) -> bool {
        self.field
            .setter
            .skip
            .or(self.parent.setter.skip)
            .unwrap_or_default()
    }

    pub fn try_setter(&self) -> bool {
        self.field
            .try_setter
            .or(self.parent.try_setter)
            .unwrap_or_default()
    }

    /// Get the ident of the emitted setter method
    pub fn setter_ident(&self) -> syn::Ident {
        if let Some(ref custom) = self.field.setter.name {
            return custom.clone();
        }

        let ident = self.field.ident;

        if let Some(ref prefix) = self.parent.setter.prefix {
            return syn::parse_str(&format!("{}_{}", prefix, ident.as_ref().unwrap())).unwrap();
        }

        ident.clone().unwrap()
    }

    /// Checks if the emitted setter should be generic over types that impl
    /// `Into<FieldType>`.
    pub fn setter_into(&self) -> bool {
        self.field
            .setter
            .into
            .or(self.parent.setter.into)
            .unwrap_or_default()
    }

    /// Get the visibility of the emitted setter, if there will be one.
    pub fn setter_vis(&self) -> Visibility {
        self.field
            .as_expressed_vis()
            .or_else(|| self.parent.as_expressed_vis())
            .unwrap_or(syn::parse_str("pub").unwrap())
    }

    /// Get the ident of the input field. This is also used as the ident of the
    /// emitted field.
    pub fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("Tuple structs are not supported")
    }

    pub fn field_vis(&self) -> Visibility {
        self.field
            .field
            .as_expressed_vis()
            .or_else(|| self.parent.field.as_expressed_vis())
            .unwrap_or(Visibility::Inherited)
    }

    pub fn use_parent_default(&self) -> bool {
        self.field.default.is_none() && self.parent.default.is_some()
    }

    fn bindings(&self) -> Bindings {
        self.parent.bindings()
    }
}

/// Converters to codegen structs
impl<'a> FieldWithDefaults<'a> {
    /// Returns a `Setter` according to the options.
    pub fn as_setter(&'a self) -> Setter<'a> {
        Setter {
            enabled: self.enabled(),
            try_setter: self.try_setter(),
            visibility: &self.setter_vis(),
            pattern: self.parent.pattern,
            attrs: &self.field.attrs,
            ident: &self.setter_ident(),
            field_ident: &self.field_ident(),
            field_type: &self.field.ty,
            generic_into: self.setter_into(),
            deprecation_notes: &Default::default(),
            bindings: self.bindings(),
        }
    }

    /// Returns an `Initializer` according to the options.
    ///
    /// # Panics
    ///
    /// if `default_expression` can not be parsed as `Block`.
    pub fn as_initializer(&'a self) -> Initializer<'a> {
        Initializer {
            setter_enabled: self.enabled(),
            field_ident: self.field_ident(),
            builder_pattern: self.parent.pattern,
            default_value: self.field
                .default
                .as_ref()
                .map(|x| x.parse_block(self.parent.no_std.into())),
            use_default_struct: self.use_parent_default(),
            bindings: self.bindings(),
        }
    }

    pub fn as_builder_field(&'a self) -> BuilderField<'a> {
        BuilderField {
            field_ident: self.field_ident(),
            field_type: &self.field.ty,
            setter_enabled: self.enabled(),
            field_visibility: &self.field_vis(),
            attrs: &self.field.attrs,
            bindings: self.bindings(),
        }
    }
}

pub struct FieldIter<'a>(&'a Options, IntoIter<&'a Field>);

impl<'a> Iterator for FieldIter<'a> {
    type Item = FieldWithDefaults<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|field| FieldWithDefaults {
            parent: self.0,
            field,
        })
    }
}
