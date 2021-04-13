#![no_std]
#![allow(unused)]

#[macro_use]
extern crate derive_builder;

extern crate alloc;

#[derive(Builder)]
#[builder(no_std)]
struct Foo {
    bar: i32,
}

fn main() {
    let foo = FooBuilder::default().build();
    assert!(foo.is_err());
}
