#[macro_use]
extern crate derive_builder;

#[derive(Builder)]
#[builder(suppress_derive_clone)]
pub struct Example {
    field: String,
}

fn main() {
    let _ = ExampleBuilder::default().clone();
}
