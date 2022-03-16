use std::vec::IntoIter;

use crate::BuildMethod;

use darling::util::{Flag, PathList};
use darling::{self, FromMeta};
use proc_macro2::{Span, TokenStream};
use syn::parse::{ParseStream, Parser};
use syn::Meta;
use syn::{self, spanned::Spanned, Attribute, Generics, Ident, Path, Visibility};

use crate::{
    Builder, BuilderField, BuilderPattern, DefaultExpression, DeprecationNotes, Each, Initializer,
    Setter,
};

/// `derive_builder` uses separate sibling keywords to represent
/// mutually-exclusive visibility states. This trait requires implementers to
/// expose those flags and provides a method to compute any explicit visibility
/// bounds.
trait FlagVisibility {
    fn public(&self) -> &Flag;
    fn private(&self) -> &Flag;

    /// Get the explicitly-expressed visibility preference from the attribute.
    /// This returns `None` if the input didn't include either keyword.
    ///
    /// # Panics
    /// This method panics if the input specifies both `public` and `private`.
    fn as_expressed_vis(&self) -> Option<Visibility> {
        match (self.public().is_some(), self.private().is_some()) {
            (true, true) => panic!("A field cannot be both public and private"),
            (true, false) => Some(syn::parse_quote!(pub)),
            (false, true) => Some(Visibility::Inherited),
            (false, false) => None,
        }
    }
}

/// Options for the `build_fn` property in struct-level builder options.
/// There is no inheritance for these settings from struct-level to field-level,
/// so we don't bother using `Option` for values in this struct.
#[derive(Debug, Clone, FromMeta)]
#[darling(default)]
pub struct BuildFn {
    skip: bool,
    name: Ident,
    validate: Option<Path>,
    public: Flag,
    private: Flag,
    /// The path to an existing error type that the build method should return.
    ///
    /// Setting this will prevent `derive_builder` from generating an error type for the build
    /// method.
    ///
    /// # Type Bounds
    /// This type's bounds depend on other settings of the builder.
    ///
    /// * If uninitialized fields cause `build()` to fail, then this type
    ///   must `impl From<UninitializedFieldError>`. Uninitialized fields do not cause errors
    ///   when default values are provided for every field or at the struct level.
    /// * If `validate` is specified, then this type must provide a conversion from the specified
    ///   function's error type.
    error: Option<Path>,
}

impl Default for BuildFn {
    fn default() -> Self {
        BuildFn {
            skip: false,
            name: Ident::new("build", Span::call_site()),
            validate: None,
            public: Default::default(),
            private: Default::default(),
            error: None,
        }
    }
}

impl FlagVisibility for BuildFn {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

/// Contents of the `field` meta in `builder` attributes.
#[derive(Debug, Clone, Default, FromMeta)]
#[darling(default)]
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
#[darling(default)]
pub struct StructLevelSetter {
    prefix: Option<Ident>,
    into: Option<bool>,
    strip_option: Option<bool>,
    skip: Option<bool>,
}

impl StructLevelSetter {
    /// Check if setters are explicitly enabled or disabled at
    /// the struct level.
    pub fn enabled(&self) -> Option<bool> {
        self.skip.map(|x| !x)
    }
}

/// Create `Each` from an attribute's `Meta`.
///
/// Two formats are supported:
///
/// * `each = "..."`, which provides the name of the `each` setter and otherwise uses default values
/// * `each(name = "...")`, which allows setting additional options on the `each` setter
fn parse_each(meta: &Meta) -> darling::Result<Option<Each>> {
    if let Meta::NameValue(mnv) = meta {
        if let syn::Lit::Str(v) = &mnv.lit {
            v.parse::<Ident>()
                .map(Each::from)
                .map(Some)
                .map_err(|_| darling::Error::unknown_value(&v.value()).with_span(v))
        } else {
            Err(darling::Error::unexpected_lit_type(&mnv.lit))
        }
    } else {
        Each::from_meta(meta).map(Some)
    }
}

/// The `setter` meta item on fields in the input type.
/// Unlike the `setter` meta item at the struct level, this allows specific
/// name overrides.
#[derive(Debug, Clone, Default, FromMeta)]
#[darling(default)]
pub struct FieldLevelSetter {
    prefix: Option<Ident>,
    name: Option<Ident>,
    into: Option<bool>,
    strip_option: Option<bool>,
    skip: Option<bool>,
    custom: Option<bool>,
    #[darling(with = "parse_each")]
    each: Option<Each>,
}

impl FieldLevelSetter {
    /// Get whether the setter should be emitted. The rules are the same as
    /// for `field_enabled`, except we only skip the setter if `setter(custom)` is present.
    pub fn setter_enabled(&self) -> Option<bool> {
        if self.custom.is_some() {
            return self.custom.map(|x| !x);
        }

        self.field_enabled()
    }

