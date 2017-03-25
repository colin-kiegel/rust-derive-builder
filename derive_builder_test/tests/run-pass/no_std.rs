// requires nightly toolchain!
#![no_std]
#![feature(collections)]
#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;
extern crate collections;

#[derive(Builder)]
#[builder(no_std)]
struct IgnoreEmptyStruct {}
