#![deny(unused_must_use)]

#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
pub struct Lorem {
    ok: String,

    // This foolish repr() attribute generates an unused attribute warning
    #[builder_setter_attrs(must_use)]
    broken: usize,
}

fn main() {
    LoremBuilder::default().broken(42);
}
