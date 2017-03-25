// requires nightly toolchain!
#![no_std]
#![feature(collections, lang_items, start)]
#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;
extern crate collections;

#[derive(Builder)]
#[builder(no_std)]
struct IgnoreEmptyStruct {}

///////////////////////////////////////////////////////////////
// some no_std-boilerplate
// from https://doc.rust-lang.org/book/no-stdlib.html
///////////////////////////////////////////////////////////////

// These functions and traits are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }

// Entry point for this program
#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    0
}
