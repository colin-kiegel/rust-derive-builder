#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(setter_prefix="with")]
struct Lorem {
    ipsum: String,
    #[builder(setter_prefix="set")]
    pub dolor: Option<String>,
}

#[test]
fn prefixed_setters() {
    let x = LoremBuilder::default()
        .with_ipsum("ipsum")
        .set_dolor(Some("dolor".into()))
        .build()
        .unwrap();

    assert_eq!(x,
               Lorem {
                   ipsum: "ipsum".into(),
                   dolor: Some("dolor".into()),
               });
}
