#[macro_use]
extern crate derive_builder;

/// The `LoremBuilder` struct will have private fields for `ipsum` and `dolor`, and
/// a public `sit` field.
#[derive(Debug, Builder)]
#[builder(field(private), setter(into))]
pub struct Lorem {
    ipsum: String,
    dolor: u16,
    #[builder(field(public))]
    sit: bool,
}

fn main() {
    
}