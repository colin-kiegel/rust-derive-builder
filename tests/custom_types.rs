#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;

type Clone = ();
type Into = ();
type Option = ();
type Result = ();

#[derive(Builder)]
struct IgnoreEmptyStruct {}

#[test]
fn empty_struct() {
    // this is just a compile-test - no run time checks required.
}
