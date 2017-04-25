#[macro_use]
extern crate derive_builder;

/// A struct that deliberately doesn't implement `Clone`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Dolor(String);

/// Notice that this type derives Builder without disallowing
/// `Lorem<Dolor>`.
#[derive(Debug, Clone, Builder, PartialEq, Eq)]
#[builder(field(private), setter(into))]
pub struct Lorem<T> {
    ipsum: T,
}

fn make_default_lorem<T: Default>() -> Lorem<T> {
    Lorem {
        ipsum: Default::default()
    }
}

fn make_u16_lorem(v: u16) -> Lorem<u16> {
    Lorem {
        ipsum: v
    }
}

fn make_dolor_lorem(v: Dolor) -> Lorem<Dolor> {
    Lorem {
        ipsum: v
    }
}

fn build_u16_lorem(v: u16) -> Lorem<u16> {
    LoremBuilder::default()
        .ipsum(v)
        .build()
        .unwrap()
}

#[test]
fn u16_lorems() {
    assert_eq!(make_u16_lorem(10), build_u16_lorem(10));
}

#[test]
fn dolor_lorems() {
    assert_eq!(make_default_lorem(), make_dolor_lorem(Dolor::default()));
}

#[test]
fn type_inference() {
    assert_eq!(LoremBuilder::<String>::default().ipsum("".to_string()).build().unwrap(), 
               make_default_lorem());
}