    /// Get whether or not this field-level setter indicates a setter and
    /// field should be emitted. The setter shorthand rules are that the
    /// presence of a `setter` with _any_ properties set forces the setter
    /// to be emitted.
    pub fn field_enabled(&self) -> Option<bool> {
        if self.skip.is_some() {
            return self.skip.map(|x| !x);
        }

        if self.prefix.is_some()
            || self.name.is_some()
            || self.into.is_some()
            || self.strip_option.is_some()
            || self.each.is_some()
        {
            return Some(true);
        }

        None
    }
}

/// `derive_builder` allows the calling code to use `setter` as a word to enable
/// setters when they've been disabled at the struct level.
fn field_setter(meta: &Meta) -> darling::Result<FieldLevelSetter> {
    // it doesn't matter what the path is; the fact that this function
    // has been called means that a valueless path is the shorthand case.
    if let Meta::Path(_) = meta {
        Ok(FieldLevelSetter {
            skip: Some(false),
            ..Default::default()
        })
    } else {
        FieldLevelSetter::from_meta(meta)
    }
}

/// Data extracted from the fields of the input struct.
#[derive(Debug, Clone, FromField)]
#[darling(
    attributes(builder),
    forward_attrs(doc, cfg, allow, builder_field_attr, builder_setter_attr),
    and_then = "Self::unnest_attrs"
)]
pub struct Field {
    ident: Option<Ident>,
    /// Raw input attributes, for consumption by Field::unnest_attrs.  Do not use elsewhere.
    attrs: Vec<syn::Attribute>,
    ty: syn::Type,
    /// Field-level override for builder pattern.
    /// Note that setting this may force the builder to derive `Clone`.
    #[darling(default)]
    pattern: Option<BuilderPattern>,
    #[darling(default)]
    public: Flag,
    #[darling(default)]
    private: Flag,
    // See the documentation for `FieldSetterMeta` to understand how `darling`
    // is interpreting this field.
    #[darling(default, with = "field_setter")]
    setter: FieldLevelSetter,
    /// The value for this field if the setter is never invoked.
    ///
    /// A field can get its default one of three ways:
    ///
    /// 1. An explicit `default = "..."` expression
    /// 2. An explicit `default` word, in which case the field type's `Default::default()`
    ///    value is used
    /// 3. Inherited from the field's value in the struct's `default` value.
    ///
    /// This property only captures the first two, the third is computed in `FieldWithDefaults`.
    #[darling(default)]
    default: Option<DefaultExpression>,
    #[darling(default)]
    try_setter: Flag,
    #[darling(default)]
    field: FieldMeta,
    #[darling(skip)]
    field_attrs: Vec<Attribute>,
    #[darling(skip)]
    setter_attrs: Vec<Attribute>,
}

