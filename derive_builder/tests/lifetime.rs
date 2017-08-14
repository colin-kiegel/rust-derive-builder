#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Lorem<'a> {
    ipsum: &'a str,
}

#[test]
#[should_panic(expected = "`ipsum` must be initialized")]
fn panic_if_uninitialized() {
    LoremBuilder::default().build().unwrap();
}

#[test]
fn builder() {
    let x = LoremBuilder::default().ipsum("ipsum").build().unwrap();

    assert_eq!(x, Lorem { ipsum: "ipsum" });
}
