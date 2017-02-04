#[macro_use] extern crate derive_builder;

pub mod foo {
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[builder(private)]
    pub struct Lorem {
        pub private: String,
        #[builder(public)]
        pub public: String,
    }

    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[builder(public)]
    pub struct Ipsum {
        #[builder(private)]
        pub private: String,
        pub public: String,
    }

    #[test]
    fn setters_same_module() {
        let x = Lorem::default()
            .public("Hello")
            .private("world!")
            .clone();

        assert_eq!(x, Lorem { public: "Hello".into(), private: "world!".into() });

        let y = Ipsum::default()
            .public("Hello")
            .private("world!")
            .clone();

        assert_eq!(y, Ipsum { public: "Hello".into(), private: "world!".into() });
    }
}

#[test]
fn public_setters_foreign_module() {
    let x = foo::Lorem::default()
        .public("Hello")
        .clone();

    assert_eq!(x.public, String::from("Hello") );

    let y = foo::Ipsum::default()
        .public("Hello")
        .clone();

    assert_eq!(y.public, String::from("Hello") );
}

// compile-test should fail with "error: method `ipsum` is private"
// fn setters_foreign_module() {
//     let x = foo::Lorem::default()
//         .ipsum("Hello world!")
//         .clone();
//
//     assert_eq!(x, foo::Lorem { ipsum: "Hello world!".into() });
// }
