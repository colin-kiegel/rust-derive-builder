#![cfg_attr(feature = "try_from", feature(try_from))]

// NOTE: generate fully expanded version with `cargo expand`.
//
//       cargo expand --example doc_example

#[macro_use]
extern crate derive_builder;

#[allow(dead_code)]
#[derive(Builder)]
struct Lorem {
    ipsum: u32,
}

fn main() {}
