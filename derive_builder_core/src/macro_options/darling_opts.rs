use std::{borrow::Cow, slice};

use crate::macro_options::{parse_optional_bool, set, Diagnostic};
use crate::BuildMethod;

use proc_macro2::Span;
use syn::{
    meta::ParseNestedMeta, spanned::Spanned, token, Attribute, Data, Generics, Ident, LitBool,
    LitStr, Meta, Path, Visibility,
};

use crate::{
    BlockContents, Builder, BuilderField, BuilderFieldType, BuilderPattern, DefaultExpression,
    DeprecationNotes, Each, FieldConversion, Initializer, Setter,
};

/// `derive_builder` uses separate sibling keywords to represent
/// mutually-exclusive visibility states.
#[derive(Debug)]
enum VisibilityAttr {
    /// `public`
    Public,
    /// `private`
    Private,
    /// `vis = "pub(crate)"`
    Explicit(Visibility),
    None,
}

impl Default for VisibilityAttr {
    fn default() -> Self {
        Self::None
    }
}

impl VisibilityAttr {
    fn parse_nested_meta(
        &mut self,
        meta: &ParseNestedMeta,
        diag: &mut Diagnostic,
    ) -> syn::Result<bool> {
        if meta.path.is_ident("public") {
            self.report_conflict(meta, diag);
            *self = Self::Public;
            Ok(true)
        } else if meta.path.is_ident("private") {
            self.report_conflict(meta, diag);
            *self = Self::Private;
            Ok(true)
        } else if meta.path.is_ident("vis") {
            let vis: Visibility = meta.value()?.parse::<LitStr>()?.parse()?;
            self.report_conflict(meta, diag);
            *self = Self::Explicit(vis);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn report_conflict(&self, meta: &ParseNestedMeta, diag: &mut Diagnostic) {
        match self {
            Self::Public => {
                let msg = "this visibility conflicts with a `public` specified earlier";
                diag.push(meta.error(msg));
            }
            Self::Private => {
                let msg = "this visibility conflicts with a `private` specified earlier";
                diag.push(meta.error(msg));
            }
            Self::Explicit(_) => {
                let msg = r#"this visibility conflicts with a `vis = "..."` specified earlier"#;
                diag.push(meta.error(msg));
            }
            Self::None => {}
        }
    }

    /// Get the explicitly-expressed visibility preference from the attribute.
    /// This returns `None` if the input didn't include either keyword.
    fn as_expressed_vis(&self) -> Option<Cow<Visibility>> {
        match self {
            Self::Public => Some(Cow::Owned(parse_quote!(pub))),
            Self::Private => Some(Cow::Owned(Visibility::Inherited)),
            Self::Explicit(vis) => Some(Cow::Borrowed(vis)),
            Self::None => None,
        }
    }
}

#[derive(Debug)]
struct BuildFnErrorGenerated {
    /// Indicates whether or not the generated error should have
    /// a validation variant that takes a `String` as its contents.
    validation_error: bool,
}

#[derive(Debug)]
enum BuildFnError {
    Existing(Path),
    Generated(BuildFnErrorGenerated),
}

impl BuildFnError {
    fn parse_nested_meta(meta: &ParseNestedMeta, diag: &mut Diagnostic) -> syn::Result<Self> {
        let lookahead = meta.input.lookahead1();
        if lookahead.peek(Token![=]) {
            let path: Path = meta.value()?.parse::<LitStr>()?.parse()?;
            return Ok(Self::Existing(path));
        } else if !lookahead.peek(token::Paren) {
            return Err(lookahead.error());
        }

        let mut validation_error = None;

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("validation_error") {
                let lit: LitBool = meta.value()?.parse()?;
                set(&meta, &mut validation_error, lit.value, diag);
            } else {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(Self::Generated(BuildFnErrorGenerated {
            validation_error: validation_error.ok_or_else(|| {
                syn::Error::new_spanned(&meta.path, "missing attribute `validation_error`")
            })?,
        }))
    }

    fn as_existing(&self) -> Option<&Path> {
        match self {
            BuildFnError::Existing(p) => Some(p),
            BuildFnError::Generated(_) => None,
        }
    }

    fn as_generated(&self) -> Option<&BuildFnErrorGenerated> {
        match self {
            BuildFnError::Generated(e) => Some(e),
            BuildFnError::Existing(_) => None,
        }
    }
}

/// Options for the `build_fn` property in struct-level builder options.
/// There is no inheritance for these settings from struct-level to field-level,
/// so we don't bother using `Option` for values in this struct.
#[derive(Debug)]
pub struct BuildFn {
    skip: bool,
    name: Ident,
    validate: Option<Path>,
    vis: VisibilityAttr,
    /// Either the path to an existing error type that the build method should return or a meta
    /// list of options to modify the generated error.
    ///
    /// Setting this to a path will prevent `derive_builder` from generating an error type for the
    /// build method.
    ///
    /// This options supports to formats: path `error = "path::to::Error"` and meta list
    /// `error(<options>)`. Supported mata list options are the following:
    ///
    /// * `validation_error = bool` - Whether to generate `ValidationError(String)` as a variant
    ///   of the build error type. Setting this to `false` will prevent `derive_builder` from
    ///   using the `validate` function but this also means it does not generate any usage of the
    ///  `alloc` crate (useful when disabling the `alloc` feature in `no_std`).
    ///
    /// # Type Bounds for Custom Error
    /// This type's bounds depend on other settings of the builder.
    ///
    /// * If uninitialized fields cause `build()` to fail, then this type
    ///   must `impl From<UninitializedFieldError>`. Uninitialized fields do not cause errors
    ///   when default values are provided for every field or at the struct level.
    /// * If `validate` is specified, then this type must provide a conversion from the specified
    ///   function's error type.
    error: Option<BuildFnError>,
}

impl BuildFn {
    fn parse_nested_meta(meta: &ParseNestedMeta, diag: &mut Diagnostic) -> syn::Result<Self> {
        let mut skip = None;
        let mut name = None;
        let mut validate = None;
        let mut vis = VisibilityAttr::None;
        let mut build_fn_error = None;

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut skip, value, diag);
            } else if meta.path.is_ident("name") {
                let value: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut name, value, diag);
            } else if meta.path.is_ident("validate") {
                let value: Path = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut validate, value, diag);
                Self::check_validation(&meta, &validate, &build_fn_error, diag);
            } else if meta.path.is_ident("error") {
                let value = BuildFnError::parse_nested_meta(&meta, diag)?;
                set(&meta, &mut build_fn_error, value, diag);
                Self::check_validation(&meta, &validate, &build_fn_error, diag);
            } else if !vis.parse_nested_meta(&meta, diag)? {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(BuildFn {
            skip: skip.unwrap_or(false),
            name: name.unwrap_or_else(|| Ident::new("build", Span::call_site())),
            validate,
            vis,
            error: build_fn_error,
        })
    }