impl Field {
    /// Remove the `builder_field_attr(...)` packaging around an attribute
    fn unnest_attrs(mut self) -> darling::Result<Self> {
        let mut errors = vec![];

        for attr in self.attrs.drain(..) {
            let unnest = {
                if attr.path.is_ident("builder_field_attr") {
                    Some(&mut self.field_attrs)
                } else if attr.path.is_ident("builder_setter_attr") {
                    Some(&mut self.setter_attrs)
                } else {
                    None
                }
            };
            if let Some(unnest) = unnest {
                match unnest_from_one_attribute(attr) {
                    Ok(n) => unnest.push(n),
                    Err(e) => errors.push(e),
                }
            } else {
                self.field_attrs.push(attr.clone());
                self.setter_attrs.push(attr);
            }
        }

        if !errors.is_empty() {
            return Err(darling::Error::multiple(errors));
        }

        Ok(self)
    }
}

fn unnest_from_one_attribute(attr: syn::Attribute) -> darling::Result<Attribute> {
    match &attr.style {
        syn::AttrStyle::Outer => (),
        syn::AttrStyle::Inner(bang) => {
            return Err(darling::Error::unsupported_format(&format!(
                "{} must be an outer attribute",
                attr.path
                    .get_ident()
                    .map(Ident::to_string)
                    .unwrap_or_else(|| "Attribute".to_string())
            ))
            .with_span(bang));
        }
    };

    #[derive(Debug)]
    struct ContainedAttribute(syn::Attribute);
    impl syn::parse::Parse for ContainedAttribute {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Strip parentheses, and save the span of the parenthesis token
            let content;
            let paren_token = parenthesized!(content in input);
            let wrap_span = paren_token.span;

            // Wrap up in #[ ] instead.
            let pound = Token![#](wrap_span); // We can't write a literal # inside quote
            let content: TokenStream = content.parse()?;
            let content = quote_spanned!(wrap_span=> #pound [ #content ]);

            let parser = syn::Attribute::parse_outer;
            let mut attrs = parser.parse2(content)?.into_iter();
            // TryFrom for Array not available in Rust 1.40
            // We think this error can never actually happen, since `#[...]` ought to make just one Attribute
            let attr = match (attrs.next(), attrs.next()) {
                (Some(attr), None) => attr,
                _ => return Err(input.error("expected exactly one attribute")),
            };
            Ok(Self(attr))
        }
    }

    let ContainedAttribute(attr) = syn::parse2(attr.tokens)?;
    Ok(attr)
}

impl FlagVisibility for Field {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

fn default_create_empty() -> Ident {
    Ident::new("create_empty", Span::call_site())
}

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(
    attributes(builder),
    forward_attrs(cfg, allow, builder_struct_attr, builder_impl_attr),
    supports(struct_named),
    and_then = "Self::unnest_attrs"
)]
pub struct Options {
    ident: Ident,

    /// DO NOT USE.
    ///
    /// Initial receiver for forwarded attributes from the struct; these are split
    /// into `Options::struct_attrs` and `Options::impl_attrs` before `FromDeriveInput`
    /// returns.
    attrs: Vec<Attribute>,

    #[darling(skip)]
    struct_attrs: Vec<Attribute>,

    #[darling(skip)]
    impl_attrs: Vec<Attribute>,

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
    derive: PathList,

    #[darling(default)]
    custom_constructor: Flag,

    /// The ident of the inherent method which takes no arguments and returns
    /// an instance of the builder with all fields empty.
    #[darling(default = "default_create_empty")]
    create_empty: Ident,

    /// Setter options applied to all field setters in the struct.
    #[darling(default)]
    setter: StructLevelSetter,

    /// Struct-level value to use in place of any unfilled fields
    #[darling(default)]
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
    try_setter: Flag,

    #[darling(default)]
    field: FieldMeta,

    #[darling(skip, default)]
    deprecation_notes: DeprecationNotes,
}

impl FlagVisibility for Options {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

impl Options {
    /// Remove the `builder_struct_attr(...)` packaging around an attribute
    fn unnest_attrs(mut self) -> darling::Result<Self> {
        let mut errors = vec![];

        for attr in self.attrs.drain(..) {
            let unnest = {
                if attr.path.is_ident("builder_struct_attr") {
                    Some(&mut self.struct_attrs)
                } else if attr.path.is_ident("builder_impl_attr") {
                    Some(&mut self.impl_attrs)
                } else {
                    None
                }
            };
            if let Some(unnest) = unnest {
                match unnest_from_one_attribute(attr) {
                    Ok(n) => unnest.push(n),
                    Err(e) => errors.push(e),
                }
            } else {
                self.struct_attrs.push(attr.clone());
                self.impl_attrs.push(attr);
            }
        }

        if !errors.is_empty() {
            return Err(darling::Error::multiple(errors));
        }

        Ok(self)
    }
}

/// Accessors for parsed properties.
impl Options {
    pub fn builder_ident(&self) -> Ident {
        if let Some(ref custom) = self.name {
            return custom.clone();
        }

        format_ident!("{}Builder", self.ident)
    }

    pub fn builder_error_ident(&self) -> Path {
        if let Some(existing) = self.build_fn.error.as_ref() {
            existing.clone()
        } else if let Some(ref custom) = self.name {
            format_ident!("{}Error", custom).into()
        } else {
            format_ident!("{}BuilderError", self.ident).into()
        }
    }

    /// The visibility of the builder struct.
    /// If a visibility was declared in attributes, that will be used;
    /// otherwise the struct's own visibility will be used.
    pub fn builder_vis(&self) -> Visibility {
        self.as_expressed_vis().unwrap_or_else(|| self.vis.clone())
    }

    /// Get the visibility of the emitted `build` method.
    /// This defaults to the visibility of the parent builder, but can be overridden.
    pub fn build_method_vis(&self) -> Visibility {
        self.build_fn
            .as_expressed_vis()
            .unwrap_or_else(|| self.builder_vis())
    }

    pub fn raw_fields(&self) -> Vec<&Field> {
        self.data
            .as_ref()
            .take_struct()
            .expect("Only structs supported")
            .fields
    }

    /// A builder requires `Clone` to be derived if its build method or any of its setters
    /// use the mutable or immutable pattern.
    pub fn requires_clone(&self) -> bool {
        self.pattern.requires_clone() || self.fields().any(|f| f.pattern().requires_clone())
    }

    /// Get an iterator over the input struct's fields which pulls fallback
    /// values from struct-level settings.
    pub fn fields(&self) -> FieldIter {
        FieldIter(self, self.raw_fields().into_iter())
    }

    pub fn field_count(&self) -> usize {
        self.raw_fields().len()
    }
}

/// Converters to codegen structs
impl Options {
    pub fn as_builder(&self) -> Builder {
        Builder {
            enabled: true,
            ident: self.builder_ident(),
            pattern: self.pattern,
            derives: &self.derive,
            struct_attrs: &self.struct_attrs,
            impl_attrs: &self.impl_attrs,
            impl_default: {
                let custom_constructor: bool = self.custom_constructor.into();
                !custom_constructor
            },
            create_empty: self.create_empty.clone(),
            generics: Some(&self.generics),
            visibility: self.builder_vis(),
            fields: Vec::with_capacity(self.field_count()),
            field_initializers: Vec::with_capacity(self.field_count()),
            functions: Vec::with_capacity(self.field_count()),
            generate_error: self.build_fn.error.is_none(),
            must_derive_clone: self.requires_clone(),
            doc_comment: None,
            deprecation_notes: Default::default(),
            std: {
                let no_std: bool = self.no_std.into();
                !no_std
            },
        }
    }

    pub fn as_build_method(&self) -> BuildMethod {
        let (_, ty_generics, _) = self.generics.split_for_impl();
        BuildMethod {
            enabled: !self.build_fn.skip,
            ident: &self.build_fn.name,
            visibility: self.build_method_vis(),
            pattern: self.pattern,
            target_ty: &self.ident,
            target_ty_generics: Some(ty_generics),
            error_ty: self.builder_error_ident(),
            initializers: Vec::with_capacity(self.field_count()),
            doc_comment: None,
            default_struct: self.default.as_ref(),
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
    pub fn setter_enabled(&self) -> bool {
        self.field
            .setter
            .setter_enabled()
            .or_else(|| self.parent.setter.enabled())
            .unwrap_or(true)
    }

    pub fn field_enabled(&self) -> bool {
        self.field
            .setter
            .field_enabled()
            .or_else(|| self.parent.setter.enabled())
            .unwrap_or(true)
    }

    /// Check if this field should emit a fallible setter.
    /// This depends on the `TryFrom` trait, which hasn't yet stabilized.
    pub fn try_setter(&self) -> bool {
        self.field.try_setter.is_some() || self.parent.try_setter.is_some()
    }

    /// Get the prefix that should be applied to the field name to produce
    /// the setter ident, if any.
    pub fn setter_prefix(&self) -> Option<&Ident> {
        self.field
            .setter
            .prefix
            .as_ref()
            .or(self.parent.setter.prefix.as_ref())
    }

    /// Get the ident of the emitted setter method
    pub fn setter_ident(&self) -> syn::Ident {
        if let Some(ref custom) = self.field.setter.name {
            return custom.clone();
        }

        let ident = &self.field.ident;

        if let Some(ref prefix) = self.setter_prefix() {
            return format_ident!("{}_{}", prefix, ident.as_ref().unwrap());
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

    /// Checks if the emitted setter should strip the wrapper Option over types that impl
    /// `Option<FieldType>`.
    pub fn setter_strip_option(&self) -> bool {
        self.field
            .setter
            .strip_option
            .or(self.parent.setter.strip_option)
            .unwrap_or_default()
    }

    /// Get the visibility of the emitted setter, if there will be one.
    pub fn setter_vis(&self) -> Visibility {
        self.field
            .as_expressed_vis()
            .or_else(|| self.parent.as_expressed_vis())
            .unwrap_or_else(|| syn::parse_quote!(pub))
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

    pub fn pattern(&self) -> BuilderPattern {
        self.field.pattern.unwrap_or(self.parent.pattern)
    }

    pub fn use_parent_default(&self) -> bool {
        self.field.default.is_none() && self.parent.default.is_some()
    }

    pub fn deprecation_notes(&self) -> &DeprecationNotes {
        &self.parent.deprecation_notes
    }
}

/// Converters to codegen structs
impl<'a> FieldWithDefaults<'a> {
    /// Returns a `Setter` according to the options.
    pub fn as_setter(&'a self) -> Setter<'a> {
        Setter {
            setter_enabled: self.setter_enabled(),
            try_setter: self.try_setter(),
            visibility: self.setter_vis(),
            pattern: self.pattern(),
            attrs: &self.field.setter_attrs,
            ident: self.setter_ident(),
            field_ident: self.field_ident(),
            field_type: &self.field.ty,
            generic_into: self.setter_into(),
            strip_option: self.setter_strip_option(),
            deprecation_notes: self.deprecation_notes(),
            each: self.field.setter.each.as_ref(),
        }
    }

    /// Returns an `Initializer` according to the options.
    ///
    /// # Panics
    ///
    /// if `default_expression` can not be parsed as `Block`.
    pub fn as_initializer(&'a self) -> Initializer<'a> {
        Initializer {
            field_enabled: self.field_enabled(),
            field_ident: self.field_ident(),
            builder_pattern: self.pattern(),
            default_value: self.field.default.as_ref(),
            use_default_struct: self.use_parent_default(),
            custom_error_type_span: self
                .parent
                .build_fn
                .error
                .as_ref()
                .map(|err_ty| err_ty.span()),
        }
    }

    pub fn as_builder_field(&'a self) -> BuilderField<'a> {
        BuilderField {
            field_ident: self.field_ident(),
            field_type: &self.field.ty,
            field_enabled: self.field_enabled(),
            field_visibility: self.field_vis(),
            attrs: &self.field.field_attrs,
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
