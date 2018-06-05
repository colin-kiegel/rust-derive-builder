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

use darling;

use derive_builder_core::Block;

mod darling_opts;

pub use self::darling_opts::Options;

/// A `DefaultExpression` can be either explicit or refer to the canonical trait.
#[derive(Debug, Clone)]
pub enum DefaultExpression {
    Explicit(String),
    Trait,
}

impl DefaultExpression {
    pub fn parse_block(&self, no_std: bool) -> Block {
        let expr = match *self {
            DefaultExpression::Explicit(ref s) => {
                if s.is_empty() {
                    panic!(r#"Empty default expressions `default = ""` are not supported."#);
                }
                s
            }
            DefaultExpression::Trait => if no_std {
                "::core::default::Default::default()"
            } else {
                "::std::default::Default::default()"
            },
        };

        expr.parse()
            .expect(&format!("Couldn't parse default expression `{:?}`", self))
    }
}

impl darling::FromMeta for DefaultExpression {
    fn from_word() -> darling::Result<Self> {
        Ok(DefaultExpression::Trait)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(DefaultExpression::Explicit(value.into()))
    }
}
