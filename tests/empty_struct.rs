#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    /// This is a doc comment for the struct
    #[warn(missing_docs)]
    #[allow(non_snake_case)]
    #[derive(Debug, PartialEq, Default, Builder)]
    struct IgnoreEmptyStruct {  }
}

#[test]
fn empty_struct() {
    // this is just a compile-test - no run time checks required.
}
