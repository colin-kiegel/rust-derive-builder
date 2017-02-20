use syn;
use deprecation_notes::DeprecationNotes;

#[derive(PartialEq, Debug, Clone)]
pub enum SetterPattern {
    Owned,
    Mutable,
    Immutable
}

impl Default for SetterPattern {
    fn default() -> SetterPattern {
        SetterPattern::Mutable
    }
}

#[derive(Debug, Clone)]
pub struct StructOptions {
    /// defaults to format!("{}Builder", struct_name)
    builder_name: String,
    /// defaults to struct_vis
    builder_vis: syn::Visibility,
    /// see below
    field_defaults: FieldOptions,
}

#[derive(Debug, Clone)]
pub struct FieldOptions {
    /// currently hard-wired to true
    setter_enabled: bool,
    /// e.g. `#[builder(pattern="owned")]` (defaults to mutable)
    setter_pattern: SetterPattern,
    /// e.g. `#[builder(setter_prefix="with")]` (defaults to None)
    setter_prefix: String,
    /// e.g. `#[builder(private)]` (defaults to public)
    setter_vis: syn::Visibility,
    /// the _original_ field name
    field_name: String,
    /// we collect all deprecation notices that we want to send to the user (later).
    deprecation_notes: DeprecationNotes,
}

impl StructOptions {
    pub fn field_defaults(&self) -> &FieldOptions {
        &self.field_defaults
    }

    pub fn builder_visibility(&self) -> &syn::Visibility {
        &self.builder_vis
    }

    pub fn builder_name(&self) -> &str {
        &self.builder_name
    }
}

impl FieldOptions {
    pub fn setter_enabled(&self) -> bool {
        self.setter_enabled
    }

    pub fn setter_pattern(&self) -> &SetterPattern {
        &self.setter_pattern
    }

    pub fn setter_visibility(&self) -> &syn::Visibility {
        &self.setter_vis
    }

    pub fn setter_prefix(&self) -> &str {
        &self.setter_prefix
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    pub fn deprecation_notes(&self) -> &DeprecationNotes {
        &self.deprecation_notes
    }
}

impl<'a> From<&'a syn::MacroInput> for StructOptions {
    fn from(ast: &'a syn::MacroInput) -> Self {
        trace!("Parsing struct attributes.");
        let mut builder = OptionsBuilder::<StructMode>::default();
        builder.parse_attributes(&ast.attrs);

        builder.mode.struct_name = ast.ident.as_ref().to_string();
        builder.mode.struct_vis = Some(ast.vis.clone());

        builder.into()
    }
}

pub trait OptionsBuilderMode {
    fn parse_builder_name(&mut self, lit: &syn::Lit);
}

#[derive(Default)]
pub struct StructMode {
    builder_name: Option<String>,
    builder_vis: Option<syn::Visibility>,
    struct_name: String,
    struct_vis: Option<syn::Visibility>,
}

impl OptionsBuilderMode for StructMode {
    fn parse_builder_name(&mut self, name: &syn::Lit) {
        trace!("Parsing builder name `{:?}`", name);
        let value = parse_lit_as_cooked_string(name).unwrap();
        self.builder_name = Some(value.clone());
    }
}

#[derive(Default)]
pub struct FieldMode;

impl OptionsBuilderMode for FieldMode {
    fn parse_builder_name(&mut self, _name: &syn::Lit) {
        panic!("Builder name can only be set on the stuct level")
    }
}

#[derive(Default, Debug)]
pub struct OptionsBuilder<Mode: OptionsBuilderMode> {
    setter_enabled: Option<bool>,
    setter_pattern: Option<SetterPattern>,
    setter_prefix: Option<String>,
    setter_vis: Option<syn::Visibility>,
    deprecation_notes: DeprecationNotes,
    mode: Mode,
}

impl From<OptionsBuilder<StructMode>> for StructOptions {
    fn from(b: OptionsBuilder<StructMode>) -> StructOptions {
        let field_defaults = FieldOptions {
            setter_enabled: b.setter_enabled.unwrap_or(true),
            setter_pattern: b.setter_pattern.unwrap_or_default(),
            setter_prefix: b.setter_prefix.unwrap_or_default(),
            setter_vis: b.setter_vis.unwrap_or(syn::Visibility::Public),
            deprecation_notes: b.deprecation_notes,
            field_name: String::from(""),
        };

        StructOptions {
            field_defaults: field_defaults,
            builder_name: b.mode.builder_name.unwrap_or(format!("{}Builder", b.mode.struct_name)),
            builder_vis: b.mode.builder_vis.unwrap_or(
                b.mode.struct_vis.expect("Struct visibility must be initialized")
            )
        }
    }
}

impl OptionsBuilder<FieldMode> {
    pub fn build<'a, T>(&self, name: T, struct_opts: &'a StructOptions) -> FieldOptions where
        T: Into<String>
    {
        let x = struct_opts.field_defaults();
        FieldOptions {
            setter_enabled: self.setter_enabled.unwrap_or(x.setter_enabled),
            setter_pattern: self.setter_pattern.clone().unwrap_or(x.setter_pattern.clone()),
            setter_prefix: self.setter_prefix.clone().unwrap_or(x.setter_prefix.clone()),
            setter_vis: self.setter_vis.as_ref().unwrap_or(&x.setter_vis).clone(),
            field_name: name.into(),
            deprecation_notes: self.deprecation_notes.clone(), // don't inherit this
        }
    }
}

