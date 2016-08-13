#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    /// This is a doc comment for the struct
    #[warn(missing_docs)]
    #[allow(non_snake_case)]
    #[derive(Debug, PartialEq, Default, Builder)]
    struct Lorem {
        /// This is a doc comment for a field
        field_with_doc_comment: String,
        #[allow(missing_docs)]
        undocumented: String,
        #[allow(non_snake_case)]
        CamelCase: i32,
        #[cfg(target_os = "macos")] // TODO: this fails on linux builds
        mac_only: (),
        #[cfg(target_os = "linux")] // TODO: this fails on mac builds
        linux_only: (),
    }
}

#[test]
fn annotations() {
    // this is currently just a compile-test (may switch to token comparisons here)
    // https://github.com/colin-kiegel/rust-derive-builder/issues/19
}
