#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Lorem<'a> {
    ipsum: &'a str,
}

impl<'a> Lorem<'a> {
    pub fn new<T: Into<&'a str>>(value: T) -> Self {
        Lorem {
            ipsum: value.into()
        }
    }
}

#[test]
fn contructor_sanity_check() {
    let x = Lorem::new("ipsum");

    assert_eq!(x, Lorem { ipsum: "ipsum" });
}

#[test]
fn immutable_setter() {
    let x = Lorem::new("")
        .ipsum("ipsum")
        .clone();

    assert_eq!(x, Lorem { ipsum: "ipsum" });
}
