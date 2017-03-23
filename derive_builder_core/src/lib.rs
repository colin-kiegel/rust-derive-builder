//! Internal helper library for the `derive_builder` crate.
//!
//! **Important Note**:
//!
//! * You are probably looking for the [`derive_builder`] crate,
//!   which wraps this crate and is much more ergonomic to use.
//! * The API of this crate might **change frequently** in the near future.
//!   The [`derive_builder`] crate also provides a much more stable API.
//!
//! ## Purpose
//!
//! This is an internal helper library of [`derive_builder`]. Its purpose is to
//! allow [`derive_builder`] to use its own code generation technique, if needed.
//!
//! [`derive_builder_core`] might also be used in crates that [`derive_builder`] depends on -
//! again to break a dependency cycle.
//!
//! If [`derive_builder`] does not itself depend on _your_ crate, then you
//! should consider using [`derive_builder`] instead of [`derive_builder_core`].
//!
//! [`derive_builder`]: https://!crates.io/crates/derive_builder
//! [`derive_builder_core`]: https://!crates.io/crates/derive_builder_core

#![deny(warnings, missing_docs)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate log;

mod build_method;
mod builder_field;
mod builder;
mod deprecation_notes;
mod doc_comment;
mod initializer;
mod setter;
mod options;
mod block;

pub use build_method::BuildMethod;
pub use builder_field::BuilderField;
pub use builder::Builder;
pub use deprecation_notes::DeprecationNotes;
pub use initializer::Initializer;
pub use setter::Setter;
pub use doc_comment::doc_comment_from;
pub use options::BuilderPattern;
pub use block::Block;
