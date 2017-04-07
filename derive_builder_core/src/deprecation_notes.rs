use quote::{Tokens, ToTokens};
use syn;

/// Deprecation notes we want to emit to the user, implementing `quote::ToTokens`.
///
/// Can be expanded at every place that accepts item definitions (e.g. function bodys).
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate derive_builder_core;
/// # use derive_builder_core::DeprecationNotes;
/// # fn main() {
/// #    let mut note = DeprecationNotes::default();
/// #    note.push(String::from("Some Warning"));
/// #    assert_eq!(quote!(#note), quote!(
///         {
///             #[deprecated(note="Some Warning")]
///             fn derive_builder_core_deprecation_notice() { }
///             derive_builder_core_deprecation_notice();
///         }
/// #    ));
/// # }
/// ```
///
/// This will emit a deprecation warning in the downstream crate. Cool stuff. ^^
///
/// Proof of concept:
/// - https://play.rust-lang.org/?gist=8394141c07d1f6d75d314818389eb4d8
#[derive(Debug, Default, Clone)]
pub struct DeprecationNotes(Vec<String>);

impl ToTokens for DeprecationNotes {
    fn to_tokens(&self, tokens: &mut Tokens) {
        for note in &self.0 {
            let fn_ident = syn::Ident::new("derive_builder_core_deprecation_notice");
            tokens.append(quote!(
                {
                    #[deprecated(note=#note)]
                    fn #fn_ident() { }
                    #fn_ident();
                }
            ));
        }
    }
}

impl DeprecationNotes {
    /// Appends a note to the collection.
    pub fn push(&mut self, note: String) {
        self.0.push(note)
    }

    /// Extend this collection with all values from another collection.
    pub fn extend(&mut self, other: &DeprecationNotes) {
        for x in &other.0 {
            self.0.push(x.to_owned())
        }
    }
    
    /// Converts this deprecation note set into one that can annotate a struct.
    pub fn for_struct(self) -> StructDeprecationNotes {
        StructDeprecationNotes(self.0)
    }
}

#[derive(Debug, Default)]
pub struct StructDeprecationNotes(Vec<String>);

impl ToTokens for StructDeprecationNotes {
    fn to_tokens(&self, tokens: &mut Tokens) {
        for note in &self.0 {
            tokens.append(quote!(
                #[deprecated(note=#note)]
            ));
        }
    }
}

#[test]
fn deprecation_note() {
    let mut note = DeprecationNotes::default();
    note.push(String::from("Some Warning"));
    assert_eq!(quote!(#note), quote!(
        {
            #[deprecated(note="Some Warning")]
            fn derive_builder_core_deprecation_notice() { }
            derive_builder_core_deprecation_notice();
        }
    ));
}
