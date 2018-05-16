use derive_builder_core::BuilderPattern;

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(builder), supports(struct_named))]
pub struct StructOptions {
    pattern: BuilderPattern;

}