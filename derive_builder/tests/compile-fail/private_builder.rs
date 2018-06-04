#[macro_use]
extern crate derive_builder;

pub mod foo {
    /// The builder struct's declaration of privacy should override the field's
    /// attempt to be public later on.
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[builder(private, setter(into))]
    pub struct Lorem {
        pub private: String,
        #[builder(public)]
        pub public: String,
    }
}

fn main() {
    let x = foo::LoremBuilder::default()
    //~^ ERROR struct `LoremBuilder` is private
        .public("Hello")
        .build()
    //~^ ERROR method `build` is private
        .unwrap();

    assert_eq!(x.public, "Hello".to_string());
}
