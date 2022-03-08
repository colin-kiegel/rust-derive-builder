use proc_macro2::TokenStream;
use quote::ToTokens;
use syn;

/// A wrapper for syn types which impl FromMeta, parsing them from a stirng literal
#[derive(Debug, Clone)]
pub struct ParsedLiteral<T>(pub T);

impl<T: ToTokens> ToTokens for ParsedLiteral<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl<T: syn::parse::Parse> darling::FromMeta for ParsedLiteral<T> {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(s) = value {
            Ok(ParsedLiteral(s.parse()?))
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }
}
