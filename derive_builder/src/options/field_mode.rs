use syn;
use options::{OptionsBuilder, OptionsBuilderMode, FieldOptions};
use derive_builder_core::{DeprecationNotes, Bindings};

#[derive(Clone, Debug)]
pub struct FieldMode {
    field_ident: syn::Ident,
    field_type: syn::Ty,
    setter_attrs: Option<Vec<syn::Attribute>>,
    deprecation_notes: DeprecationNotes,
    pub use_default_struct: bool,
}

impl Default for FieldMode {
    fn default() -> FieldMode {
        FieldMode {
           field_ident: syn::Ident::new(""),
           field_type: syn::Ty::Never,
           setter_attrs: None,
           deprecation_notes: Default::default(),
           use_default_struct: false,
       }
    }
}

impl OptionsBuilder<FieldMode> {
    pub fn parse(f: syn::Field) -> Self {
        let ident = f.ident.expect(&format!("Missing identifier for field of type `{:?}`.", f.ty));
        trace!("Parsing field `{}`.", ident.as_ref());

        // Note: Set `field_ident` _before_ parsing attributes, for better diagnostics!
        let mut builder = Self::from(FieldMode {
            field_ident: ident,
            field_type: f.ty,
            setter_attrs: None,
            deprecation_notes: Default::default(),
            use_default_struct: false,
        });

        builder.parse_attributes(&f.attrs);

        trace!("Filtering attributes for builder field and setter.");
        builder.mode.setter_attrs = Some(f.attrs
            .iter()
            .filter(|a| {
                let keep = filter_attr(a);
                trace!("{} attribute `{:?}`", if keep { "Keeping" } else { "Ignoring" }, a);
                keep
            })
            .map(|x| x.clone())
            .collect());

        builder
    }

    /// If any field is `None`, we fallback to the supplied default value.
    pub fn with_defaults(self, defaults: &Self) -> Self {
        let mut deprecation_notes = self.mode.deprecation_notes;
        deprecation_notes.extend(&defaults.mode.deprecation_notes);

        // move a nested field out of `self`, if it is `Some(_)` or else clone it from `defaults`
        macro_rules! f {
            ($($field:ident).*) => {
                self.$($field).*.or_else(|| defaults.$($field).*.clone())
            };
        }

        let mode = FieldMode {
            field_ident: self.mode.field_ident,
            field_type: self.mode.field_type,
            setter_attrs: f!(mode.setter_attrs),
            deprecation_notes: deprecation_notes,
            use_default_struct: self.mode.use_default_struct || defaults.mode.use_default_struct,
        };

        OptionsBuilder::<FieldMode> {
            setter_enabled: f!(setter_enabled),
            builder_pattern: f!(builder_pattern),
            setter_name: f!(setter_name),
            setter_prefix: f!(setter_prefix),
            setter_vis: f!(setter_vis),
            field_vis: f!(field_vis),
            default_expression: f!(default_expression),
            setter_into: f!(setter_into),
            try_setter: f!(try_setter),
            no_std: f!(no_std),
            mode: mode,
        }
    }
}


impl OptionsBuilderMode for FieldMode {
    fn parse_builder_name(&mut self, _name: &syn::Lit) {
        panic!("Builder name can only be set on the struct level (but found {}).",
               self.where_diagnostics())
    }

    fn parse_derive(&mut self, _nested: &[syn::NestedMetaItem]) {
        panic!("Derive declarations can only be added on the struct level (but found {}).",
               self.where_diagnostics())
    }

    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self {
        self.deprecation_notes.push(x.into());
        self
    }

    /// Provide a diagnostic _where_-clause for panics.
    fn where_diagnostics(&self) -> String {
        format!("on field `{}`", self.field_ident.as_ref())
    }

    fn struct_mode(&self) -> bool {
        false
    }

    fn parse_build_fn_options(&mut self, _: &[syn::NestedMetaItem]) {
        panic!("Build function options can only be set on the struct level (but found {}).",
               self.where_diagnostics())
    }
}

impl From<OptionsBuilder<FieldMode>> for FieldOptions {
    fn from(b: OptionsBuilder<FieldMode>) -> FieldOptions {
        let field_ident = b.mode.field_ident;
        let field_type = b.mode.field_type;
        let setter_prefix = b.setter_prefix;
        let setter_ident = b.setter_name
            .as_ref()
            .map(|name| syn::Ident::new(name.as_str()))
            .unwrap_or_else(|| {
                match setter_prefix {
                    Some(ref prefix) if !prefix.is_empty() => {
                        syn::Ident::new(format!("{}_{}", prefix, field_ident))
                    },
                    _ => syn::Ident::new(field_ident.clone()),
                }});

        let setter_vis = b.setter_vis.unwrap_or(syn::Visibility::Public);

        let field_vis = b.field_vis.unwrap_or(syn::Visibility::Inherited);

        FieldOptions {
            setter_enabled: b.setter_enabled.unwrap_or(true),
            builder_pattern: b.builder_pattern.unwrap_or_default(),
            setter_ident: setter_ident,
            field_visibility: field_vis,
            setter_visibility: setter_vis,
            field_ident: field_ident,
            field_type: field_type,
            setter_into: b.setter_into.unwrap_or(false),
            try_setter: b.try_setter.unwrap_or(false),
            deprecation_notes: b.mode.deprecation_notes,
            default_expression: b.default_expression,
            use_default_struct: b.mode.use_default_struct,
            bindings: Bindings {
                no_std: b.no_std.unwrap_or(false),
            },
            attrs: b.mode.setter_attrs.unwrap_or_default(),
        }
    }
}

fn filter_attr(attr: &&syn::Attribute) -> bool {
    if attr.style != syn::AttrStyle::Outer {
        return false
    }

    if attr.is_sugared_doc == true {
        if let syn::MetaItem::NameValue(ref ident, _) = attr.value {
            // example:
            // Attribute { style: Outer, value: NameValue(Ident("doc"), Str("/// This is a doc comment for a field", Cooked)), is_sugared_doc: true }
            if ident == "doc" {
                return true
            }
        }
    } else {
        if let syn::MetaItem::List(ref ident, _) = attr.value {
            // example:
            // Attribute { style: Outer, value: List(Ident("allow"), [MetaItem(Word(Ident("non_snake_case")))]), is_sugared_doc: false }
            return match ident.as_ref() {
                "cfg" => true,
                "allow" => true,
                _ => false,
            }
        }
    }
    false
}
