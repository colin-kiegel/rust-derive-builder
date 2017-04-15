#[macro_use]
extern crate derive_builder;

mod struct_default {
    #[derive(Debug, Default, PartialEq, Builder)]
    //~^ WARN  use of deprecated item: the meaning of `#[builder(default)]` on the struct level (found on struct `Hello`) will change in the next version (see https://github.com/colin-kiegel/rust-derive-builder/issues/61 for more details). To squelch this message and adopt the new behavior now, compile `derive_builder` with `--features "struct_default"`.
    //~| NOTE in this expansion of #[derive(Builder)]
    //~| NOTE #[warn(deprecated)] on by default
    //~| NOTE in this expansion of #[derive(Builder)]

    #[builder(default)]
    struct Hello {
        world: String
    }

    #[test]
    fn defaults() {
        assert_eq!(Ok(Hello::default()), HelloBuilder::default().build());
    }
}

mod field_default {

    #[derive(Debug, Default, PartialEq, Builder)]
    struct Goodbye {
        #[builder(default)]
        world: String
    }

    #[test]
    fn pass() {}
}

fn main() {}
