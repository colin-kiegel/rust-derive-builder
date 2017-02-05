#[macro_use]
extern crate derive_builder;

#[derive(Default, Builder, Debug)]
struct Channel {
    token: i32,
    special_info: i32, 
    // .. a whole bunch of other fields ..
}

fn main() {
    // builder pattern, go, go, go!...
    let ch = ChannelBuilder::default()
        .special_info(42u8)
        .token(19124)
        .build()
        .unwrap();
    println!("{:?}", ch);
}

// NOTE: generate fully expanded version with `cargo expand`.
//
//       cargo expand --example readme_example
