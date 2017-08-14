#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(name = "MyBuilder")]
struct Lorem {
    ipsum: &'static str,
    pub dolor: Option<&'static str>,
    pub sit: i32,
    amet: bool,
}

#[test]
#[should_panic(expected = "`ipsum` must be initialized")]
fn panic_if_uninitialized() {
    MyBuilder::default().build().unwrap();
}

#[test]
fn builder() {
    let x: Lorem = MyBuilder::default()
        .ipsum("lorem")
        .dolor(Some("dolor"))
        .sit(42)
        .amet(true)
        .build()
        .unwrap();

    assert_eq!(
        x,
        Lorem {
            ipsum: "lorem",
            dolor: Some("dolor"),
            sit: 42,
            amet: true,
        }
    );
}
