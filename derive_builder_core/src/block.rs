use std::str::FromStr;
use quote::{Tokens, ToTokens};
use syn::{self, TokenTree};

/// A permissive wrapper for expressions/blocks, implementing `quote::ToTokens`.
///
/// - **full access** to variables environment.
/// - **full access** to control-flow of the environment via `return`, `?` etc.
///
/// # Examples
///
/// Will expand to something like the following (depending on settings):
///
/// ```rust
/// # #[macro_use]
/// # extern crate quote;
/// # extern crate derive_builder_core;
/// # use std::str::FromStr;
/// # use derive_builder_core::Block;
/// # fn main() {
/// #    let expr = Block::from_str("x+1").unwrap();
/// #    assert_eq!(quote!(#expr), quote!(
/// { x + 1 }
/// #    ));
/// # }
/// ```
#[derive(Debug, Default, Clone)]
pub struct Block(Vec<TokenTree>);

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let inner = &self.0;
        tokens.append(quote!(
            { #( #inner )* }
        ));
    }
}

impl FromStr for Block {
    type Err = String;

    /// Parses a string `s` to return a `Block`.
    ///
    /// # Errors
    ///
    /// When `expr` cannot be parsed as `Vec<syn::TokenTree>`. E.g. unbalanced
    /// opening/closing delimiters like `{`, `(` and `[` will be _rejected_ as
    /// parsing error.
    fn from_str(expr: &str) -> Result<Self, Self::Err> {
        Ok(Block(syn::parse_token_trees(expr)?))
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[should_panic(expected="called `Result::unwrap()` on an `Err` value: \
    \"unparsed tokens after token trees: \\\"{ x+1\\\"")]
    fn block_invalid_token_trees() {
        Block::from_str("let x = 2; { x+1").unwrap();
    }

    #[test]
    fn block_delimited_token_tree() {
        let expr = Block::from_str("let x = 2; { x+1 }").unwrap();
        assert_eq!(quote!(#expr), quote!(
            { let x = 2; { x+1 } }
        ));
    }

    #[test]
    fn block_single_token_tree() {
        let expr = Block::from_str("42").unwrap();
        assert_eq!(quote!(#expr), quote!(
            { 42 }
        ));
    }
}
