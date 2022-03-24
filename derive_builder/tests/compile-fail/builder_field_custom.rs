#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
pub struct Lorem {
    #[builder(default = "88", custom(type = "usize", build = "self.ipsum.unwrap_or_else(42) + 1"))]
    ipsum: usize,
}

fn main() {}
