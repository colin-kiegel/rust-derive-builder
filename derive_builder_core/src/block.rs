use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    meta::ParseNestedMeta, parse::ParseStream, spanned::Spanned, token, Block, Expr, ExprLit, Lit,
    Stmt,
};

/// A wrapper for expressions/blocks which automatically adds the start and end
/// braces.
///
/// - **full access** to variables environment.
/// - **full access** to control-flow of the environment via `return`, `?` etc.
#[derive(Debug, Clone)]
pub struct BlockContents(Block);

impl BlockContents {
    #[cfg(test)]
    pub(crate) fn new(block: Block) -> Self {
        Self(block)
    }

    pub(crate) fn parse_nested_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        let expr: Expr = meta.value()?.parse()?;
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) = expr
        {
            Ok(Self(Block {
                brace_token: token::Brace(lit.span()),
                stmts: lit.parse_with(parse_nonempty_block)?,
            }))
        } else {
            Ok(Self(Block {
                brace_token: token::Brace(expr.span()),
                stmts: vec![Stmt::Expr(expr, None)],
            }))
        }
    }
}

fn parse_nonempty_block(input: ParseStream) -> syn::Result<Vec<Stmt>> {
    if input.is_empty() {
        Err(input.error("expected expression"))
    } else {
        Block::parse_within(input)
    }
}

impl ToTokens for BlockContents {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::MetaList;

    fn parse(s: &str) -> Result<BlockContents, syn::Error> {
        let mut block_contents = None;
        let attr: MetaList = parse_quote!(field(build = #s));
        attr.parse_nested_meta(|meta| {
            block_contents = Some(BlockContents::parse_nested_meta(&meta)?);
            Ok(())
        })
        .map(|()| block_contents.unwrap())
    }

    #[test]
    #[should_panic(expected = r#"cannot parse"#)]
    fn block_invalid_token_trees() {
        parse("let x = 2; { x+1").unwrap();
    }

    #[test]
    fn block_delimited_token_tree() {
        let expr = parse("let x = 2; { x+1 }").unwrap();
        assert_eq!(
            quote!(#expr).to_string(),
            quote!({
                let x = 2;
                {
                    x + 1
                }
            })
            .to_string()
        );
    }

    #[test]
    fn block_single_token_tree() {
        let expr = parse("42").unwrap();
        assert_eq!(quote!(#expr).to_string(), quote!({ 42 }).to_string());
    }
}
