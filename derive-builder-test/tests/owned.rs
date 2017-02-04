#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(pattern="owned")]
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
fn consuming_setter() {
    // the setter must have the correct signature
    let consuming_setter: fn(Lorem, String) -> Lorem = Lorem::ipsum;

    let old = Lorem::new("lorem");
    let new = consuming_setter(old, "new".to_string());

    assert_eq!(new, Lorem { ipsum: "new".into() });
}
