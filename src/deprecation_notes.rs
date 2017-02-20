use quote::{Tokens, ToTokens};
use syn;

/// Deprecation notes we want to emit to the user.
#[derive(Debug, Default, Clone)]
pub struct DeprecationNotes(Vec<String>);

impl DeprecationNotes {
    /// Appends a note to the collection.
    pub fn push(&mut self, note: String) {
        self.0.push(note)
    }
}

/// This will emit a block which can be inserted into any fn body to emit a
/// deprecation warning in the downstream crate. Cool stuff. ^^
/// Proof of concept: https://play.rust-lang.org/?gist=8394141c07d1f6d75d314818389eb4d8&version=stable&backtrace=0
impl ToTokens for DeprecationNotes {
    fn to_tokens(&self, tokens: &mut Tokens) {
        for note in &self.0 {
            let fn_ident = syn::Ident::new("derive_builder_deprecation_notice");
            tokens.append(&quote!(
                {
                    #[deprecated(note=#note)]
                    fn #fn_ident() { }
                    #fn_ident();
                }
            ).to_string());
        }
    }
}
