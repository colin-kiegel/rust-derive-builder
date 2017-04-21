#[macro_use]
extern crate derive_builder;

// this is meant to generate a deprecation warning! :-)
#[allow(dead_code)]
#[derive(Builder)]
//~^ WARN  use of deprecated item: warning: deprecated syntax `#[builder(setter_prefix="old_syntax")]`, please use `#[builder(setter(prefix="old_syntax"))]` instead on field `ipsum`.
//~| NOTE in this expansion of #[derive(Builder)]
//~| NOTE #[warn(deprecated)] on by default
//~| NOTE in this expansion of #[derive(Builder)]
#[builder(field(private))]
struct Lorem {
    #[builder(setter_prefix="old_syntax")]
    ipsum: String,
}

fn main() {}
