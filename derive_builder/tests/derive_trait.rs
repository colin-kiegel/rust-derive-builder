#[macro_use]
extern crate derive_builder;

#[derive(Debug, Default, Clone)]
struct NotPartialEq(String);

#[derive(Debug, Clone, Builder)]
#[builder(derive(Debug, PartialEq, Eq))]
struct Lorem { 
    foo: u8,
    
    /// This type doesn't have `PartialEq` support, but that's fine
    /// since we don't want it in the builder.
    #[builder(setter(skip))]
    excluded: NotPartialEq,
}

#[test]
fn defaults() {
    assert_eq!(LoremBuilder::default(), LoremBuilder::default());
}