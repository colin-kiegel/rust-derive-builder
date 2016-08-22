#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    struct GenLorem<T> {
        ipsum: String,
        pub dolor: T, // generics are a pain, so this field name is fitting
    }
}

impl<T: Default> GenLorem<T> {
    pub fn new<V: Into<String>>(value: V) -> Self {
        GenLorem {
            ipsum: value.into(),
            ..Default::default()
        }
    }
}

#[test]
fn contructor_sanity_check() {
    let x: GenLorem<bool> = GenLorem::new("GenLorem");

    assert_eq!(x, GenLorem { ipsum: "GenLorem".into(), dolor: false, });
}

#[test]
fn setters() {
    let x = GenLorem::new("GenLorem")
        .dolor(true)
        .clone();

    assert_eq!(x, GenLorem { ipsum: "GenLorem".into(), dolor: true, });
}
