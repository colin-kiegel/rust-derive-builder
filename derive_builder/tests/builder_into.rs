#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(setter(into, strip_option))]
struct Lorem {
    pub ipsum: String,
    pub dolor: Option<String>,
    #[builder(setter(skip), default = "4")]
    pub sit: u32,
}

#[test]
fn builder_into() {
    let lorem: Lorem = LoremBuilder::default()
        .ipsum("Foo")
        .dolor("Bar")
        .build()
        .unwrap();

    let mut builder: LoremBuilder = lorem.into();

    let lorem: Lorem = builder.ipsum("Baz").build().unwrap();

    assert_eq!(
        lorem,
        Lorem {
            ipsum: "Baz".to_owned(),
            dolor: Some("Bar".to_owned()),
            sit: 4
        }
    );
}
