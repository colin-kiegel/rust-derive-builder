#![allow(dead_code)]

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
