//! This module is organizes chunks of documentation templates
//! for the generated code.
//!
//! Documentation templates may contain the following placeholders
//! - {struct_name}
//! - {builder_name}
//! - {field_name}
//!
//! The `build.rs` script will generate documentation tests with the help
//! of `skeptic`.
//!
//! Templates are used like this:
//!
//! ```rust,ignore
//! let builder_struct_doc = format!(
//!     include_str!("doc_tpl/builder_struct.md"),
//!     struct_name = /*..*/,
//!     builder_name = /*..*/),
//!     field_name = /*..*/);
//! ```
