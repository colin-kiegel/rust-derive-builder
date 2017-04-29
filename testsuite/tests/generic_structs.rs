#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

use std::fmt::Display;
use std::clone::Clone;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Generic<T: Display>
    where T: Clone
{
    ipsum: &'static str,
    pub dolor: T,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
pub struct GenericReference<'a, T: 'a + Default>
    where T: Display
{
    pub bar: Option<&'a T>,
}

#[test]
#[should_panic(expected="`ipsum` must be initialized")]
fn panic_if_uninitialized() {
    GenericBuilder::<String>::default().build().unwrap();
}

#[test]
fn generic_builder() {
    let x = GenericBuilder::default()
        .ipsum("Generic")
        .dolor(true)
        .build()
        .unwrap();

    assert_eq!(x,
               Generic {
                   ipsum: "Generic".into(),
                   dolor: true,
               });
}

#[test]
fn generic_reference_builder() {
    static BAR: u32 = 42;

    let x = GenericReferenceBuilder::<'static, u32>::default()
        .bar(Some(&BAR))
        .build()
        .unwrap();

    assert_eq!(x, GenericReference { bar: Some(&BAR) });
}
