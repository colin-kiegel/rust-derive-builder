#[macro_use]
extern crate derive_builder;

// deny `#[builder(default = "")]`, because we don't want to define a meaning (yet)! :-)
#[allow(dead_code)]
#[derive(Builder)]
struct Lorem {
    #[builder(default = "")]
    //~^ ERROR Unknown literal value ``
    ipsum: String,
}

fn main() {}
