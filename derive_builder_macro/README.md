[![Build status](https://travis-ci.org/colin-kiegel/rust-derive-builder.svg?branch=master)](https://travis-ci.org/colin-kiegel/rust-derive-builder)
[![Documentation](https://docs.rs/derive_builder_macro/badge.svg)](https://docs.rs/derive_builder_macro)
[![Latest version](https://img.shields.io/crates/v/derive_builder_macro.svg)](https://crates.io/crates/derive_builder_macro)
[![All downloads](https://img.shields.io/crates/d/derive_builder_macro.svg)](https://crates.io/crates/derive_builder_macro)
[![Downloads of latest version](https://img.shields.io/crates/dv/derive_builder_macro.svg)](https://crates.io/crates/derive_builder_macro)

# Crate [`derive_builder_macro`]

**Important Note**:

* You are probably looking for the [`derive_builder`] crate,
  which re-exports the builder derive defined in this crate.
* Some features of the builder derive require items defined in
  [`derive_builder`].

## Purpose

Proc macro crates (like this one) can define custom derives, but cannot export
any other items. However _another_ crate like [`derive_builder`] can both
re-export the custom derive and export additional items. That's what we do.

[`derive_builder`]: https://crates.io/crates/derive_builder
[`derive_builder_macro`]: https://crates.io/crates/derive_builder_macro

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
