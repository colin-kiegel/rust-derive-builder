use syn;
use quote;

#[derive(PartialEq, Debug)]
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

#[derive(Debug)]
pub struct Options {
    /// e.g. `#[setters]` (defaults to true)
    setter_enabled: bool,
    /// e.g. `#[setters(owned)]` (defaults to mutable)
    setter_pattern: SetterPattern,
    /// e.g. `#[setters(prefix="with")]` (defaults to None)
    setter_prefix: Option<String>,
    /// e.g. `#[setters(private)]` (defaults to public)
    setter_public: bool,
    /// e.g. `#[setters(options="implicit")]` (defaults to explicit)
    setter_implicit_options: bool,
    /// e.g. `#[getters]` (defaults to false)
    getter_enabled: bool,
    /// e.g. `#[getters(prefix="with")]` (defaults to None)
    getter_prefix: Option<String>,
    /// e.g. `#[getters(private)]` (defaults to public)
    getter_public: bool,
}

impl Options {
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

    pub fn setter_enabled(&self) -> bool {
        self.setter_enabled
    }
}

impl<T> From<T> for Options where
    T: IntoIterator<Item=syn::Attribute>
{
    fn from(attributes: T) -> Self {
        trace!("Parsing struct attributes.");
        let mut ob = OptionsBuilder::default();

        for attr in attributes {
            ob.parse_attribute(attr);
        }

        ob.into()
    }
}

#[derive(Default, Debug)]
pub struct OptionsBuilder {
    setter_enabled: Option<bool>,
    setter_pattern: Option<SetterPattern>,
    setter_prefix: Option<String>,
    setter_public: Option<bool>,
    setter_implicit_options: Option<bool>,
    getter_enabled: Option<bool>,
    getter_prefix: Option<String>,
    getter_public: Option<bool>,
}

impl From<OptionsBuilder> for Options {
    fn from(b: OptionsBuilder) -> Options {
        Options {
            setter_enabled: b.setter_enabled.unwrap_or(true),
            setter_pattern: b.setter_pattern.unwrap_or_default(),
            setter_prefix: b.setter_prefix,
            setter_public: b.setter_public.unwrap_or(true),
            setter_implicit_options: b.setter_implicit_options.unwrap_or(false),
            getter_enabled: b.getter_enabled.unwrap_or(false),
            getter_prefix: b.getter_prefix,
            getter_public: b.getter_public.unwrap_or(true),
        }
    }
}

impl OptionsBuilder {
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

    fn setter_implicit_options(&mut self, x: bool) -> &mut Self {
        if self.setter_implicit_options.is_some() {
            warn!("Setter implicit options already defined as {:?}, new value is {:?}.",
                self.setter_implicit_options, x);
        }
        self.setter_implicit_options = Some(x);
        self
    }

    fn getter_enabled(&mut self, x: bool) -> &mut Self {
        if self.getter_enabled.is_some() {
            warn!("Getter enabled already defined as {:?}, new value is {:?}.",
                self.getter_enabled, x);
        }
        self.getter_enabled = Some(x);
        self
    }

    fn parse_attribute(&mut self, attr: syn::Attribute) {
        trace!("Parsing attribute {:?}.", attr);
        if attr.style != syn::AttrStyle::Outer || attr.is_sugared_doc {
            trace!("Ignoring attribute (outer or sugared doc).");
            return
        }

        // e.g. `#[setters]`
        if let syn::MetaItem::Word(ref ident) = attr.value {
            match ident.as_ref() {
                "setters" => {
                        self.setter_enabled(true);
                        return
                    },
                "getters" => {
                        self.getter_enabled(true);
                        return
                    },
                _ => {}
            }
        }

        // e.g. `#[setters(...)]`
        if let syn::MetaItem::List(ref ident, ref nested_attrs) = attr.value {
            match ident.as_ref() {
                "setters" => {
                        self.setter_enabled(true);
                        self.parse_setters_options(nested_attrs);
                        return
                    },
                "getters" => {
                        self.getter_enabled(true);
                        self.parse_getters_options(nested_attrs);
                        return
                    },
                _ => {}
            }
        }
        debug!("Ignoring attribute.");
    }

    fn parse_setters_options(&mut self, nested: &[syn::NestedMetaItem]) {
        trace!("Parsing setter options.");
        for x in nested {
            if let syn::NestedMetaItem::MetaItem(ref meta_item) = *x {
                self.parse_setters_options_metaItem(meta_item)
            } else {
                panic!("Expected NestedMetaItem::MetaItem, found {:?}", x)
            }
        }
    }

    #[allow(non_snake_case)]
    fn parse_setters_options_metaItem(&mut self, meta_item: &syn::MetaItem) {
        trace!("Parsing MetaItem {:?}", meta_item);
        match *meta_item {
            syn::MetaItem::Word(ref ident) => {
                self.parse_setters_options_word(ident)
            },
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                self.parse_setters_options_nameValue(ident, lit)
            },
            _ => {
                panic!("Expected MetaItem::Word/NameValue, found {:?}", meta_item)
            }
        }
    }

    /// e.g `owned` in `#[setters(owned)]`
    fn parse_setters_options_word(&mut self, ident: &syn::Ident) {
        trace!("Parsing word {:?}", ident);
        match ident.as_ref() {
            "owned" => {
                self.setter_pattern(SetterPattern::Owned)
            },
            "mutable" => {
                self.setter_pattern(SetterPattern::Mutable)
            },
            "immutable" => {
                self.setter_pattern(SetterPattern::Immutable)
            },
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

    /// e.g `prefix="with"` in `#[setters(prefix="with")]`
    #[allow(non_snake_case)]
    fn parse_setters_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Parsing named value {:?} = {:?}", ident, lit);
        match ident.as_ref() {
            "prefix" => {
                self.parse_setters_prefix(lit)
            },
            "option" => {
                self.parse_setters_implicit_options(lit)
            }
            _ => {
                panic!("Unknown option {:?}", ident)
            }
        }
    }

    fn parse_setters_prefix(&mut self, lit: &syn::Lit) {
        trace!("Parsing prefix {:?}", lit);
        let value = parse_lit_as_cooked_string(lit);
        debug!("Setting prefix {:?}", value);
        self.setter_prefix = Some(value.clone());
    }

    fn parse_setters_implicit_options(&mut self, lit: &syn::Lit) {
        trace!("Parsing implicit options {:?}", lit);
        let value = parse_lit_as_cooked_string(lit);
        match value.as_str() {
            "implicit" => self.setter_implicit_options(true),
            "explicit" => self.setter_implicit_options(true),
            _ => panic!("Unknown value {:?}", value),
        };
    }

    fn parse_getters_options(&mut self, nested: &[syn::NestedMetaItem]) -> &mut Self {
        panic!("TODO: parse getter options -> {:?}", nested);
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
