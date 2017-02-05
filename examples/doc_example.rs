#[macro_use]
extern crate derive_builder;

#[derive(Builder)]
struct Lorem {
    ipsum: String, 
    // ..
}

fn main() {}

// NOTE: generate fully expanded version with `cargo expand`.
//
//       cargo expand --example doc_example
