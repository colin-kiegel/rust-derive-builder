#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

use std::convert::From;

#[derive(PartialEq, Default, Debug, Clone)]
struct Uuid(i32);
#[derive(PartialEq, Default, Debug, Clone)]
struct Authentication(i32);

impl From<i32> for Uuid {
    fn from(x: i32) -> Uuid {
        Uuid(x)
    }
}

impl From<i32> for Authentication {
    fn from(x: i32) -> Authentication {
        Authentication(x)
    }
}

custom_derive!{
    #[derive(Debug, Default, Builder)]
    struct Channel {
        id: Uuid,
        token: Authentication,
        special_info: i32
    }
}

fn main() {
    let ch = Channel::default().special_info(42);
    println!("{:?}", ch);
}
