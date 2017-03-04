use syn;
use options::{OptionsBuilder, OptionsBuilderMode, FieldOptions};
use derive_builder_core::DeprecationNotes;

#[derive(Default, Clone)]
pub struct FieldMode {
    field_ident: Option<syn::Ident>,
    field_type: Option<syn::Ty>,
    setter_attrs: Option<Vec<syn::Attribute>>,
    deprecation_notes: DeprecationNotes,
}

impl OptionsBuilder<FieldMode> {
    pub fn parse(f: syn::Field) -> Self {
        let mut builder = Self::default();

        {
            let ident = f.ident.as_ref().expect(&format!("Missing identifier for field of type `{:?}`.", f.ty));
            trace!("Parsing field `{}`.", ident);
        }

        builder.parse_attributes(&f.attrs);
        builder.mode.field_ident = f.ident;
        builder.mode.field_type = Some(f.ty);

        trace!("Filtering attributes for builder field and setter.");
        builder.mode.setter_attrs = Some(f.attrs
            .iter()
            .filter(|a| {
                let keep = filter_attr(a);
                match keep {
                    true => trace!("Keeping attribute `{:?}`", a),
                    false => trace!("Ignoring attribute `{:?}`", a)
                }
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

        let mode = FieldMode {
            field_ident: self.mode.field_ident.or_else(|| defaults.mode.field_ident.clone()),
            field_type: self.mode.field_type.or_else(|| defaults.mode.field_type.clone()),
            setter_attrs: self.mode.setter_attrs.or_else(|| defaults.mode.setter_attrs.clone()),
            deprecation_notes: deprecation_notes,
        };
        OptionsBuilder::<FieldMode> {
            setter_enabled: self.setter_enabled.or_else(|| defaults.setter_enabled),
            builder_pattern: self.builder_pattern.or_else(|| defaults.builder_pattern),
            setter_name: self.setter_name.or_else(|| defaults.setter_name.clone()),
            setter_prefix: self.setter_prefix.or_else(|| defaults.setter_prefix.clone()),
            setter_vis: self.setter_vis.or_else(|| defaults.setter_vis.clone()),
            mode: mode,
        }
    }
}


impl OptionsBuilderMode for FieldMode {
    fn parse_builder_name(&mut self, _name: &syn::Lit) {
        panic!("Builder name can only be set on the stuct level")
    }

    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self {
        self.deprecation_notes.push(x.into());
        self
    }
}

impl From<OptionsBuilder<FieldMode>> for FieldOptions {
    fn from(b: OptionsBuilder<FieldMode>) -> FieldOptions {
        let field_ident = b.mode.field_ident
            .clone()
            .expect("Setter name must be set.");
        let field_type = b.mode.field_type
            .clone()
            .expect(&format!("Setter type must be set for field `{}`.", field_ident));
        let setter_ident = b.setter_name
            .as_ref()
            .map(|name| syn::Ident::new(name.as_str()))
            .unwrap_or_else(|| {
                match b.setter_prefix {
                    Some(ref prefix) if !prefix.is_empty() => {
                        syn::Ident::new(format!("{}_{}", prefix, field_ident))
                    },
                    _ => syn::Ident::new(field_ident.clone()),
                }});

        FieldOptions {
            setter_enabled: b.setter_enabled.unwrap_or(true),
            builder_pattern: b.builder_pattern.clone().unwrap_or_default(),
            setter_ident: setter_ident,
            setter_visibility: b.setter_vis.clone().unwrap_or(syn::Visibility::Public),
            field_ident: field_ident,
            field_type: field_type,
            deprecation_notes: b.mode.deprecation_notes.clone(),
            attrs: b.mode.setter_attrs.clone().unwrap_or_default(),
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
