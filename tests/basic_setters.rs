#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    #[derive(Debug, PartialEq, Default, Builder)]
    struct Lorem {
        ipsum: String,
        dolor: String,
        sit: i32,
        amet: bool,
    }
}

impl Lorem {
    pub fn new<T: Into<String>>(value: T) -> Self {
        Lorem {
            ipsum: value.into(),
            ..Default::default()
        }
    }
}

#[test]
fn contructor_sanity_check() {
    let x = Lorem::new("lorem");

    assert_eq!(x, Lorem { ipsum: "lorem".into(), dolor: "".into(), sit: 0, amet: false, });
}

#[test]
fn setters() {
    let x = Lorem::new("lorem")
        .dolor("dolor")
        .sit(42)
        .amet(true);

    assert_eq!(x, Lorem { ipsum: "lorem".into(), dolor: "dolor".into(), sit: 42, amet: true, });
}
