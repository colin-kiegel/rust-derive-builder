use syn::Attribute;
use syn::synom::Parser;

/// Doc-comment, implementing `quote::ToTokens`.
///
/// # Examples
///
/// Will expand to something like the following (depending on inner value):
///
/// ```rust
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate syn;
/// # extern crate derive_builder_core;
/// # use derive_builder_core::doc_comment_from;
/// # fn main() {
/// #    let doc_comment = doc_comment_from("foo".to_string());
/// #
/// #    assert_eq!(quote!(#doc_comment), quote!(
/// #[doc = r##"foo"##]
/// #    ));
/// # }
/// ```
pub fn doc_comment_from(s: String) -> Attribute {
    named!(doc_attr -> Attribute, call!(Attribute::parse_outer));
    doc_attr.parse_str(&quote!(#[doc=#s]).to_string()).unwrap()
}
