// requires nightly toolchain!
//
// compile-flags:-C panic=abort
#![no_std]
#![feature(collections, lang_items, start, core_intrinsics)]
#![allow(dead_code)]
use core::intrinsics;

// Pull in the system libc library for what crt0.o likely requires.
// extern crate libc;

#[macro_use]
extern crate derive_builder;
extern crate collections;

#[derive(Builder)]
#[builder(no_std)]
struct IgnoreEmptyStruct {}

#[derive(Builder, PartialEq, Debug)]
#[builder(no_std)]
struct Foo {
    #[builder(default)]
    defaulted: u32,
    #[builder(setter(skip), try_setter)]
    skipped: u32,
}

fn main() {
    let foo = FooBuilder::default()
        .build()
        .unwrap();

    assert_eq!(foo, Foo {
        defaulted: 0,
        skipped: 0,
    })
}

///////////////////////////////////////////////////////////////
// some no_std-boilerplate
// from https://doc.rust-lang.org/book/no-stdlib.html
///////////////////////////////////////////////////////////////

// These functions and traits are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"]
#[no_mangle]
pub extern  fn eh_personality() {}

// This function may be needed based on the compilation target.
#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern fn rust_eh_unwind_resume() {
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    unsafe { intrinsics::abort() }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    unsafe { intrinsics::abort() }
}

// Entry point for this program
#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    main();
    0
}
