use crate::macro_options::{parse_optional_bool, set, Diagnostic};
use syn::meta::ParseNestedMeta;
use syn::{token, Ident, LitStr};

/// Controls the signature of a setter method,
/// more specifically how `self` is passed and returned.
///
/// It can also be generalized to methods with different parameter sets and
/// return types, e.g. the `build()` method.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BuilderPattern {
    /// E.g. `fn bar(self, bar: Bar) -> Self`.
    Owned,
    /// E.g. `fn bar(&mut self, bar: Bar) -> &mut Self`.
    Mutable,
    /// E.g. `fn bar(&self, bar: Bar) -> Self`.
    ///
    /// Note:
    /// - Needs to `clone` in order to return an _updated_ instance of `Self`.
    /// - There is a great chance that the Rust compiler (LLVM) will
    ///   optimize chained `clone` calls away in release mode.
    ///   Therefore this turns out not to be as bad as it sounds.
    Immutable,
}

impl BuilderPattern {
    pub(crate) fn parse_nested_meta(
        meta: &ParseNestedMeta,
        diag: &mut Diagnostic,
    ) -> syn::Result<Self> {
        let lit: LitStr = meta.value()?.parse()?;
        Ok(match lit.value().as_str() {
            "owned" => BuilderPattern::Owned,
            "mutable" => BuilderPattern::Mutable,
            "immutable" => BuilderPattern::Immutable,
            unknown => {
                let msg = format!("unknown literal value `{}`", unknown);
                diag.push(syn::Error::new(lit.span(), msg));
                Self::default()
            }
        })
    }

    /// Returns true if this style of builder needs to be able to clone its
    /// fields during the `build` method.
    pub fn requires_clone(&self) -> bool {
        *self != Self::Owned
    }
}

/// Defaults to `Mutable`.
impl Default for BuilderPattern {
    fn default() -> Self {
        Self::Mutable
    }
}

#[derive(Debug, Clone)]
pub struct Each {
    pub name: syn::Ident,
    pub into: bool,
}

impl Each {
    /// Create `Each` from an attribute's `Meta`.
    ///
    /// Two formats are supported:
    ///
    /// * `each = "..."`, which provides the name of the `each` setter and otherwise uses default values
    /// * `each(name = "...")`, which allows setting additional options on the `each` setter
    pub(crate) fn parse_nested_meta(
        meta: &ParseNestedMeta,
        diag: &mut Diagnostic,
    ) -> syn::Result<Self> {
        let lookahead = meta.input.lookahead1();
        if lookahead.peek(Token![=]) {
            let name: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
            return Ok(Each { name, into: false });
        } else if !lookahead.peek(token::Paren) {
            return Err(lookahead.error());
        }

        let mut name: Option<syn::Ident> = None;
        let mut into: Option<bool> = None;

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value: Ident = meta.value()?.parse::<LitStr>()?.parse()?;
                set(&meta, &mut name, value, diag);
            } else if meta.path.is_ident("into") {
                let value = parse_optional_bool(&meta)?;
                set(&meta, &mut into, value, diag);
            } else {
                return Err(meta.error("unrecognized derive_builder attribute"));
            }
            Ok(())
        })?;

        Ok(Each {
            name: name
                .ok_or_else(|| syn::Error::new_spanned(&meta.path, "missing attribute `name`"))?,
            into: into.unwrap_or(false),
        })
    }
}
