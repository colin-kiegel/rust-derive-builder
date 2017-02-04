#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(immutable)]
struct Lorem {
    ipsum: String,
}

impl Lorem {
    pub fn new<T: Into<String>>(value: T) -> Self {
        Lorem {
            ipsum: value.into()
        }
    }
}

#[test]
fn contructor_sanity_check() {
    let x = Lorem::new("lorem");

    assert_eq!(x, Lorem { ipsum: "lorem".into() });
}

#[test]
fn immutable_setter() {
    // the setter must have the correct signature
    let immutable_setter: fn(&Lorem, String) -> Lorem = Lorem::ipsum;

    let old = Lorem::new("lorem");
    let new = immutable_setter(&old, "new".to_string());

    assert_eq!(new, Lorem { ipsum: "new".into() });
}
