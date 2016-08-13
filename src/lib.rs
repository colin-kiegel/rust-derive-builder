//! Derive a builder for a struct
//!
//! This crate implements the _non-consuming_ [builder pattern] as an extension of the
//! [custom_derive] macro.
//! When applied to a struct, it will derive **setter-methods** for all struct fields.
//!
//! **Please note**:
//!
//! * There are slightly different ways to implement the builder pattern in rust.
//!   The preferred way to do it, is the so called _non-consuming_ variant.
//!   That means: all generated setter-methods take and return `&mut self`.
//! * To complete the builder pattern you only have to implement at least one method
//!   which actually builds something based on the struct.
//!   These custom build methods of yours should also take `&mut self` to take advantage of the
//!   non-consuming pattern.
//! * **Don't worry at all** if you have to `clone` or `copy` data in your build methods,
//!   because luckily the Compiler is smart enough to optimize them away in release builds
//!   for your every-day use cases. Thats quite a safe bet - we checked this for you. ;-)
//!   Switching to consuming signatures (=`self`) would not give you any performance
//!   gain, but only restrict your API for every-day use cases
//!
//! [builder pattern]: https://aturon.github.io/ownership/builders.html
//! [custom_derive]: https://crates.io/crates/custom_derive
//!
//! # Examples
//!
//! This crate is best used in combination with [custom_derive].
//!
//! ```rust
//! #[macro_use] extern crate custom_derive;
//! #[macro_use] extern crate derive_builder;
//!
//! custom_derive!{
//!     #[derive(Debug, PartialEq, Default, Clone, Builder)]
//!     struct Lorem {
//!         ipsum: String,
//!         dolor: i32,
//!     }
//! }
//!
//! fn main() {
//!     let x = Lorem::default().ipsum("sit").dolor(42).clone();
//!     assert_eq!(x, Lorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! In `main()`: The final call of `clone()` represents the act of **building a new struct**
//! when our builder is ready. For the sake of brevity we chose clone and pretend we get
//! something brand new. As already mentioned, the compiler will optimize this away in release
//! mode.
//!
//! ## Generic structs
//!
//! ```rust
//! #[macro_use] extern crate custom_derive;
//! #[macro_use] extern crate derive_builder;
//!
//! custom_derive!{
//!     #[derive(Debug, PartialEq, Default, Clone, Builder)]
//!     struct GenLorem<T> {
//!         ipsum: String,
//!         dolor: T,
//!     }
//! }
//!
//! fn main() {
//!     let x = GenLorem::default().ipsum("sit").dolor(42).clone();
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
//!     #[derive(Builder)]
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
//! # fn main() {}
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
