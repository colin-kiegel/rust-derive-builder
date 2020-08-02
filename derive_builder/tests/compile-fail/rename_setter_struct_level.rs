#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(setter(name = "foo"))]
//~^ ERROR Unknown field: `name`
struct Lorem {
    ipsum: &'static str,
    pub dolor: &'static str,
}

fn main() {
    let x = LoremBuilder::default()
    //~^ ERROR use of undeclared type or module `LoremBuilder`
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
