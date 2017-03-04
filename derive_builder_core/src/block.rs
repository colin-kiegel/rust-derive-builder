use std::str::FromStr;
use quote::{Tokens, ToTokens};
use syn::{self, TokenTree};

/// A safeguarded wrapper for expressions/blocks, implementing `quote::ToTokens`.
pub enum WrappedBlock {
    /// A **restrictive**/sandboxed wrapper, e.g. `{ let f = || {...}; f() }`
    ///
    /// Without side-effects on control-flow.
    Sandboxed(SandboxedBlock),
    /// A **permissive** wrapper `{...}`
    ///
    ///  With possible side-effects on control-flow.
    Block(Block),
}

impl ToTokens for WrappedBlock {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(match *self {
            WrappedBlock::Sandboxed(ref x) => quote!(#x),
            WrappedBlock::Block(ref x) => quote!(#x),
        })
    }
}

impl From<Block> for WrappedBlock {
    fn from(x: Block) -> WrappedBlock {
        WrappedBlock::Block(x)
    }
}

impl From<SandboxedBlock> for WrappedBlock {
    fn from(x: SandboxedBlock) -> WrappedBlock {
        WrappedBlock::Sandboxed(x)
    }
}

/// A restrictive/sandboxed wrapper for expressions/blocks, implementing `quote::ToTokens`.
///
/// Wraps everything inside a call to a closure `|| {...}`.
///
/// - **controlled access** to variables environment.
/// - **no access** to control-flow of the environment via `return`, `?` etc.
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
/// # use derive_builder_core::{SandboxedBlock, Block};
/// # fn main() {
/// #     let expr = SandboxedBlock {
/// #         body: Block::from_str("x+1").unwrap(),
/// #         mut_env: false,
/// #         move_env: false,
/// #     };
/// #     assert_eq!(quote!(#expr), quote!(
/// {
///     let f = || { x + 1 };
///     f()
/// }
/// #     ));
/// # }
/// ```
#[derive(Debug, Default, Clone)]
pub struct SandboxedBlock{
    /// The closure body.
    pub body: Block,
    /// Allow mutation of environment variables.
    pub mut_env: bool,
    /// Move environment values into the closure.
    pub move_env: bool,
}

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

impl ToTokens for SandboxedBlock {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let body = &self.body;
        let mut_env = if self.mut_env { quote!(mut) } else { quote!() };
        let move_env = if self.move_env { quote!(move) } else { quote!() };
        tokens.append(quote!(
            {
                let #mut_env f = #move_env || #body;
                f()
            }
        ));
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let inner = &self.0;
        tokens.append(quote!(
            { #( #inner )* }
        ));
    }
}

/// Errors
///
/// When `expr` cannot be parsed as `Vec<syn::TokenTree>`, e.g. unbalanced
/// opening/closing delimiters like `{`, `(` and `[` will be _rejected_ as parsing error.
impl FromStr for Block {
    type Err = String;

    fn from_str(expr: &str) -> Result<Self, Self::Err> {
        Ok(Block(syn::parse_token_trees(expr)?))
    }
}

#[cfg(test)]
mod test{
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

    #[test]
    fn sandboxed_move_environment() {
        let expr = SandboxedBlock {
            body: Block::from_str("42").unwrap(),
            mut_env: false,
            move_env: true,
        };
        assert_eq!(quote!(#expr), quote!(
            {
                let f = move || { 42 };
                f()
            }
        ));
    }

    #[test]
    fn sandboxed_mut_environment() {
        let expr = SandboxedBlock {
            body: Block::from_str("42").unwrap(),
            mut_env: true,
            move_env: false,
        };
        assert_eq!(quote!(#expr), quote!(
            {
                let mut f = || { 42 };
                f()
            }
        ));
    }
}
