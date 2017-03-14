//! Derive a builder for a struct
//!
//! This crate implements the [builder pattern] for you.
//! Just apply `#[derive(Builder)]` to a struct `Foo`, and it will derive an additional
//! struct `FooBuilder` with **setter**-methods for all fields and a **build**-method
//! — the way you want it.
//!
//! # Quick Start
//!
//! Add `derive_builder` as a dependency to you `Cargo.toml`.
//!
//! ## What you write
//!
//! ```rust
//! #[macro_use]
//! extern crate derive_builder;
//!
//! #[derive(Builder)]
//! struct Lorem {
//!     ipsum: String,
//!     // ..
//! }
//! # fn main() {}
//! ```
//!
//! ## What you get
//!
//! ```rust
//! # #[macro_use]
//! # extern crate derive_builder;
//! #
//! # struct Lorem {
//! #     ipsum: String,
//! # }
//! # fn main() {}
//! #
//! #[derive(Clone, Default)]
//! struct LoremBuilder {
//!     ipsum: Option<String>,
//! }
//!
//! #[allow(dead_code)]
//! impl LoremBuilder {
//!     pub fn ipsum<VALUE: Into<String>>(&mut self, value: VALUE) -> &mut Self {
//!         let mut new = self;
//!         new.ipsum = Some(value.into());
//!         new
//!     }
//!     fn build(&self) -> Result<Lorem, String> {
//!         Ok(Lorem {
//!             ipsum: Clone::clone(self.ipsum
//!                 .as_ref()
//!                 .ok_or("ipsum must be initialized")?),
//!         })
//!     }
//! }
//! ```
//!
//! By default all generated setter-methods take and return `&mut self`
//! (aka _non-conusuming_ builder pattern). Accordingly, the build method also takes a
//! reference by default.
//!
//! You can easily opt into different patterns and control many other aspects.
//!
//! The build method returns `Result<T, String>`, where `T` is the struct you started with.
//! It returns `Err<String>` if you didn't initialize all fields.
//!
//! # Builder Patterns
//!
//! Let's look again at the example above. You can build structs like this:
//!
//! ```rust
//! # #[macro_use] extern crate derive_builder;
//! # #[derive(Builder)] struct Lorem { ipsum: String }
//! # fn try_main() -> Result<(), String> {
//! let x: Lorem = LoremBuilder::default().ipsum("42").build()?;
//! # Ok(())
//! # } fn main() { try_main().unwrap(); }
//! ```
//!
//! Ok, _chaining_ method calls is nice, but what if `ipsum("42")` should only happen if `geek = true`?
//!
//! So let's make this call conditional
//!
//! ```rust
//! # #[macro_use] extern crate derive_builder;
//! # #[derive(Builder)] struct Lorem { ipsum: String }
//! # fn try_main() -> Result<(), String> {
//! # let geek = true;
//! let mut builder = LoremBuilder::default();
//! if geek {
//!     builder.ipsum("42");
//! }
//! let x: Lorem = builder.build()?;
//! # Ok(())
//! # } fn main() { try_main().unwrap(); }
//! ```
//!
//! Now it comes in handy that our setter methods take and return mutable references. Otherwise
//! we would need to write something more clumsy like `builder = builder.ipsum("42")` to reassign
//! the return value each time we have to call a setter conditionally.
//!
//! Setters with mutable references are therefore a convenient default for the builder
//! pattern in Rust.
//!
//! But this is a free world and the choice is still yours!
//!
//! ## Owned, aka Consuming
//!
//! Precede your struct (or field) with `#[builder(pattern="owned")]` to opt into this pattern.
//!
//! * Setters take and return `self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: If you don't chain your calls, you have to create a reference to each return value,
//!   e.g. `builder = builder.ipsum("42")`.
//!
//! ## Mutable, aka Non-Comsuming (recommended)
//!
//! This pattern is recommended and active by default if you don't specify anything else.
//! You can precede your struct (or field) with `#[builder(pattern="mutable")]`
//! to make this choice explicit.
//!
//! * Setters take and return `&mut self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: The build method must clone or copy data to create something owned out of a
//!   mutable reference. Otherwise it could not be used in a chain. **(*)**
//!
//! ## Immutable
//!
//! Precede your struct (or field) with `#[builder(pattern="immutable")]` to opt into this pattern.
//!
//! * Setters take and return `&self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: If you don't chain your calls, you have to create a reference to each return value,
//!   e.g. `builder = builder.ipsum("42")`.
//! * CON: The build method _and each setter_ must clone or copy data to create something owned
//!   out of a reference. **(*)**
//!
//! ## (*) Performance Considerations
//!
//! Luckily Rust is clever enough to optimize these clone-calls away in release builds
//! for your every-day use cases. Thats quite a safe bet - we checked this for you. ;-)
//! Switching to consuming signatures (=`self`) is unlikely to give you any performance
//! gain, but very likely to restrict your API for non-chained use cases.
//!
//! # More Features
//!
//! ## Generic structs
//!
//! ```rust
//! # #[macro_use]
//! # extern crate derive_builder;
//! #
//! #[derive(Builder, Debug, PartialEq, Default, Clone)]
//! struct GenLorem<T: Clone> {
//!     ipsum: String,
//!     dolor: T,
//! }
//!
//! fn main() {
//!     let x = GenLoremBuilder::default().ipsum("sit").dolor(42).build().unwrap();
//!     assert_eq!(x, GenLorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! ## Documentation Comments and Attributes
//!
//! `#[derive(Builder)]` copies doc comments and attributes (`#[...]`) from your fields
//! to the according builder fields and setter-methods, if it is one of the following:
//!
//! * `/// ...`
//! * `#[doc = ...]`
//! * `#[cfg(...)]`
//! * `#[allow(...)]`
//!
//! The whitelisting minimizes interference with other custom attributes like
//! those used by Serde, Diesel, or others.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate derive_builder;
//! #
//! #[derive(Builder)]
//! struct Lorem {
//!     /// `ipsum` may be any `String` (be creative).
//!     ipsum: String,
//!     #[doc = r"`dolor` is the estimated amount of work."]
//!     dolor: i32,
//!     // `#[derive(Builder)]` understands conditional compilation via cfg-attributes,
//!     // i.e. => "no field = no setter".
//!     #[cfg(target_os = "macos")]
//!     #[allow(non_snake_case)]
//!     Im_a_Mac: bool,
//! }
//! # fn main() {}
//! ```
//!
//! ## Hidden Fields
//!
//! You can hide fields by skipping their setters on the builder struct.
//!
//! - Opt-out &mdash; skip setters via `#[builder(setter(skip))]` on individual fields.
//! - Opt-in &mdash; set `#[builder(setter(skip))]` on the whole struct
//!   and enable individual setters via `#[builder(setter)]`.
//!
//! The types of skipped fields must implement `Default`.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate derive_builder;
//! #
//! #[derive(Builder)]
//! struct SetterOptOut {
//!     setter_present: u32,
//!     #[builder(setter(skip))]
//!     setter_skipped: u32,
//! }
//! # fn main() {}
//! ```
//!
//! Alternatively, you can use the more verbose form:
//!
//! - `#[builder(setter(skip="true"))]`
//! - `#[builder(setter(skip="false"))]`
//!
//! ## Setter Visibility
//!
//! Setters are public by default. You can precede your struct (or field) with `#[builder(public)]`
//! to make this explicit.
//!
//! Otherwise precede your struct (or field) with `#[builder(private)]` to opt into private
//! setters.
//!
//! ## Setter Prefixes
//!
//! Setter methods are named after their corresponding field by default.
//!
//! You can precede your struct (or field) with e.g. `#[builder(setter(prefix="xyz"))` to change
//! the method name to `xyz_foo` if the field is named `foo`. Note that an underscore is included
//! by default, since Rust favors snake case here.
//!
//! # Troubleshooting
//!
//! ## Gotchas
//!
//! - Tuple structs and unit structs are not supported as they have no field
//!   names.
//! - When defining a generic struct, you cannot use `VALUE` as a generic
//!   parameter as this is what all setters are using.
//! - When re-exporting the underlying struct under a different name, the
//!   auto-generated documentation will not match.
//! - If derive_builder depends on your crate, and vice versa, then a cyclic
//!   dependency would occur. To break it you could try to depend on the
//!   [`derive_builder_core`] crate instead.
//!
//! ## Debugging Info
//!
//! If you experience any problems during compilation, you can enable additional debug output
//! by setting the environment variable `RUST_LOG=derive_builder=trace` before you call `cargo`
//! or `rustc`. Example: `env RUST_LOG=derive_builder=trace cargo test`.
//!
//! ## Report Issues and Ideas
//!
//! [Open an issue on GitHub](https://github.com/colin-kiegel/rust-derive-builder/issues)
//!
//! If possible please try to provide the debugging info if you experience unexpected
//! compilation errors (see above).
//!
//! [builder pattern]: https://aturon.github.io/ownership/builders.html
//! [`derive_builder_core`]: https://crates.io/crates/derive_builder_core

