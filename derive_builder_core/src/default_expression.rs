use crate::BlockContents;
use quote::ToTokens;
use syn::{meta::ParseNestedMeta, Token};

/// A `DefaultExpression` can be either explicit or refer to the canonical trait.
#[derive(Debug, Clone)]
pub enum DefaultExpression {
    Explicit(BlockContents),
    Trait,
}

impl DefaultExpression {
    pub(crate) fn parse_nested_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.input.peek(Token![=]) {
            let block_contents = BlockContents::parse_nested_meta(meta)?;
            Ok(DefaultExpression::Explicit(block_contents))
        } else {
            Ok(DefaultExpression::Trait)
        }
    }

    /// Add the crate root path so the default expression can be emitted
    /// to a `TokenStream`.
    ///
    /// This function is needed because the crate root is inherited from the container, so it cannot
    /// be provided at parse time to [`darling::FromMeta::from_word`] when reading, and [`ToTokens`] does not
    /// accept any additional parameters, so it annot be provided at emit time.
    pub fn with_crate_root<'a>(&'a self, crate_root: &'a syn::Path) -> impl 'a + ToTokens {
        DefaultExpressionWithCrateRoot {
            crate_root,
            expr: self,
        }
    }
}

/// Wrapper for `DefaultExpression`
struct DefaultExpressionWithCrateRoot<'a> {
    crate_root: &'a syn::Path,
    expr: &'a DefaultExpression,
}

impl<'a> ToTokens for DefaultExpressionWithCrateRoot<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let crate_root = self.crate_root;
        match self.expr {
            DefaultExpression::Explicit(ref block) => block.to_tokens(tokens),
            DefaultExpression::Trait => quote!(
                #crate_root::export::core::default::Default::default()
            )
            .to_tokens(tokens),
        }
    }
}
