#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[setters(mutable)]
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
fn mutable_setter() {
    // the setter must have the correct signature
    let mutable_setter: fn(&mut Lorem, String) -> &mut Lorem = Lorem::ipsum;

    let mut old = Lorem::new("lorem");
    let new = mutable_setter(&mut old, "new".to_string());

    assert_eq!(*new, Lorem { ipsum: "new".into() });
}
