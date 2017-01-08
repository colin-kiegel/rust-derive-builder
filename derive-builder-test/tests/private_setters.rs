#[macro_use] extern crate derive_builder;

pub mod foo {
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[setters(private)]
    pub struct Lorem {
        pub ipsum: String,
    }

    #[test]
    fn setters_same_module() {
        let x = Lorem::default()
            .ipsum("Hello world!")
            .clone();

        assert_eq!(x, Lorem { ipsum: "Hello world!".into() });
    }
}

// compile-test should fail with "error: method `ipsum` is private"
// fn setters_foreign_module() {
//     let x = foo::Lorem::default()
//         .ipsum("Hello world!")
//         .clone();
//
//     assert_eq!(x, foo::Lorem { ipsum: "Hello world!".into() });
// }
