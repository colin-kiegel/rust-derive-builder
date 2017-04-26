use quote::{Tokens, ToTokens};

/// `RawTokens` can be directly appended to a `quote::Tokens` instance without
/// any parsing.
#[derive(PartialEq, Debug)]
pub struct RawTokens<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> RawTokens<T> {
    /// View the underlying data as string.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Write `self` to a new instance of `quote::Tokens`.
    pub fn to_tokens(&self) -> Tokens {
        quote!(#self)
    }
}

impl<T: AsRef<str>> ToTokens for RawTokens<T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(&self.0);
    }
}
