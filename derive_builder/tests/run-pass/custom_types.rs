#![allow(unused_features, dead_code)]
#![feature(try_from)]

#[macro_use]
extern crate derive_builder;

type Clone = ();
type Into = ();
type Option = ();
type Result = ();
type Some = ();
type String = ();

#[derive(Builder)]
struct IgnoreEmptyStruct {}

fn main() { }
