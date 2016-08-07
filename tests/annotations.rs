#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    /// This is a doc comment for the struct
    #[warn(missing_docs)]
    #[derive(Debug, PartialEq, Default, Builder)]
    struct Lorem {
        /// This is a doc comment for a field
        field_with_doc_comment: String,
        #[allow(missing_docs)]
        undocumented: String,
    }
}

// this is just a compile-test - no run time checks required.
