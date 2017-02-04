#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(prefix="with")]
struct Lorem {
    ipsum: String,
    #[builder(prefix="set")]
    pub dolor: Option<String>,
}

#[test]
fn prefixed_setters() {
    let x = Lorem::default()
        .with_ipsum("ipsum")
        .set_dolor(Some("dolor".into()))
        .clone();

    assert_eq!(x, Lorem { ipsum: "ipsum".into(), dolor: Some("dolor".into())});
}