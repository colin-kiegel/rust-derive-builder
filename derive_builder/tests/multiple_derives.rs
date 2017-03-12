#[macro_use]
extern crate derive_builder;

#[allow(dead_code)]
#[derive(Builder)]
struct Foo {
    lorem: bool,
}

#[allow(dead_code)]
#[derive(Builder)]
struct Bar {
    ipsum: bool,
}

#[test]
fn multiple_builder_derives() {
    // this is just a compile-test - no run time checks required.
}
