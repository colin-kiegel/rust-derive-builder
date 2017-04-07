#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct GenLorem<T>
    where T: std::clone::Clone
{
    ipsum: &'static str,
    pub dolor: T,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct GenLorem2<T>
    where T: std::clone::Clone
{
    ipsum: &'static str,
    pub dolor: T,
}

#[test]
#[should_panic(expected="`ipsum` must be initialized")]
fn panic_if_uninitialized() {
    GenLoremBuilder::<String>::default().build().unwrap();
}

#[test]
fn builder() {
    let x = GenLoremBuilder::default()
        .ipsum("GenLorem")
        .dolor(true)
        .build()
        .unwrap();

    assert_eq!(x,
               GenLorem {
                   ipsum: "GenLorem".into(),
                   dolor: true,
               });
}
