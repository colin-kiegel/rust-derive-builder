use std::str::FromStr;

use quote::{ToTokens, Tokens};
use syn;
use syn::synom::Parser;

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
#[derive(Debug, Clone)]
pub struct Block(Vec<syn::Stmt>);

impl Default for Block {
    fn default() -> Self {
        "".parse().unwrap()
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let inner = &self.0;
        tokens.append_all(quote!(
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
        named!(block_contents -> Vec<syn::Stmt>, call!(syn::Block::parse_within));
        Ok(Block(
            block_contents
                .parse_str(expr)
                .map_err(|e| format!("{}", e))?,
        ))
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[should_panic(
        expected = r#"called `Result::unwrap()` on an `Err` value: "error while lexing input string""#
    )]
    fn block_invalid_token_trees() {
        Block::from_str("let x = 2; { x+1").unwrap();
    }

    #[test]
    fn block_delimited_token_tree() {
        let expr = Block::from_str("let x = 2; { x+1 }").unwrap();
        assert_eq!(
            quote!(#expr),
            quote!({
                let x = 2;
                {
                    x + 1
                }
            })
        );
    }

    #[test]
    fn block_single_token_tree() {
        let expr = Block::from_str("42").unwrap();
        assert_eq!(quote!(#expr), quote!({ 42 }));
    }
}