#![crate_type = "proc-macro"]
#![cfg_attr(feature = "deny_warnings", deny(warnings))]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate derive_builder_core;

mod options;

use proc_macro::TokenStream;
use std::sync::{Once, ONCE_INIT};
use options::{struct_options_from, field_options_from};

static INIT_LOGGER: Once = ONCE_INIT;

#[doc(hidden)]
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    INIT_LOGGER.call_once(|| {
        env_logger::init().unwrap();
    });

    let input = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = builder_for_struct(ast).to_string();
    debug!("generated tokens: {}", result);

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
}

fn builder_for_struct(ast: syn::MacroInput) -> quote::Tokens {
    debug!("Deriving Builder for `{}`.", ast.ident);
    let (opts, field_defaults) = struct_options_from(&ast);

    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields)) => fields,
        _ => panic!("`#[derive(Builder)]` can only be used with braced structs"),
    };

    let mut builder = opts.to_builder();
    let mut build_fn = opts.to_build_method();

    builder.doc_comment(format!(include_str!("doc_tpl/builder_struct.md"),
                                struct_name = ast.ident.as_ref()));
    build_fn.doc_comment(format!(include_str!("doc_tpl/builder_method.md"),
                                struct_name = ast.ident.as_ref()));

    for f in fields {
        let f_opts = field_options_from(f, &field_defaults);

        builder.push_field(f_opts.to_builder_field());
        builder.push_setter_fn(f_opts.to_setter());
        build_fn.push_initializer(f_opts.to_initializer());
    }

    builder.push_build_fn(build_fn);

    quote!(#builder)
}
