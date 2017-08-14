#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

pub mod foo {
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[builder(private, setter(into))]
    pub struct Lorem {
        pub private: String,
        #[builder(public)]
        pub public: String,
    }

    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[builder(public, setter(into))]
    pub struct Ipsum {
        #[builder(private)]
        pub private: String,
        pub public: String,
    }

    #[test]
    fn setters_same_module() {
        let x = LoremBuilder::default()
            .public("Hello")
            .private("world!")
            .build()
            .unwrap();

        assert_eq!(
            x,
            Lorem {
                public: "Hello".into(),
                private: "world!".into(),
            }
        );

        let y = IpsumBuilder::default()
            .public("Hello")
            .private("world!")
            .build()
            .unwrap();

        assert_eq!(
            y,
            Ipsum {
                public: "Hello".into(),
                private: "world!".into(),
            }
        );
    }
}

#[test]
#[should_panic(expected = "`private` must be initialized")]
fn public_setters_override_foreign_module() {
    let x = foo::LoremBuilder::default()
        .public("Hello")
        .build()
        .unwrap();

    assert_eq!(x.public, "Hello".to_string());
}

#[test]
#[should_panic(expected = "`private` must be initialized")]
fn public_setters_foreign_module() {
    let y = foo::IpsumBuilder::default()
        .public("Hello")
        .build()
        .unwrap();

    assert_eq!(y.public, "Hello".to_string());
}

// compile-test should fail with "error: method `ipsum` is private"
// fn setters_foreign_module() {
//     let x = foo::Lorem::default()
//         .ipsum("Hello world!")
//         .clone();
//
//     assert_eq!(x, foo::Lorem { ipsum: "Hello world!".into() });
// }