    fn check_validation(
        meta: &ParseNestedMeta,
        validate: &Option<Path>,
        build_fn_error: &Option<BuildFnError>,
        diag: &mut Diagnostic,
    ) {
        if validate.is_some() {
            if let Some(BuildFnError::Generated(e)) = build_fn_error {
                if !e.validation_error {
                    diag.push(meta.error(
                        "`error(validation_error = false)` and `validate` cannot be used together",
                    ));
                }
            }
        }
    }
}

impl Default for BuildFn {
    fn default() -> Self {
        BuildFn {
            skip: false,
            name: Ident::new("build", Span::call_site()),
            validate: None,
            vis: VisibilityAttr::None,
            error: None,
        }
    }
}

/// Contents of the `field` meta in `builder` attributes at the struct level.
#[derive(Debug, Default)]
pub struct StructLevelFieldMeta {
    vis: VisibilityAttr,
}

impl StructLevelFieldMeta {
    fn parse_nested_meta(meta: &ParseNestedMeta, diag: &mut Diagnostic) -> syn::Result<Self> {
        let mut vis = VisibilityAttr::None;

        meta.parse_nested_meta(|meta| {
            if !vis.parse_nested_meta(&meta, diag)? {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(StructLevelFieldMeta { vis })
    }
}

/// Contents of the `field` meta in `builder` attributes at the field level.
//
// This is a superset of the attributes permitted in `field` at the struct level.
// Perhaps the data structures can be refactored to share common parts.
#[derive(Debug, Default)]
pub struct FieldLevelFieldMeta {
    vis: VisibilityAttr,
    /// Custom builder field type
    builder_type: Option<syn::Type>,
    /// Custom builder field method, for making target struct field value
    build: Option<BlockContents>,
}

impl FieldLevelFieldMeta {
    fn parse_nested_meta(meta: &ParseNestedMeta, diag: &mut Diagnostic) -> syn::Result<Self> {
        let mut vis = VisibilityAttr::None;
        let mut builder_type = None;
        let mut build = None;

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("ty") || meta.path.is_ident("type") {
                let value: syn::Type = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut builder_type, value, diag);
            } else if meta.path.is_ident("build") {
                let value = BlockContents::parse_nested_meta(&meta)?;
                set(&meta, &mut build, value, diag);
            } else if !vis.parse_nested_meta(&meta, diag)? {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(FieldLevelFieldMeta {
            vis,
            builder_type,
            build,
        })
    }
}

#[derive(Debug, Default)]
pub struct StructLevelSetter {
    prefix: Option<Ident>,
    into: Option<bool>,
    strip_option: Option<bool>,
    skip: Option<bool>,
}

impl StructLevelSetter {
    fn parse_nested_meta(meta: &ParseNestedMeta, diag: &mut Diagnostic) -> syn::Result<Self> {
        let mut prefix = None;
        let mut into = None;
        let mut strip_option = None;
        let mut skip = None;

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                let value = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut prefix, value, diag);
            } else if meta.path.is_ident("into") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut into, value, diag);
            } else if meta.path.is_ident("strip_option") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut strip_option, value, diag);
            } else if meta.path.is_ident("skip") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut skip, value, diag);
            } else {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(StructLevelSetter {
            prefix,
            into,
            strip_option,
            skip,
        })
    }

    /// Check if setters are explicitly enabled or disabled at
    /// the struct level.
    pub fn enabled(&self) -> Option<bool> {
        self.skip.map(|x| !x)
    }
}

