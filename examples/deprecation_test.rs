#[macro_use]
extern crate derive_builder;

// this is meant to generate a deprecation warning! :-)
#[allow(dead_code)]
#[derive(Builder)]
struct Lorem {
    #[builder(setter_prefix="old_syntax")]
    ipsum: String,
}

fn main() {}
