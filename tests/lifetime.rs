#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    #[derive(Debug, PartialEq, Builder)]
    struct Lorem<'a, T> {
        ipsum: &'a str,
        dolor: Option<T>,
    }
}

impl<'a, T> Lorem<'a, T> {
    pub fn new(ipsum: &'a str) -> Self {
        Lorem {
            ipsum: ipsum,
            dolor: None,
        }
    }
}

#[test]
fn contructor_sanity_check() {
    let ipsum: String = "Ipsum with references to it".into();
    let x: Lorem<()> = Lorem::new(&ipsum);

    assert_eq!(x, Lorem { ipsum: &ipsum, dolor: None, });
}

#[test]
fn setters() {
    let ipsum: String = "Ipsum with references to it".into();
    let dolor = true;
    let x: Lorem<&bool> = Lorem::new(&ipsum)
        .dolor(Some(&dolor));

    assert_eq!(x, Lorem { ipsum: &ipsum, dolor: Some(&dolor), });
}