/// The `setter` meta item on fields in the input type.
/// Unlike the `setter` meta item at the struct level, this allows specific
/// name overrides.
#[derive(Debug, Default)]
pub struct FieldLevelSetter {
    prefix: Option<Ident>,
    name: Option<Ident>,
    into: Option<bool>,
    strip_option: Option<bool>,
    skip: Option<bool>,
    custom: Option<bool>,
    each: Option<Each>,
}

impl FieldLevelSetter {
    fn parse_nested_meta(meta: &ParseNestedMeta, diag: &mut Diagnostic) -> syn::Result<Self> {
        if !meta.input.peek(token::Paren) {
            // `setter` as a word is equivalent to `setter(skip = false)`. This
            // is useful for re-enabling setter for one field when they've been
            // disabled at the struct level.
            return Ok(FieldLevelSetter {
                skip: Some(false),
                ..Default::default()
            });
        }

        let mut prefix = None;
        let mut name = None;
        let mut into = None;
        let mut strip_option = None;
        let mut skip = None;
        let mut custom = None;
        let mut each = None;

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                let value: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut prefix, value, diag);
            } else if meta.path.is_ident("name") {
                let value: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut name, value, diag);
            } else if meta.path.is_ident("into") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut into, value, diag);
            } else if meta.path.is_ident("strip_option") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut strip_option, value, diag);
            } else if meta.path.is_ident("skip") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut skip, value, diag);
            } else if meta.path.is_ident("custom") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut custom, value, diag);
            } else if meta.path.is_ident("each") {
                let value = Each::parse_nested_meta(&meta, diag)?;
                set(&meta, &mut each, value, diag);
            } else {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(FieldLevelSetter {
            prefix,
            name,
            into,
            strip_option,
            skip,
            custom,
            each,
        })
    }

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

