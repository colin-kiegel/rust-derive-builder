//! Types and functions for parsing attribute options.
//!
//! Attribute parsing occurs in multiple stages:
//!
//! 1. Builder options on the struct are parsed into `OptionsBuilder<StructMode>`.
//! 1. The `OptionsBuilder<StructMode>` instance is converted into a starting point for the
//!    per-field options (`OptionsBuilder<FieldMode>`) and the finished struct-level config,
//!    called `StructOptions`.
//! 1. Each struct field is parsed, with discovered attributes overriding or augmenting the
//!    options specified at the struct level. This creates one `OptionsBuilder<FieldMode>` per
//!    struct field on the input/target type. Once complete, these get converted into
//!    `FieldOptions` instances.

mod darling_opts;
mod diagnostic;

use syn::meta::ParseNestedMeta;
use syn::{LitBool, LitStr};

pub use self::darling_opts::Options;
pub use self::diagnostic::Diagnostic;

pub(crate) fn set<T>(meta: &ParseNestedMeta, out: &mut Option<T>, value: T, diag: &mut Diagnostic) {
    if out.is_some() {
        diag.push(meta.error("duplicate attribute"));
    } else {
        *out = Some(value);
    }
}

/// Parse boolean value for a nested attribute in one of the following forms:
/// `k = true`, `k = "true"`, or just `k` which means true.
pub(crate) fn parse_optional_bool(meta: &ParseNestedMeta) -> syn::Result<bool> {
    if meta.input.peek(Token![=]) {
        let value = meta.value()?;
        let lit: LitBool = if value.peek(LitStr) {
            value.parse::<LitStr>()?.parse()?
        } else {
            value.parse()?
        };
        Ok(lit.value)
    } else {
        Ok(true)
    }
}
