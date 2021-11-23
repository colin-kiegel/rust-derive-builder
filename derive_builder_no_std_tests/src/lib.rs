#![no_std]
#![allow(unused, clippy::blacklisted_name)]

extern crate alloc;
#[macro_use]
extern crate derive_builder;

use alloc::string::String;
use alloc::string::ToString;

#[derive(Builder)]
#[builder(no_std)]
pub struct Foo {
    pub bar: i32,
}

pub fn build_foo_ok() -> Foo {
    FooBuilder::default().bar(42).build().unwrap()
}

pub fn build_foo_err() -> Option<String> {
    let foo = FooBuilder::default().build();
    foo.err().map(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_ok() {
        assert_eq!(build_foo_ok().bar, 42);
    }

    #[test]
    fn test_builder_err() {
        assert_eq!(
            build_foo_err().as_deref(),
            Some("`bar` must be initialized")
        );
    }
}
