use syn;
use options::{OptionsBuilder, OptionsBuilderMode, parse_lit_as_string, FieldMode, StructOptions};
use derive_builder_core::{DeprecationNotes, Bindings};

#[derive(Debug, Clone)]
pub struct StructMode {
    build_target_name: String,
    build_target_generics: syn::Generics,
    build_target_vis: syn::Visibility,
    builder_name: Option<String>,
    builder_vis: Option<syn::Visibility>,
    derive_traits: Option<Vec<syn::Ident>>,
    deprecation_notes: DeprecationNotes,
    struct_size_hint: usize,
}

impl OptionsBuilder<StructMode> {
    pub fn parse(ast: &syn::MacroInput) -> Self {
        trace!("Parsing struct `{}`.", ast.ident.as_ref());

        // Note: Set `build_target_name` _before_ parsing attributes, for better diagnostics!
        let mut builder = Self::from(StructMode {
            build_target_name: ast.ident.as_ref().to_string(),
            build_target_generics: ast.generics.clone(),
            build_target_vis: ast.vis.clone(),
            builder_name: None,
            builder_vis: None,
            derive_traits: None,
            deprecation_notes: Default::default(),
            struct_size_hint: 0,
        });

        builder.parse_attributes(&ast.attrs);

        builder
    }
}

impl StructMode {
    impl_setter!{
        ident: builder_name,
        desc: "builder name",
        map: |x: String| { x },
    }

    impl_setter!{
        ident: derive_traits,
        desc: "derive traits",
        map: |x: Vec<syn::Ident>| { x },
    }
}

impl OptionsBuilderMode for StructMode {
    fn parse_builder_name(&mut self, name: &syn::Lit) {
        trace!("Parsing builder name `{:?}`", name);
        let value = parse_lit_as_string(name).unwrap();
        self.builder_name(value.clone());
    }

    fn parse_derive(&mut self, nested: &[syn::NestedMetaItem]) {
        let mut traits = vec![];
        for x in nested {
            match *x {
                // We don't allow name-value pairs or further nesting here, so
                // only look for words.
                syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref tr)) => {
                    traits.push(tr.clone())
                }
                _ => {
                    panic!("The derive(...) option should be a list of traits (at {}).",
                           self.where_diagnostics())
                }
            }
        }

        self.derive_traits(traits);
    }

    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self {
        self.deprecation_notes.push(x.into());
        self
    }

    /// Provide a diagnostic _where_-clause for panics.
    fn where_diagnostics(&self) -> String {
        format!("on struct `{}`", self.build_target_name)
    }

    fn struct_mode(&self) -> bool {
        true
    }
}

impl From<OptionsBuilder<StructMode>> for (StructOptions, OptionsBuilder<FieldMode>) {
    fn from(b: OptionsBuilder<StructMode>) -> (StructOptions, OptionsBuilder<FieldMode>) {
        #[cfg(feature = "struct_default")]
        let (field_default_expression, struct_default_expression) = (None, b.default_expression);
        #[cfg(not(feature = "struct_default"))]
        let (field_default_expression, struct_default_expression) = (b.default_expression, None);

        let field_defaults = OptionsBuilder::<FieldMode> {
            setter_enabled: b.setter_enabled,
            builder_pattern: b.builder_pattern,
            setter_name: None,
            setter_prefix: b.setter_prefix,
            setter_vis: b.setter_vis,
            setter_into: b.setter_into,
            try_setter: b.try_setter,
            default_expression: field_default_expression,
            no_std: b.no_std,
            mode: {
                let mut mode = FieldMode::default();
                mode.use_default_struct = struct_default_expression.is_some();
                mode
            },
        };

        let m = b.mode;

        let struct_options = StructOptions {
            builder_ident: syn::Ident::new(
                m.builder_name.unwrap_or(format!("{}Builder", m.build_target_name))
            ),
            builder_visibility: m.builder_vis.unwrap_or(m.build_target_vis),
            builder_pattern: b.builder_pattern.unwrap_or_default(),
            build_target_ident: syn::Ident::new(m.build_target_name),
            derives: m.derive_traits.unwrap_or_default(),
            deprecation_notes: m.deprecation_notes,
            generics: m.build_target_generics,
            struct_size_hint: m.struct_size_hint,
            bindings: Bindings {
                no_std: b.no_std.unwrap_or(false),
            },
            default_expression: struct_default_expression,
        };

        (struct_options, field_defaults)
    }
}
