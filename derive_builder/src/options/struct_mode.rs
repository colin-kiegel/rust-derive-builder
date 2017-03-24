use syn;
use options::{OptionsBuilder, OptionsBuilderMode, parse_lit_as_string, FieldMode, StructOptions};
use derive_builder_core::DeprecationNotes;

#[derive(Debug, Clone)]
pub struct StructMode {
    build_target_name: String,
    build_target_generics: syn::Generics,
    build_target_vis: syn::Visibility,
    builder_name: Option<String>,
    builder_vis: Option<syn::Visibility>,
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
            deprecation_notes: Default::default(),
            struct_size_hint: 0,
        });

        builder.parse_attributes(&ast.attrs);

        builder
    }
}

impl OptionsBuilderMode for StructMode {
    fn parse_builder_name(&mut self, name: &syn::Lit) {
        trace!("Parsing builder name `{:?}`", name);
        let value = parse_lit_as_string(name).unwrap();
        self.builder_name = Some(value.clone());
    }

    fn push_deprecation_note<T: Into<String>>(&mut self, x: T) -> &mut Self {
        self.deprecation_notes.push(x.into());
        self
    }

    /// Provide a diagnostic _where_-clause for panics.
    fn where_diagnostics(&self) -> String {
        format!("on struct `{}`", self.build_target_name)
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
            default_expression: b.default_expression,
            mode: FieldMode::default(),
        };

        let m = b.mode;

        let struct_options = StructOptions {
            builder_ident: syn::Ident::new(
                m.builder_name.unwrap_or(format!("{}Builder", m.build_target_name))
            ),
            builder_visibility: m.builder_vis.unwrap_or(m.build_target_vis),
            builder_pattern: b.builder_pattern.unwrap_or_default(),
            build_target_ident: syn::Ident::new(m.build_target_name),
            deprecation_notes: m.deprecation_notes,
            generics: m.build_target_generics,
            struct_size_hint: m.struct_size_hint,
        };

        (struct_options, field_defaults)
    }
}
