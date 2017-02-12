#[macro_use]
extern crate derive_builder;

#[allow(dead_code)]
#[derive(Builder)]
struct IgnoreEmptyStruct {}

#[test]
fn empty_struct() {
    // this is just a compile-test - no run time checks required.
}