impl<Mode> OptionsBuilder<Mode> where
    Mode: OptionsBuilderMode
{
    fn setter_enabled(&mut self, x: bool) -> &mut Self {
        if self.setter_enabled.is_some() {
            warn!("Setter enabled already defined as `{:?}`, new value is `{:?}`.",
                self.setter_enabled, x);
        }
        self.setter_enabled = Some(x);
        self
    }

    fn setter_pattern(&mut self, x: SetterPattern) -> &mut Self {
        if self.setter_pattern.is_some() {
            warn!("Setter pattern already defined as `{:?}`, new value is `{:?}`.",
                self.setter_pattern, x);
        }
        self.setter_pattern = Some(x);
        self
    }

    fn setter_public(&mut self, x: bool) -> &mut Self {
        if self.setter_vis.is_some() {
            warn!("Setter visibility already defined as `{:?}`, new value is `{:?}`.",
                self.setter_vis, x);
        }
        self.setter_vis = Some(syn::Visibility::Public);
        self
    }

    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self {
        self.deprecation_notes.push(x.into());
        self
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
            syn::MetaItem::List(ref _ident, ref nested_attrs)
            => {
                self.setter_enabled(true);
                self.parse_builder_options(nested_attrs);
                return
            },
            syn::MetaItem::Word(_) |
            syn::MetaItem::NameValue(_, _) => {
                error!("Expected MetaItem::List, found `{:?}`", attr.value);
                panic!("Could not parse builder options.");
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
                    panic!("Could not parse builder option `{:?}`.", lit);
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
        trace!("Parsing word `{:?}`", ident);
        match ident.as_ref() {
            "public" => {
                self.setter_public(true)
            },
            "private" => {
                self.setter_public(false)
            },
            "setter" => {
                self.setter_enabled(true)
            },
            _ => {
                panic!("Unknown option `{:?}`", ident)
            }
        };
    }

    /// e.g `setter_prefix="with"` in `#[builder(setter_prefix="with")]`
    #[allow(non_snake_case)]
    fn parse_builder_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Parsing named value `{:?}` = `{:?}`", ident, lit);
        match ident.as_ref() {
            "setter_prefix" => {
                let val = quote!(#lit);
                self.push_deprecation_note(format!(
                    "warning: deprecated syntax `#[builder(setter_prefix={})]`, \
                     please use `#[builder(setter(prefix={}))]` instead!",
                    val, val));
                self.parse_setter_prefix(lit)
            },
            "pattern" => {
                self.parse_setter_pattern(lit)
            },
            "name" => {
                self.mode.parse_builder_name(lit)
            },
            _ => {
                panic!("Unknown option `{}`.", ident.as_ref())
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
                self.parse_setter_options(nested)
            },
            _ => {
                panic!("Unknown option `{}`.", ident.as_ref())
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
                    // setters implicitly enabled
                    if self.setter_enabled.is_none() {
                        self.setter_enabled(true);
                    }
                },
                syn::NestedMetaItem::Literal(ref _lit) => {
                    // setters explicitly enabled
                    self.setter_enabled(true);
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
        trace!("Setter Options - Parsing word `{:?}`", ident);
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
                panic!("Unknown setter option `{:?}`.", ident)
            }
        };
    }

    /// e.g `prefix="with"` in `#[builder(setter(prefix="with"))]`
    #[allow(non_snake_case)]
    fn parse_setter_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Setter Options - Parsing named value `{:?}` = `{:?}`", ident, lit);
        match ident.as_ref() {
            "prefix" => {
                self.parse_setter_prefix(lit)
            },
            "skip" => {
                self.parse_setter_skip(lit)
            },
            _ => {
                panic!("Unknown setter option `{}`.", ident.as_ref())
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
                panic!("Unknown option `{}`.", ident.as_ref())
            }
        }
    }

    fn parse_setter_prefix(&mut self, lit: &syn::Lit) {
        trace!("Parsing prefix `{:?}`", lit);
        let value = parse_lit_as_cooked_string(lit).unwrap();
        self.setter_prefix = Some(value.clone());
    }

    fn parse_setter_pattern(&mut self, lit: &syn::Lit) {
        trace!("Parsing pattern `{:?}`", lit);
        let value = parse_lit_as_cooked_string(lit).unwrap();
        match value.as_ref() {
            "owned" => {
                self.setter_pattern(SetterPattern::Owned)
            },
            "mutable" => {
                self.setter_pattern(SetterPattern::Mutable)
            },
            "immutable" => {
                self.setter_pattern(SetterPattern::Immutable)
            },
            _ => {
                panic!("Unknown pattern value `{}`.", value)
            }
        };
    }

    fn parse_setter_skip(&mut self, skip: &syn::Lit) {
        trace!("Parsing skip setter `{:?}`", skip);
        self.setter_enabled(!parse_lit_as_bool(skip).unwrap());
    }
}

fn parse_lit_as_cooked_string(lit: &syn::Lit) -> Result<&String, String> {
    if let syn::Lit::Str(ref value, str_style) = *lit {
        if str_style != syn::StrStyle::Cooked {
            return Err(format!("Non-standard string found `{:?}`", lit))
        }
        Ok(value)
    } else {
        Err(format!("Unable to interpret as string `{:?}`.", lit))
    }
}

fn parse_lit_as_bool(lit: &syn::Lit) -> Result<bool, String> {
    if let syn::Lit::Bool(ref value) = *lit {
        Ok(*value)
    } else {
        parse_lit_as_cooked_string(lit).map_err(|_| {
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
