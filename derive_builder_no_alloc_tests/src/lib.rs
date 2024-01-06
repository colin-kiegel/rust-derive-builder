#![no_std]
#![allow(unused, clippy::blacklisted_name)]

use derive_builder::{self, Builder};

#[derive(Debug, Eq, PartialEq)]
pub struct FooError(&'static str);

impl From<derive_builder::UninitializedFieldError> for FooError {
    fn from(err: derive_builder::UninitializedFieldError) -> Self {
        Self(err.field_name())
    }
}

#[derive(Builder)]
#[builder(no_std, build_fn(error = "FooError"))]
pub struct Foo {
    pub bar: i32,
}

#[derive(Builder)]
#[builder(no_std, build_fn(error(validation_error = false)))]
pub struct Fee {
    pub bar: i32,
}

pub fn build_foo_ok() -> Foo {
    FooBuilder::default().bar(42).build().unwrap()
}

pub fn build_foo_err() -> Option<FooError> {
    let foo = FooBuilder::default().build();
    foo.err()
}

pub fn build_fee_ok() -> Foo {
    FooBuilder::default().bar(42).build().unwrap()
}

pub fn build_fee_err() -> Option<FeeBuilderError> {
    let fee = FeeBuilder::default().build();
    fee.err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo_builder_ok() {
        assert_eq!(build_foo_ok().bar, 42);
    }

    #[test]
    fn test_foo_builder_err() {
        assert_eq!(build_foo_err(), Some(FooError("bar")));
    }

    #[test]
    fn test_fee_builder_ok() {
        assert_eq!(build_fee_ok().bar, 42);
    }

    #[test]
    fn test_fee_builder_err() {
        assert!(matches!(
            build_fee_err(),
            Some(FeeBuilderError::UninitializedField("bar"))
        ));
    }
}
