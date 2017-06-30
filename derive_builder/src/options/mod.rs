//! Types and functions for parsing attribute options.
//!
//! Attribute parsing occurs in multiple stages:
//!
//! 1. Builder options on the struct are parsed into `OptionsBuilder<StructMode>`.
//! 1. The `OptionsBuilder<StructMode>` instance is converted into a starting point for the
//!    per-field options (`OptionsBuilder<FieldMode>`) and the finished struct-level config,
//!    called `StructOptions`.
//! 1. Each struct field is parsed, with discovered attributes overriding or augmenting the
//!    options specified at the struct level. This creates one `OptionsBuilder<FieldMode>` per
//!    struct field on the input/target type. Once complete, these get converted into
//!    `FieldOptions` instances.

use syn;
use derive_builder_core::BuilderPattern;

#[macro_use]
mod macros;
mod field_mode;
mod field_options;
mod struct_mode;
mod struct_options;

pub use self::field_mode::FieldMode;
pub use self::field_options::FieldOptions;
pub use self::struct_mode::StructMode;
pub use self::struct_options::StructOptions;

/// A `DefaultExpression` can be either explicit or refer to the canonical trait.
#[derive(Debug, Clone)]
pub enum DefaultExpression {
    Explicit(String),
    Trait,
}

/// Get the tuple of `StructOptions` and field defaults (`OptionsBuilder<FieldMode>`) from the AST.
pub fn struct_options_from(ast: &syn::MacroInput) -> (StructOptions, OptionsBuilder<FieldMode>) {
    OptionsBuilder::<StructMode>::parse(ast).into()
}

///  Get the `FieldOptions` for a field with respect to some custom default values.
pub fn field_options_from(f: syn::Field, defaults: &OptionsBuilder<FieldMode>) -> FieldOptions {
    OptionsBuilder::<FieldMode>::parse(f).with_defaults(defaults).into()
}

/// Build `StructOptions` and `FieldOptions`.
///
/// The difference between `StructOptions` and `FieldOptions` is expressed via a different `Mode`.
#[derive(Debug, Clone)]
pub struct OptionsBuilder<Mode> {
    builder_pattern: Option<BuilderPattern>,
    setter_enabled: Option<bool>,
    setter_prefix: Option<String>,
    /// Takes precedence over `setter_prefix`
    setter_name: Option<String>,
    setter_vis: Option<syn::Visibility>,
    field_vis: Option<syn::Visibility>,
    default_expression: Option<DefaultExpression>,
    setter_into: Option<bool>,
    try_setter: Option<bool>,
    no_std: Option<bool>,
    mode: Mode,
}

/// Certain attributes need to be handled differently for `StructOptions` and `FieldOptions`.
pub trait OptionsBuilderMode: ::std::fmt::Debug {
    fn parse_builder_name(&mut self, lit: &syn::Lit);
    fn parse_derive(&mut self, nested: &[syn::NestedMetaItem]);
    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self;
    /// Provide a diagnostic _where_-clause for panics.
    fn where_diagnostics(&self) -> String;
    fn struct_mode(&self) -> bool;

    fn parse_build_fn_options(&mut self, nested: &[syn::NestedMetaItem]);
}

impl<Mode> From<Mode> for OptionsBuilder<Mode> {
    fn from(mode: Mode) -> OptionsBuilder<Mode> {
        OptionsBuilder {
            builder_pattern: None,
            setter_enabled: None,
            setter_prefix: None,
            setter_name: None,
            setter_vis: None,
            try_setter: None,
            field_vis: None,
            default_expression: None,
            setter_into: None,
            no_std: None,
            mode: mode,
        }
    }
}