/// Data extracted from the fields of the input struct.
#[derive(Debug)]
pub struct Field {
    ident: Option<Ident>,
    ty: syn::Type,
    /// Field-level override for builder pattern.
    /// Note that setting this may force the builder to derive `Clone`.
    pattern: Option<BuilderPattern>,
    /// Declared visibility for the field in the builder, e.g. `#[builder(vis = "...")]`.
    vis: VisibilityAttr,
    /// `derive_builder` allows the calling code to use `setter` as a word to enable
    /// setters when they've been disabled at the struct level.
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
    default: Option<DefaultExpression>,
    try_setter: bool,
    field: FieldLevelFieldMeta,
    field_attrs: Vec<Attribute>,
    setter_attrs: Vec<Attribute>,
}

impl Field {
    fn from_field(ast: &syn::Field, diag: &mut Diagnostic) -> syn::Result<Self> {
        let mut pattern = None;
        let mut vis = VisibilityAttr::None;
        let mut setter = None;
        let mut default = None;
        let mut try_setter = None;
        let mut field = None;
        let mut field_attrs = Vec::new();
        let mut setter_attrs = Vec::new();

        for attr in &ast.attrs {
            if attr.path().is_ident("builder") {
                if let Err(err) = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("pattern") {
                        let value = BuilderPattern::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut pattern, value, diag);
                    } else if meta.path.is_ident("setter") {
                        let value = FieldLevelSetter::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut setter, value, diag)
                    } else if meta.path.is_ident("default") {
                        let value = DefaultExpression::parse_nested_meta(&meta)?;
                        set(&meta, &mut default, value, diag);
                        Self::check_field_vs_default(&meta, &field, &default, diag);
                    } else if meta.path.is_ident("try_setter") {
                        set(&meta, &mut try_setter, true, diag);
                    } else if meta.path.is_ident("field") {
                        let value = FieldLevelFieldMeta::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut field, value, diag);
                        Self::check_field_vs_default(&meta, &field, &default, diag);
                    } else if !vis.parse_nested_meta(&meta, diag)? {
                        return Err(meta.error("unrecognized derive_builder attribute"));
                    }
                    Ok(())
                }) {
                    diag.push(err);
                }
            } else if attr.path().is_ident("doc")
                || attr.path().is_ident("cfg")
                || attr.path().is_ident("allow")
            {
                field_attrs.push(attr.clone());
                setter_attrs.push(attr.clone());
            } else if attr.path().is_ident("builder_field_attr") {
                unnest_attr(attr, &mut field_attrs, diag);
            } else if attr.path().is_ident("builder_setter_attr") {
                unnest_attr(attr, &mut setter_attrs, diag);
            }
        }

        Ok(Field {
            ident: ast.ident.clone(),
            ty: ast.ty.clone(),
            pattern,
            vis,
            setter: setter.unwrap_or_default(),
            default,
            try_setter: try_setter.unwrap_or(false),
            field: field.unwrap_or_default(),
            field_attrs,
            setter_attrs,
        })
    }

    /// Check that we don't have a custom field type or builder *and* a default value.
    fn check_field_vs_default(
        meta: &ParseNestedMeta,
        field: &Option<FieldLevelFieldMeta>,
        default: &Option<DefaultExpression>,
        diag: &mut Diagnostic,
    ) {
        // `default` can be preempted by properties in `field`. Silently ignoring a
        // `default` could cause the direct user of `derive_builder` to see unexpected
        // behavior from the builder, so instead we require that the deriving struct
        // not pass any ignored instructions.
        if let (Some(field), Some(_default)) = (field, default) {
            // `field.build` is stronger than `default`, as it contains both instructions on how to
            // deal with a missing value and conversions to do on the value during target type
            // construction.
            if field.build.is_some() {
                diag.push(meta.error(
                    r#"#[builder(default)] and #[builder(field(build="..."))] cannot be used together"#,
                ));
            }

            // `field.ty` being set means `default` will not be used, since we don't know how
            // to check a custom field type for the absence of a value and therefore we'll never
            // know that we should use the `default` value.
            if field.builder_type.is_some() {
                diag.push(meta.error(
                    r#"#[builder(default)] and #[builder(field(ty="..."))] cannot be used together"#,
                ));
            }
        }
    }
}

