#![allow(unused_features)]
#![feature(try_from)]

#[macro_use]
extern crate derive_builder;

#[allow(dead_code)]
#[derive(Builder)]
struct IgnoreEmptyStruct {}

fn main() { }
