//! Internal helper library for the `derive_builder` crate.
//!
//! **Important Note**:
//!
//! * You are probably looking for the [`derive_builder`] crate,
//!   which wraps this crate and is much more ergonomic to use.
//!
//! ## Purpose
//!
//! This is an internal helper library of [`derive_builder`], which allows for
//! all the logic of builder creation to be decoupled from the proc-macro entry
//! point.
//!
//!
//! [`derive_builder`]: https://!crates.io/crates/derive_builder
//! [`derive_builder_core`]: https://!crates.io/crates/derive_builder_core

#![deny(warnings, missing_docs)]
// Uninlined format args are required to maintain MSRV of 1.56
#![allow(clippy::uninlined_format_args)]
#![cfg_attr(test, recursion_limit = "100")]

#[macro_use]
extern crate darling;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
mod block;
mod build_method;
mod builder;
mod builder_field;
mod change_span;
mod default_expression;
mod doc_comment;
mod initializer;
mod macro_options;
mod options;
mod setter;

pub(crate) use block::BlockContents;
pub(crate) use build_method::BuildMethod;
pub(crate) use builder::Builder;
pub(crate) use builder_field::{BuilderField, BuilderFieldType};
pub(crate) use change_span::change_span;
use darling::FromDeriveInput;
pub(crate) use default_expression::DefaultExpression;
pub(crate) use doc_comment::doc_comment_from;
pub(crate) use initializer::{FieldConversion, Initializer};
pub(crate) use options::{BuilderPattern, Each};
use quote::ToTokens;
pub(crate) use setter::Setter;

const DEFAULT_STRUCT_NAME: &str = "__default";

/// Derive a builder for a struct
pub fn builder_for_struct(ast: syn::DeriveInput) -> proc_macro2::TokenStream {
    match macro_options::Options::from_derive_input(&ast) {
        Ok(val) => val.as_builder().into_token_stream(),
        Err(err) => err.write_errors(),
    }
}