/// Convert an attribute like `#[builder_struct_attr(doc(hidden))]` into `#[doc(hidden)]`.
fn unnest_attr(attr: &Attribute, out: &mut Vec<Attribute>, diag: &mut Diagnostic) {
    match attr.parse_args() {
        Ok(meta) => out.push(Attribute {
            pound_token: attr.pound_token,
            style: attr.style,
            bracket_token: attr.bracket_token,
            meta,
        }),
        Err(err) => diag.push(err),
    }
}

#[derive(Debug)]
pub struct Options {
    ident: Ident,

    struct_attrs: Vec<Attribute>,

    impl_attrs: Vec<Attribute>,

    /// The visibility of the deriving struct.
    ///
    /// Do not confuse this with `builder_vis` which is the visibility received by `#[builder(vis = "...")]`,
    struct_vis: Visibility,

    generics: Generics,

    /// The name of the generated builder. Defaults to `#{ident}Builder`.
    name: Option<Ident>,

    /// The path to the root of the derive_builder crate used in generated
    /// code.
    crate_root: Path,

    pattern: BuilderPattern,

    build_fn: BuildFn,

    /// Additional traits to derive on the builder.
    derive: Vec<syn::Path>,

    custom_constructor: bool,

    /// The ident of the inherent method which takes no arguments and returns
    /// an instance of the builder with all fields empty.
    create_empty: Ident,

    /// Setter options applied to all field setters in the struct.
    setter: StructLevelSetter,

    /// Struct-level value to use in place of any unfilled fields
    default: Option<DefaultExpression>,

    /// Desired visibility of the builder struct.
    ///
    /// Do not confuse this with `struct_vis`, which is the visibility of the deriving struct.
    builder_vis: VisibilityAttr,

    /// The parsed body of the derived struct.
    data: Vec<Field>,

    no_std: bool,

    /// When present, emit additional fallible setters alongside each regular
    /// setter.
    try_setter: bool,

    field: StructLevelFieldMeta,

    deprecation_notes: DeprecationNotes,
}

