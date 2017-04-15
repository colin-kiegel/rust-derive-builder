use syn;

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
pub fn doc_comment_from(s: String) -> syn::Attribute {
    syn::Attribute {
        style: syn::AttrStyle::Outer,
        value: syn::MetaItem::NameValue(syn::Ident::new("doc"),
                                        syn::Lit::Str(s, syn::StrStyle::Raw(2))),
        is_sugared_doc: false,
    }
}
