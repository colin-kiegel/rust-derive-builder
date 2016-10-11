#![feature(proc_macro)]
#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Lorem {
    ipsum: String,
    pub dolor: Option<String>,
    pub sit: i32,
    amet: bool,
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

    assert_eq!(x, Lorem { ipsum: "lorem".into(), dolor: None, sit: 0, amet: false, });
}

#[test]
fn setters() {
    let x = Lorem::new("lorem")
        .dolor(Some("dolor".into()))
        .sit(42)
        .amet(true)
        .clone();

    assert_eq!(x, Lorem { ipsum: "lorem".into(), dolor: Some("dolor".into()), sit: 42, amet: true, });
}