impl Options {
    pub(crate) fn from_derive_input(ast: &syn::DeriveInput) -> syn::Result<Self> {
        let diag = &mut Diagnostic::new();

        let mut struct_attrs = Vec::new();
        let mut impl_attrs = Vec::new();
        let mut name = None;
        let mut crate_root = None;
        let mut pattern = None;
        let mut build_fn = None;
        let mut derive = None;
        let mut custom_constructor = None;
        let mut create_empty = None;
        let mut setter = None;
        let mut default = None;
        let mut builder_vis = VisibilityAttr::None;
        let mut data = Vec::new();
        let mut no_std = None;
        let mut try_setter = None;
        let mut field = None;

        for attr in &ast.attrs {
            if attr.path().is_ident("builder") {
                if let Meta::Path(_) = attr.meta {
                    continue; // Ignore empty #[builder], which would otherwise be an error.
                }
                if let Err(err) = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let value: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
                        set(&meta, &mut name, value, diag);
                    } else if meta.path.is_ident("crate") {
                        let value: Path = meta.value()?.parse::<LitStr>()?.parse()?;
                        set(&meta, &mut crate_root, value, diag);
                    } else if meta.path.is_ident("pattern") {
                        let value = BuilderPattern::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut pattern, value, diag);
                    } else if meta.path.is_ident("build_fn") {
                        let value = BuildFn::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut build_fn, value, diag);
                    } else if meta.path.is_ident("derive") {
                        let mut value = Vec::new();
                        meta.parse_nested_meta(|meta| {
                            value.push(meta.path);
                            Ok(())
                        })?;
                        set(&meta, &mut derive, value, diag);
                    } else if meta.path.is_ident("custom_constructor") {
                        set(&meta, &mut custom_constructor, true, diag);
                    } else if meta.path.is_ident("create_empty") {
                        let value: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
                        set(&meta, &mut create_empty, value, diag);
                    } else if meta.path.is_ident("setter") {
                        let value = StructLevelSetter::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut setter, value, diag);
                    } else if meta.path.is_ident("default") {
                        let value = DefaultExpression::parse_nested_meta(&meta)?;
                        set(&meta, &mut default, value, diag);
                    } else if meta.path.is_ident("no_std") {
                        set(&meta, &mut no_std, true, diag);
                    } else if meta.path.is_ident("try_setter") {
                        set(&meta, &mut try_setter, true, diag);
                    } else if meta.path.is_ident("field") {
                        let value = StructLevelFieldMeta::parse_nested_meta(&meta, diag)?;
                        set(&meta, &mut field, value, diag);
                    } else if !builder_vis.parse_nested_meta(&meta, diag)? {
                        return Err(meta.error("unrecognized derive_builder attribute"));
                    }
                    Ok(())
                }) {
                    diag.push(err);
                }
            } else if attr.path().is_ident("cfg") || attr.path().is_ident("allow") {
                struct_attrs.push(attr.clone());
                impl_attrs.push(attr.clone());
            } else if attr.path().is_ident("builder_struct_attr") {
                unnest_attr(attr, &mut struct_attrs, diag);
            } else if attr.path().is_ident("builder_impl_attr") {
                unnest_attr(attr, &mut impl_attrs, diag);
            }
        }

        if let Data::Struct(ast) = &ast.data {
            for field in &ast.fields {
                let field = Field::from_field(field, diag)?;
                data.push(field);
            }
        } else {
            let msg = "this derive macro requires a struct with named fields";
            diag.push(syn::Error::new(Span::call_site(), msg));
        }

        if let Some(error) = diag.take() {
            return Err(error);
        }

        Ok(Options {
            ident: ast.ident.clone(),
            struct_attrs,
            impl_attrs,
            struct_vis: ast.vis.clone(),
            generics: ast.generics.clone(),
            name,
            crate_root: crate_root.unwrap_or_else(|| parse_quote!(::derive_builder)),
            pattern: pattern.unwrap_or_default(),
            build_fn: build_fn.unwrap_or_default(),
            derive: derive.unwrap_or_default(),
            custom_constructor: custom_constructor.unwrap_or(false),
            create_empty: create_empty.unwrap_or_else(|| parse_quote!(create_empty)),
            setter: setter.unwrap_or_default(),
            default,
            builder_vis,
            data,
            no_std: no_std.unwrap_or(false),
            try_setter: try_setter.unwrap_or(false),
            field: field.unwrap_or_default(),
            deprecation_notes: DeprecationNotes::default(),
        })
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
        if let Some(BuildFnError::Existing(existing)) = self.build_fn.error.as_ref() {
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
    pub fn builder_vis(&self) -> Cow<Visibility> {
        self.builder_vis
            .as_expressed_vis()
            .unwrap_or(Cow::Borrowed(&self.struct_vis))
    }

    /// Get the visibility of the emitted `build` method.
    /// This defaults to the visibility of the parent builder, but can be overridden.
    pub fn build_method_vis(&self) -> Cow<Visibility> {
        self.build_fn
            .vis
            .as_expressed_vis()
            .unwrap_or_else(|| self.builder_vis())
    }

    /// A builder requires `Clone` to be derived if its build method or any of its setters
    /// use the mutable or immutable pattern.
    pub fn requires_clone(&self) -> bool {
        self.pattern.requires_clone() || self.fields().any(|f| f.pattern().requires_clone())
    }

    /// Get an iterator over the input struct's fields which pulls fallback
    /// values from struct-level settings.
    pub fn fields(&self) -> FieldIter {
        FieldIter(self, self.data.iter())
    }

    pub fn field_count(&self) -> usize {
        self.data.len()
    }
}

