use syn;
use options::{OptionsBuilder, OptionsBuilderMode, parse_lit_as_string, parse_lit_as_bool,
              parse_lit_as_path, FieldMode, StructOptions};
use derive_builder_core::{DeprecationNotes, Bindings};

#[derive(Debug, Clone)]
pub struct StructMode {
    build_fn_name: Option<String>,
    build_fn_enabled: Option<bool>,
    build_target_name: String,
    build_target_generics: syn::Generics,
    build_target_vis: syn::Visibility,
    builder_name: Option<String>,
    builder_vis: Option<syn::Visibility>,
    derive_traits: Option<Vec<syn::Ident>>,
    deprecation_notes: DeprecationNotes,
    validate_fn: Option<syn::Path>,
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
            build_fn_enabled: None,
            build_fn_name: None,
            derive_traits: None,
            deprecation_notes: Default::default(),
            validate_fn: None,
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
        ident: build_fn_name,
        desc: "build function name",
        map: |x: String| { x },
    }

    impl_setter!{
        ident: build_fn_enabled,
        desc: "build function enabled",
        map: |x: bool| { x },
    }

    impl_setter!{
        ident: validate_fn,
        desc: "validator function path",
        map: |x: syn::Path| { x },
    }

    impl_setter!{
        ident: derive_traits,
        desc: "derive traits",
        map: |x: Vec<syn::Ident>| { x },
    }

    #[allow(non_snake_case)]
    fn parse_build_fn_options_metaItem(&mut self, meta_item: &syn::MetaItem) {
        trace!("Build Method Options - Parsing MetaItem `{:?}`.", meta_item);
        match *meta_item {
            syn::MetaItem::Word(ref ident) => {
                self.parse_build_fn_options_word(ident)
            },
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                self.parse_build_fn_options_nameValue(ident, lit)
            },
            syn::MetaItem::List(ref ident, ref nested_attrs) => {
                self.parse_build_fn_options_list(ident, nested_attrs)
            }
        }
    }

    #[allow(non_snake_case)]
    fn parse_build_fn_options_nameValue(&mut self, ident: &syn::Ident, lit: &syn::Lit) {
        trace!("Build fn Options - Parsing named value `{}` = `{:?}`", ident.as_ref(), lit);
        match ident.as_ref() {
            "name" => {
                self.parse_build_fn_name(lit)
            },
            "skip" => {
                self.parse_build_fn_skip(lit)
            },
            "validate" => {
                self.parse_build_fn_validate(lit)
            },
            _ => {
                panic!("Unknown build_fn option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    /// e.g `private` in `#[builder(build_fn(private))]`
    fn parse_build_fn_options_word(&mut self, ident: &syn::Ident) {
        trace!("Setter Options - Parsing word `{}`", ident.as_ref());
        match ident.as_ref() {
            "skip" => {
                self.build_fn_enabled(false);
            }
            _ => {
                panic!("Unknown build_fn option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        };
    }

    #[allow(non_snake_case)]
    fn parse_build_fn_options_list(&mut self, ident: &syn::Ident, nested: &[syn::NestedMetaItem]) {
        trace!("Build fn Options - Parsing list `{}({:?})`", ident.as_ref(), nested);
        match ident.as_ref() {
            _ => {
                panic!("Unknown option `{}` {}.", ident.as_ref(), self.where_diagnostics())
            }
        }
    }

    fn parse_build_fn_name(&mut self, lit: &syn::Lit) {
        trace!("Parsing build function name `{:?}`", lit);
        let value = parse_lit_as_string(lit).unwrap();
        self.build_fn_name(value.clone())
    }

    #[allow(dead_code,unused_variables)]
    fn parse_build_fn_skip(&mut self, skip: &syn::Lit) {
        self.build_fn_enabled(!parse_lit_as_bool(skip).unwrap());
    }

    fn parse_build_fn_validate(&mut self, lit: &syn::Lit) {
        trace!("Parsing build function validate path `{:?}`", lit);
        let value = parse_lit_as_path(lit).unwrap();
        self.validate_fn(value);
    }
}

impl OptionsBuilderMode for StructMode {
    fn parse_builder_name(&mut self, name: &syn::Lit) {
        trace!("Parsing builder name `{:?}`", name);
        let value = parse_lit_as_string(name).unwrap();
        self.builder_name(value.clone());
    }

    fn parse_build_fn_options(&mut self, nested: &[syn::NestedMetaItem]) {
        for x in nested {
            match *x {
                syn::NestedMetaItem::MetaItem(ref meta_item) => {
                    self.parse_build_fn_options_metaItem(meta_item);
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

    /// Parse the `derive` list for struct-level builder declarations.
    fn parse_derive(&mut self, nested: &[syn::NestedMetaItem]) {
        let mut traits = vec![];
        let where_diag = self.where_diagnostics();
        for x in nested {
            match *x {
        // We don't allow name-value pairs or further nesting here, so
        // only look for words.
                syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref tr)) => {
                    match tr.as_ref() {
                        "Default" | "Clone" => { self.push_deprecation_note(
                            format!("The `Default` and `Clone` traits are automatically added to all \
                            builders; explicitly deriving them is unnecessary ({})", where_diag));
                        },
                        _ => traits.push(tr.clone())
                    }
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
        let field_defaults = OptionsBuilder::<FieldMode> {
            setter_enabled: b.setter_enabled,
            builder_pattern: b.builder_pattern,
            setter_name: None,
            setter_prefix: b.setter_prefix,
            setter_vis: b.setter_vis,
            setter_into: b.setter_into,
            try_setter: b.try_setter,
            field_vis: b.field_vis,
            default_expression: None,
            no_std: b.no_std,
            mode: {
                let mut mode = FieldMode::default();
                mode.use_default_struct = b.default_expression.is_some();
                mode
            },
        };

        let m = b.mode;

        let pattern = b.builder_pattern.unwrap_or_default();
        let bindings = Bindings {
            no_std: b.no_std.unwrap_or(false)
        };

        let struct_options = StructOptions {
            build_fn_enabled: m.build_fn_enabled.unwrap_or(true),
            build_fn_name: syn::Ident::new(
                m.build_fn_name.unwrap_or("build".to_string())
            ),
            builder_ident: syn::Ident::new(
                m.builder_name.unwrap_or(format!("{}Builder", m.build_target_name))
            ),
            builder_visibility: m.builder_vis.unwrap_or(m.build_target_vis),
            builder_pattern: pattern,
            build_target_ident: syn::Ident::new(m.build_target_name),
            derives: m.derive_traits.unwrap_or_default(),
            deprecation_notes: m.deprecation_notes,
            generics: m.build_target_generics,
            struct_size_hint: m.struct_size_hint,
            bindings: bindings,
            default_expression: b.default_expression,
            validate_fn: m.validate_fn,
        };

        (struct_options, field_defaults)
    }
}
