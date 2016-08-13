//! Derive a builder for a struct
//!
//! Use in combination with [custom_derive][custom_derive].
//!
//! [custom_derive]: https://crates.io/crates/custom_derive
//!
//! # Examples
//!
//! ```rust
//! #[macro_use] extern crate custom_derive;
//! #[macro_use] extern crate derive_builder;
//!
//! custom_derive!{
//!     #[derive(Debug, PartialEq, Default, Builder)]
//!     struct Lorem {
//!         ipsum: String,
//!         dolor: i32,
//!     }
//! }
//!
//! fn main() {
//!     let x = Lorem::default().ipsum("sit").dolor(42);
//!     assert_eq!(x, Lorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! ## Generic structs
//!
//! ```rust
//! #[macro_use] extern crate custom_derive;
//! #[macro_use] extern crate derive_builder;
//!
//! custom_derive!{
//!     #[derive(Debug, PartialEq, Default, Builder)]
//!     struct GenLorem<T> {
//!         ipsum: String,
//!         dolor: T,
//!     }
//! }
//!
//! fn main() {
//!     let x = GenLorem::default().ipsum("sit").dolor(42);
//!     assert_eq!(x, GenLorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! ## Doc-Comments and Attributes
//!
//! `#[derive(Builder)]` copies doc-comments and attributes `#[...]` from your fields
//! to the according setter-method, if it is one of the following:
//!
//! * `/// ...`
//! * `#[doc = ...]`
//! * `#[cfg(...)]`
//! * `#[allow(...)]`
//!
//! ```rust
//! #[macro_use] extern crate custom_derive;
//! #[macro_use] extern crate derive_builder;
//!
//! custom_derive!{
//!     #[derive(Debug, PartialEq, Default, Builder)]
//!     struct Lorem {
//!         /// `ipsum` may be any `String` (be creative).
//!         ipsum: String,
//!         #[doc = r"`dolor` is the estimated amount of work."]
//!         dolor: i32,
//!         // `#[derive(Builder)]` understands conditional compilation via cfg-attributes,
//!         // i.e. => "no field = no setter".
//!         #[cfg(target_os = "macos")]
//!         #[allow(non_snake_case)]
//!         Im_a_Mac: bool,
//!     }
//! }
//!
//! fn main() {
//!     let x = Lorem::default().ipsum("sit").dolor(42);
//!     assert_eq!(x, Lorem { ipsum: "sit".into(), dolor: 42, ..Lorem::default() });
//! }
//! ```
//!
//! ## Gotchas
//!
//! - Tuple structs and unit structs are not supported as they have no field
//!   names.
//! - When defining a generic struct, you cannot use `VALUE` as a generic
//!   parameter as this is what all setters are using.

mod parse_struct;
mod derive_builder;