impl<Mode> OptionsBuilder<Mode>
    where Mode: OptionsBuilderMode
{
    impl_setter!{
        ident: setter_enabled,
        desc: "setter activation",
        map: |x: bool| { x },
    }

    impl_setter!{
        ident: builder_pattern,
        desc: "builder pattern",
        map: |x: BuilderPattern| { x },
    }

    impl_setter!{
        ident: field_public for field_vis,
        desc: "field visibility",
        map: |x: bool| { if x { syn::Visibility::Public } else { syn::Visibility::Inherited } },
    }

    impl_setter!{
        ident: setter_public for setter_vis,
        desc: "setter visibility",
        map: |x: bool| { if x { syn::Visibility::Public } else { syn::Visibility::Inherited } },
    }

    impl_setter!{
        ident: setter_into,
        desc: "setter type conversion",
        map: |x: bool| { x },
    }

    impl_setter!{
        ident: try_setter,
        desc: "try_setter activation",
        map: |x: bool| { x },
    }

    impl_setter!{
        ident: default_expression,
        desc: "default expression",
        map: |x: DefaultExpression| { x },
    }

    impl_setter!{
        ident: no_std,
        desc: "no_std support",
        map: |x: bool| { x },
    }

    impl_setter!{
        ident: setter_prefix,
        desc: "setter prefix",
        map: |x: String| { x },
    }

    impl_setter!{
        ident: setter_name,
        desc: "setter name",
        map: |x: String| { x },
    }

    pub fn parse_attributes<'a, T>(&mut self, attributes: T) -> &mut Self
        where T: IntoIterator<Item = &'a syn::Attribute>
    {
        trace!("Parsing attributes.");
        for attr in attributes {
            self.parse_attribute(attr);
        }

        self
    }

    fn parse_attribute(&mut self, attr: &syn::Attribute) {
        const BUILDER_ATTRIBUTE_IDENT: &'static str = "builder";

        if attr.value.name() != BUILDER_ATTRIBUTE_IDENT {
            trace!("Ignoring attribute `{}`.", attr.value.name());
            return
        }

        if attr.style != syn::AttrStyle::Outer || attr.is_sugared_doc {
            debug!("Ignoring attribute `{:?}` (outer or sugared doc).", attr);
            return
        }

        match attr.value {
            // i.e. `#[builder(...)]`
            syn::MetaItem::List(ref _ident, ref nested_attrs) => {
                self.parse_builder_options(nested_attrs);
                return
            },
            syn::MetaItem::Word(_) |
            syn::MetaItem::NameValue(_, _) => {
                error!("Expected MetaItem::List, found `{:?}`", attr.value);
                panic!("Could not parse builder options {}.", self.where_diagnostics());
            }
        }
    }

    fn parse_builder_options(&mut self, nested: &[syn::NestedMetaItem]) {
        trace!("Parsing builder options.");
        for x in nested {
            match *x {
                syn::NestedMetaItem::MetaItem(ref meta_item) => {
                    self.parse_builder_options_metaItem(meta_item)
                },
                syn::NestedMetaItem::Literal(ref lit) => {
                    error!("Expected NestedMetaItem::MetaItem, found `{:?}`.", x);
                    panic!("Could not parse builder option `{:?}` {}.",
                           lit,
                           self.where_diagnostics());
                }
            }
        }
    }

    #[allow(non_snake_case)]
    fn parse_builder_options_metaItem(&mut self, meta_item: &syn::MetaItem) {
        trace!("Parsing MetaItem `{:?}`", meta_item);
        match *meta_item {
            syn::MetaItem::Word(ref ident) => {
                self.parse_builder_options_word(ident)
            },
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                self.parse_builder_options_nameValue(ident, lit)
            },
            syn::MetaItem::List(ref ident, ref nested_attrs) => {
                self.parse_builder_options_list(ident, nested_attrs)
            }
        }
    }

    /// e.g `private` in `#[builder(private)]`
    fn parse_builder_options_word(&mut self, ident: &syn::Ident) {
        trace!("Parsing word `{}`", ident.as_ref());
        match ident.as_ref() {
            "public" => {
                self.setter_public(true)
            },
            "private" => {
                self.setter_public(false)
            },
            "setter" => {
                // setter implicitly enabled
                self.setter_enabled(true)
            },
            "try_setter" => {
                self.try_setter(true)
            }
            "default" => {
                self.default_expression(DefaultExpression::Trait)
            },
            "no_std" => {
                if self.mode.struct_mode() {
                    self.no_std(true)
                } else {
                    panic!("Support for `#![no_std]` can only be set on the struct level \
                            (but found {}).", self.where_diagnostics())
                }
            },
            _ => {
                panic!("Unknown option `{}` {}", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    /// e.g `name="FooBuilder"` in `#[builder(name="FooBuilder")]`
    #[allow(non_snake_case)]
    fn parse_builder_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Parsing named value `{}` = `{:?}`", ident.as_ref(), lit);
        match ident.as_ref() {
            "pattern" => {
                self.parse_builder_pattern(lit)
            },
            "name" => {
                self.mode.parse_builder_name(lit)
            },
            "default" => {
                self.parse_default_expression(lit)
            },
            _ => {
                panic!("Unknown option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    /// e.g `setter(skip)` in `#[builder(setter(skip))]`
    #[allow(non_snake_case)]
    fn parse_builder_options_list(&mut self, ident: &syn::Ident, nested: &[syn::NestedMetaItem]) {
        trace!("Parsing list `{}({:?})`", ident.as_ref(), nested);
        match ident.as_ref() {
            "setter" => {
                self.parse_setter_options(nested);
                // setter implicitly enabled
                if self.setter_enabled.is_none() {
                    self.setter_enabled(true);
                }
            },
            "build_fn" => {
                self.mode.parse_build_fn_options(nested)
            },
            "derive" => {
                self.mode.parse_derive(nested);
            }
            "field" => {
                self.parse_field_options(nested);
            }
            _ => {
                panic!("Unknown option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    fn parse_field_options(&mut self, nested: &[syn::NestedMetaItem]) {
        trace!("Parsing field options.");
        for x in nested {
            match *x {
                syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                    match ident.as_ref() {
                        "private" => self.field_public(false),
                        "public" => self.field_public(true),
                        _ => panic!("Unknown field word `{:?}`. {}", ident, self.where_diagnostics())
                    }
                },
                _ => panic!("Unknown field option `{:?}`. {}", x, self.where_diagnostics())
            }
        }
    }

    /// e.g `skip` in `#[builder(setter(skip))]`
    #[allow(non_snake_case)]
    fn parse_setter_options(&mut self, nested: &[syn::NestedMetaItem]) {
        trace!("Parsing setter options.");
        for x in nested {
            match *x {
                syn::NestedMetaItem::MetaItem(ref meta_item) => {
                    self.parse_setter_options_metaItem(meta_item);
                },
                syn::NestedMetaItem::Literal(ref lit) => {
                    error!("Expected NestedMetaItem::MetaItem, found `{:?}`.", x);
                    panic!("Could not parse builder option `{:?}` {}.",
                           lit,
                           self.where_diagnostics());
                }
            }
        }
    }

    #[allow(non_snake_case)]
    fn parse_setter_options_metaItem(&mut self, meta_item: &syn::MetaItem) {
        trace!("Setter Options - Parsing MetaItem `{:?}`.", meta_item);
        match *meta_item {
            syn::MetaItem::Word(ref ident) => {
                self.parse_setter_options_word(ident)
            },
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                self.parse_setter_options_nameValue(ident, lit)
            },
            syn::MetaItem::List(ref ident, ref nested_attrs) => {
                self.parse_setter_options_list(ident, nested_attrs)
            }
        }
    }

    /// e.g `private` in `#[builder(setter(private))]`
    fn parse_setter_options_word(&mut self, ident: &syn::Ident) {
        trace!("Setter Options - Parsing word `{}`", ident.as_ref());
        match ident.as_ref() {
            "public" => {
                self.setter_public(true)
            },
            "private" => {
                self.setter_public(false)
            },
            "skip" => {
                self.setter_enabled(false)
            },
            "into" => {
                self.setter_into(true)
            }
            _ => {
                panic!("Unknown setter option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        };
    }

    /// e.g `prefix="with"` in `#[builder(setter(prefix="with"))]`
    #[allow(non_snake_case)]
    fn parse_setter_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Setter Options - Parsing named value `{}` = `{:?}`", ident.as_ref(), lit);
        match ident.as_ref() {
            "prefix" => {
                self.parse_setter_prefix(lit)
            },
            "name" => {
                self.parse_setter_name(lit)
            },
            "skip" => {
                self.parse_setter_skip(lit)
            },
            _ => {
                panic!("Unknown setter option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    /// e.g `setter(skip)` in `#[builder(setter(skip))]`
    #[allow(non_snake_case)]
    fn parse_setter_options_list(&mut self, ident: &syn::Ident, nested: &[syn::NestedMetaItem]) {
        trace!("Setter Options - Parsing list `{}({:?})`", ident.as_ref(), nested);
        match ident.as_ref() {
            _ => {
                panic!("Unknown option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    fn parse_setter_prefix(&mut self, lit: &syn::Lit) {
        trace!("Parsing prefix `{:?}`", lit);
        let value = parse_lit_as_string(lit).unwrap();
        self.setter_prefix(value.clone());
    }

    fn parse_setter_name(&mut self, lit: &syn::Lit) {
        trace!("Parsing name `{:?}`", lit);
        let value = parse_lit_as_string(lit).unwrap();
        if self.mode.struct_mode() {
            panic!("Setter names can only be set on the field level \
                    (but found {}).", self.where_diagnostics())
        } else {
            self.setter_name(value.clone());
        }
    }

    fn parse_default_expression(&mut self, lit: &syn::Lit) {
        trace!("Parsing default expression `{:?}`", lit);
        let value = parse_lit_as_string(lit).unwrap();
        self.default_expression(DefaultExpression::Explicit(value.clone()));
    }

    fn parse_builder_pattern(&mut self, lit: &syn::Lit) {
        trace!("Parsing pattern `{:?}`", lit);
        let value = parse_lit_as_string(lit).unwrap();
        match value.as_ref() {
            "owned" => {
                self.builder_pattern(BuilderPattern::Owned)
            },
            "mutable" => {
                self.builder_pattern(BuilderPattern::Mutable)
            },
            "immutable" => {
                self.builder_pattern(BuilderPattern::Immutable)
            },
            _ => {
                panic!("Unknown pattern value `{}` {}.", value, self.where_diagnostics())
            }
        };
    }

    fn parse_setter_skip(&mut self, skip: &syn::Lit) {
        trace!("Parsing skip setter `{:?}`", skip);
        self.setter_enabled(!parse_lit_as_bool(skip).unwrap());
    }

    /// Provide a diagnostic _where_-clause for panics.
    ///
    /// Delegete to the `OptionsBuilderMode`.
    fn where_diagnostics(&self) -> String {
        self.mode.where_diagnostics()
    }
}

fn parse_lit_as_string(lit: &syn::Lit) -> Result<&String, String> {
    if let syn::Lit::Str(ref value, _str_style) = *lit {
        Ok(value)
    } else {
        Err(format!("Unable to interpret as string `{:?}`.", lit))
    }
}

fn parse_lit_as_bool(lit: &syn::Lit) -> Result<bool, String> {
    if let syn::Lit::Bool(ref value) = *lit {
        Ok(*value)
    } else {
        parse_lit_as_string(lit).map_err(|_| {
            format!("Value must be a bool or string, but found `{:?}`", lit)
        }).and_then(|value| {
            match value.as_ref() {
                "true" => {
                    Ok(true)
                },
                "false" => {
                    Ok(false)
                },
                _ => {
                    Err(format!("Invalid boolean value `{}`, expected `true` or `false`.", value))
                }
            }
        })
    }
}

fn parse_lit_as_path(lit: &syn::Lit) -> Result<syn::Path, String> {
    syn::parse_path(parse_lit_as_string(lit)?)
        .or_else(|_| Err(format!("Unable to interpret as path `{:?}`.", lit)))
}
