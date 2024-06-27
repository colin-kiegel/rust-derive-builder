#[macro_use]
extern crate derive_builder;

#[derive(Debug, Builder, PartialEq, Clone)]
#[builder(suppress_derive_clone)]
struct Lorem {
    #[builder(setter(into))]
    ipsum: String,
}

impl Clone for LoremBuilder {
    fn clone(&self) -> Self {
        Self {
            ipsum: self.ipsum.clone(),
        }
    }
}

#[test]
fn error_if_uninitialized() {
    let error = LoremBuilder::default().build().unwrap_err();
    assert_eq!(&error.to_string(), "`ipsum` must be initialized");
}

#[test]
fn builder_test() {
    let x = LoremBuilder::default().ipsum("ipsum").build().unwrap();

    assert_eq!(
        x,
        Lorem {
            ipsum: "ipsum".into()
        }
    );
}

#[test]
fn builder_is_clone() {
    let _ = LoremBuilder::default().clone();
}
