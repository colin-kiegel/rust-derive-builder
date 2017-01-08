#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[immutable]
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
fn immutable() {
    let f: fn(&Lorem, String) -> Lorem = Lorem::ipsum;

    let old = Lorem::new("lorem");
    let new = f(&old, "new".to_string());

    assert_eq!(new, Lorem { ipsum: "new".into() });
}
