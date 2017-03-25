use syn;
use derive_builder_core::BuilderPattern;

mod field_mode;
mod field_options;
mod struct_mode;
mod struct_options;

pub use self::field_mode::FieldMode;
pub use self::field_options::FieldOptions;
pub use self::struct_mode::StructMode;
pub use self::struct_options::StructOptions;
use self::field_options::DefaultExpression;

/// Get the tuple of `StructOptions` and field defaults (`OptionsBuilder<FieldMode>`) from the AST.
pub fn struct_options_from(ast: &syn::MacroInput) -> (StructOptions, OptionsBuilder<FieldMode>) {
    OptionsBuilder::<StructMode>::parse(ast).into()
}

///  Get the `FieldOptions` for a field with respect to some custom default values.
pub fn field_options_from(f: syn::Field,
                          defaults: &OptionsBuilder<FieldMode>)
                          -> FieldOptions {
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
    default_expression: Option<DefaultExpression>,
    mode: Mode,
}

/// Certain attributes need to be handled differently for `StructOptions` and `FieldOptions`.
pub trait OptionsBuilderMode: ::std::fmt::Debug {
    fn parse_builder_name(&mut self, lit: &syn::Lit);
    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self;
    /// Provide a diagnostic _where_-clause for panics.
    fn where_diagnostics(&self) -> String;
}

impl<Mode> From<Mode> for OptionsBuilder<Mode> {
    fn from(mode: Mode) -> OptionsBuilder<Mode> {
        OptionsBuilder {
            builder_pattern: None,
            setter_enabled: None,
            setter_prefix: None,
            setter_name: None,
            setter_vis: None,
            default_expression: None,
            mode: mode,
        }
    }
}

macro_rules! impl_setter {
    (
        ident: $ident:ident,
        desc: $desc:expr,
        map: |$x:ident: $ty:ty| {$( $map:tt )*},
    ) => {
        impl_setter!{
            ident: $ident for $ident,
            desc: $desc,
            map: |$x: $ty| {$( $map )*},
        }
    };
    (
        ident: $setter:ident for $field:ident,
        desc: $desc:expr,
        map: |$x:ident: $ty:ty| {$( $map:tt )*},
    ) => {
        fn $setter(&mut self, $x: $ty) -> &mut Self {
            if let Some(ref current) = self.$field {
                panic!("Failed to set {} to `{:?}` (already defined as `{:?}`) {}.",
                    $desc,
                    $x,
                    current,
                    self.where_diagnostics());
            }
            self.$field = Some({$( $map )*});
            self
        }
    }
}

impl<Mode> OptionsBuilder<Mode> where
    Mode: OptionsBuilderMode
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
        ident: setter_public for setter_vis,
        desc: "setter visibility",
        map: |x: bool| { if x { syn::Visibility::Public } else { syn::Visibility::Inherited } },
    }

    impl_setter!{
        ident: default_expression,
        desc: "default expression",
        map: |x: DefaultExpression| { x },
    }

    pub fn parse_attributes<'a, T>(&mut self, attributes: T) -> &mut Self where
        T: IntoIterator<Item=&'a syn::Attribute>
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
            "default" => {
                self.default_expression(DefaultExpression::Trait)
            },
            _ => {
                panic!("Unknown option `{}` {}", ident.as_ref(), self.where_diagnostics())
            }
        };
    }

    /// e.g `setter_prefix="with"` in `#[builder(setter_prefix="with")]`
    #[allow(non_snake_case)]
    fn parse_builder_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Parsing named value `{}` = `{:?}`", ident.as_ref(), lit);
        match ident.as_ref() {
            "setter_prefix" => {
                let val = quote!(#lit);
                self.mode.push_deprecation_note(format!(
                    "warning: deprecated syntax `#[builder(setter_prefix={})]`, \
                     please use `#[builder(setter(prefix={}))]` instead!",
                    val, val));
                self.parse_setter_prefix(lit)
            },
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
    fn parse_builder_options_list(
        &mut self,
        ident: &syn::Ident,
        nested: &[syn::NestedMetaItem]
    ) {
        trace!("Parsing list `{}({:?})`", ident.as_ref(), nested);
        match ident.as_ref() {
            "setter" => {
                self.parse_setter_options(nested);
                // setter implicitly enabled
                if self.setter_enabled.is_none() {
                    self.setter_enabled(true);
                }
            },
            _ => {
                panic!("Unknown option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    /// e.g `skip` in `#[builder(setter(skip))]`
    #[allow(non_snake_case)]
    fn parse_setter_options(
        &mut self,
        nested: &[syn::NestedMetaItem]
    ) {
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
    fn parse_setter_options_list(
        &mut self,
        ident: &syn::Ident,
        nested: &[syn::NestedMetaItem]
    ) {
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
        self.setter_prefix = Some(value.clone());
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
