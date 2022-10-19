// NOTE: generate fully expanded version with `cargo expand`.
//
//       cargo expand --example new_builder_function
#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
struct Channel {
    token: i32,
    special_info: i32,
}

fn main() {
    // create a new `ChannelBuilder` using `builder` fn from the `Channel` struct
    let ch = Channel::builder()
        .special_info(42u8)
        .token(19_124)
        .build()
        .unwrap();
    println!("{:#?}", ch);
}
