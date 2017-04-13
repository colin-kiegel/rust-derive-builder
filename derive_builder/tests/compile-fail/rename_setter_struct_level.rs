#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
//~^ ERROR proc-macro derive panicked
#[builder(setter(name="foo"))]
struct Lorem {
    ipsum: &'static str,
    pub dolor: &'static str,
}

#[test]
fn renamed_setter() {
    let x = LoremBuilder::default()
        .ipsum("ipsum")
        .foo("dolor")
        .build()
        .unwrap();

    assert_eq!(x,
               Lorem {
                   ipsum: "ipsum",
                   dolor: "dolor",
               });
}
