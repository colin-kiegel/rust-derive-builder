#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
pub struct Lorem {
    ok: String,

    // This foolish repr() attribute generates an unused attribute warning
    #[builder_field_attrs(
        #[no_such_attribute]
    )]
    broken: String,
}

fn main() {}