/// Converters to codegen structs
impl Options {
    pub fn as_builder(&self) -> Builder {
        Builder {
            crate_root: &self.crate_root,
            enabled: true,
            ident: self.builder_ident(),
            pattern: self.pattern,
            derives: &self.derive,
            struct_attrs: &self.struct_attrs,
            impl_attrs: &self.impl_attrs,
            impl_default: !self.custom_constructor,
            create_empty: self.create_empty.clone(),
            generics: Some(&self.generics),
            visibility: self.builder_vis(),
            fields: Vec::with_capacity(self.field_count()),
            field_initializers: Vec::with_capacity(self.field_count()),
            functions: Vec::with_capacity(self.field_count()),
            generate_error: self
                .build_fn
                .error
                .as_ref()
                .and_then(BuildFnError::as_existing)
                .is_none(),
            generate_validation_error: self
                .build_fn
                .error
                .as_ref()
                .and_then(BuildFnError::as_generated)
                .map(|e| e.validation_error)
                .unwrap_or(true),
            no_alloc: cfg!(not(any(feature = "alloc", feature = "lib_has_std"))),
            must_derive_clone: self.requires_clone(),
            doc_comment: None,
            deprecation_notes: Default::default(),
            std: !self.no_std,
        }
    }

    pub fn as_build_method(&self) -> BuildMethod {
        let (_, ty_generics, _) = self.generics.split_for_impl();
        BuildMethod {
            crate_root: &self.crate_root,
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
        self.field.try_setter || self.parent.try_setter
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
    pub fn setter_vis(&self) -> Cow<Visibility> {
        self.field
            .vis
            .as_expressed_vis()
            .or_else(|| self.parent.builder_vis.as_expressed_vis())
            .unwrap_or_else(|| Cow::Owned(syn::parse_quote!(pub)))
    }

    /// Get the ident of the input field. This is also used as the ident of the
    /// emitted field.
    pub fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("Tuple structs are not supported")
    }

    pub fn field_vis(&self) -> Cow<Visibility> {
        self.field
            .field
            .vis
            .as_expressed_vis()
            .or_else(
                // Disabled fields become a PhantomData in the builder.  We make that field
                // non-public, even if the rest of the builder is public, since this field is just
                // there to make sure the struct's generics are properly handled.
                || {
                    if self.field_enabled() {
                        None
                    } else {
                        Some(Cow::Owned(Visibility::Inherited))
                    }
                },
            )
            .or_else(|| self.parent.field.vis.as_expressed_vis())
            .unwrap_or(Cow::Owned(Visibility::Inherited))
    }

    pub fn field_type(&'a self) -> BuilderFieldType<'a> {
        if !self.field_enabled() {
            BuilderFieldType::Phantom(&self.field.ty)
        } else if let Some(custom_ty) = self.field.field.builder_type.as_ref() {
            BuilderFieldType::Precise(custom_ty)
        } else {
            BuilderFieldType::Optional(&self.field.ty)
        }
    }

    pub fn conversion(&'a self) -> FieldConversion<'a> {
        match (&self.field.field.builder_type, &self.field.field.build) {
            (_, Some(block)) => FieldConversion::Block(block),
            (Some(_), None) => FieldConversion::Move,
            (None, None) => FieldConversion::OptionOrDefault,
        }
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
            crate_root: &self.parent.crate_root,
            setter_enabled: self.setter_enabled(),
            try_setter: self.try_setter(),
            visibility: self.setter_vis(),
            pattern: self.pattern(),
            attrs: &self.field.setter_attrs,
            ident: self.setter_ident(),
            field_ident: self.field_ident(),
            field_type: self.field_type(),
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
            crate_root: &self.parent.crate_root,
            field_enabled: self.field_enabled(),
            field_ident: self.field_ident(),
            builder_pattern: self.pattern(),
            default_value: self.field.default.as_ref(),
            use_default_struct: self.use_parent_default(),
            conversion: self.conversion(),
            custom_error_type_span: self.parent.build_fn.error.as_ref().and_then(|err_ty| {
                match err_ty {
                    BuildFnError::Existing(p) => Some(p.span()),
                    _ => None,
                }
            }),
        }
    }

    pub fn as_builder_field(&'a self) -> BuilderField<'a> {
        BuilderField {
            crate_root: &self.parent.crate_root,
            field_ident: self.field_ident(),
            field_type: self.field_type(),
            field_visibility: self.field_vis(),
            attrs: &self.field.field_attrs,
        }
    }
}

pub struct FieldIter<'a>(&'a Options, slice::Iter<'a, Field>);

impl<'a> Iterator for FieldIter<'a> {
    type Item = FieldWithDefaults<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|field| FieldWithDefaults {
            parent: self.0,
            field,
        })
    }
}
