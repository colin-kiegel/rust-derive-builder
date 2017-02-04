use syn;
use quote;

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
pub struct Options {
    // e.g. `#[builder]` (defaults to true)
    setter_enabled: bool,
    // e.g. `#[builder(pattern="owned")]` (defaults to mutable)
    setter_pattern: SetterPattern,
    // e.g. `#[builder(prefix="with")]` (defaults to None)
    setter_prefix: String,
    // e.g. `#[builder(private)]` (defaults to public)
    setter_public: bool,
}

impl Options {
    pub fn setter_enabled(&self) -> bool {
        self.setter_enabled
    }

    pub fn setter_pattern(&self) -> &SetterPattern {
        &self.setter_pattern
    }

    pub fn setter_visibility(&self) -> Option<quote::Tokens> {
        if self.setter_public {
            Some(quote!(pub))
        } else {
            None
        }
    }

    pub fn setter_prefix(&self) -> &str {
        &self.setter_prefix
    }
}

impl<'a, T> From<T> for Options where
    T: IntoIterator<Item=&'a syn::Attribute>
{
    fn from(attributes: T) -> Self {
        trace!("Parsing struct attributes.");
        let mut builder = OptionsBuilder::<StructMode>::default();
        builder.parse_attributes(attributes);

        builder.into()
    }
}

pub trait OptionsBuilderMode {}

#[derive(Default)]
pub struct StructMode;

impl OptionsBuilderMode for StructMode {}

#[derive(Default)]
pub struct FieldMode;

impl OptionsBuilderMode for FieldMode {}

#[derive(Default, Debug)]
pub struct OptionsBuilder<Mode: OptionsBuilderMode> {
    setter_enabled: Option<bool>,
    setter_pattern: Option<SetterPattern>,
    setter_prefix: Option<String>,
    setter_public: Option<bool>,
    mode: Mode,
}

impl From<OptionsBuilder<StructMode>> for Options {
    fn from(b: OptionsBuilder<StructMode>) -> Options {
        Options {
            setter_enabled: b.setter_enabled.unwrap_or(true),
            setter_pattern: b.setter_pattern.unwrap_or_default(),
            setter_prefix: b.setter_prefix.unwrap_or_default(),
            setter_public: b.setter_public.unwrap_or(true),
        }
    }
}

impl OptionsBuilder<FieldMode> {
    pub fn with_struct_options<'a>(&self, o: &'a Options) -> Options {
        Options {
            setter_enabled: self.setter_enabled.unwrap_or(o.setter_enabled),
            setter_pattern: self.setter_pattern.clone().unwrap_or(o.setter_pattern.clone()),
            setter_prefix: self.setter_prefix.clone().unwrap_or(o.setter_prefix.clone()),
            setter_public: self.setter_public.unwrap_or(o.setter_public),
        }
    }
}

impl<Mode> OptionsBuilder<Mode> where
    Mode: OptionsBuilderMode
{
    fn setter_enabled(&mut self, x: bool) -> &mut Self {
        if self.setter_enabled.is_some() {
            warn!("Setter enabled already defined as {:?}, new value is {:?}.",
                self.setter_enabled, x);
        }
        self.setter_enabled = Some(x);
        self
    }

    fn setter_pattern(&mut self, x: SetterPattern) -> &mut Self {
        if self.setter_pattern.is_some() {
            warn!("Setter pattern already defined as {:?}, new value is {:?}.",
                self.setter_pattern, x);
        }
        self.setter_pattern = Some(x);
        self
    }

    fn setter_public(&mut self, x: bool) -> &mut Self {
        if self.setter_public.is_some() {
            warn!("Setter visibility already defined as {:?}, new value is {:?}.",
                self.setter_public, x);
        }
        self.setter_public = Some(x);
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
        trace!("Parsing attribute {:?}.", attr);
        if attr.style != syn::AttrStyle::Outer || attr.is_sugared_doc {
            trace!("Ignoring attribute (outer or sugared doc).");
            return
        }

        const BUILDER_ATTRIBUTE_IDENT: &'static str = "builder";

        // e.g. `#[builder]`
        if let syn::MetaItem::Word(ref ident) = attr.value {
            if ident == BUILDER_ATTRIBUTE_IDENT {
                self.setter_enabled(true);
                return
            }
        }

        // e.g. `#[builder(...)]`
        if let syn::MetaItem::List(ref ident, ref nested_attrs) = attr.value {
            if ident == BUILDER_ATTRIBUTE_IDENT {
                self.setter_enabled(true);
                self.parse_setter_options(nested_attrs);
                return
            }
        }
        debug!("Ignoring attribute.");
    }

    fn parse_setter_options(&mut self, nested: &[syn::NestedMetaItem]) {
        trace!("Parsing setter options.");
        for x in nested {
            if let syn::NestedMetaItem::MetaItem(ref meta_item) = *x {
                self.parse_setter_options_metaItem(meta_item)
            } else {
                panic!("Expected NestedMetaItem::MetaItem, found {:?}", x)
            }
        }
    }

    #[allow(non_snake_case)]
    fn parse_setter_options_metaItem(&mut self, meta_item: &syn::MetaItem) {
        trace!("Parsing MetaItem {:?}", meta_item);
        match *meta_item {
            syn::MetaItem::Word(ref ident) => {
                self.parse_setter_options_word(ident)
            },
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                self.parse_setter_options_nameValue(ident, lit)
            },
            _ => {
                panic!("Expected MetaItem::Word/NameValue, found {:?}", meta_item)
            }
        }
    }

    /// e.g `private` in `#[builder(private)]`
    fn parse_setter_options_word(&mut self, ident: &syn::Ident) {
        trace!("Parsing word {:?}", ident);
        match ident.as_ref() {
            "public" => {
                self.setter_public(true)
            },
            "private" => {
                self.setter_public(false)
            },
            _ => {
                panic!("Unknown option {:?}", ident)
            }
        };
    }

    /// e.g `prefix="with"` in `#[builder(prefix="with")]`
    #[allow(non_snake_case)]
    fn parse_setter_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Parsing named value {:?} = {:?}", ident, lit);
        match ident.as_ref() {
            "prefix" => {
                self.parse_setter_prefix(lit)
            },
            "pattern" => {
                self.parse_setter_pattern(lit)
            },
            _ => {
                panic!("Unknown option {:?}", ident)
            }
        }
    }

    fn parse_setter_prefix(&mut self, lit: &syn::Lit) {
        trace!("Parsing prefix {:?}", lit);
        let value = parse_lit_as_cooked_string(lit);
        debug!("Setting prefix {:?}", value);
        self.setter_prefix = Some(value.clone());
    }

    fn parse_setter_pattern(&mut self, lit: &syn::Lit) {
        trace!("Parsing pattern {:?}", lit);
        let value = parse_lit_as_cooked_string(lit);
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
                panic!("Unknown option {:?}", value)
            }
        };
    }
}

fn parse_lit_as_cooked_string(lit: &syn::Lit) -> &String {
    if let syn::Lit::Str(ref value, str_style) = *lit {
        if str_style != syn::StrStyle::Cooked {
            panic!("Value must be a *standard* string, but found {:?}", lit);
        }
        value
    } else {
        panic!("Value must be a string, but found {:?}", lit);
    }
}
