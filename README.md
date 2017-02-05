[![Build Status](https://travis-ci.org/colin-kiegel/rust-derive-builder.svg?branch=master)](https://travis-ci.org/colin-kiegel/rust-derive-builder)
[![Coverage Status](https://coveralls.io/repos/github/colin-kiegel/rust-derive-builder/badge.svg?branch=master)](https://coveralls.io/github/colin-kiegel/rust-derive-builder?branch=master)
[![Latest Version](https://img.shields.io/crates/v/derive_builder.svg)](https://crates.io/crates/derive_builder)
[![Documentation](https://docs.rs/derive_builder/badge.svg)](https://docs.rs/derive_builder)

# Builder pattern derive

[Rust][rust] macro to automatically implement the **builder pattern** for arbitrary structs. A simple `#[derive(Builder)]` will generate a `FooBuilder` for your struct `Foo` with all setter-methods and a build method.

**This requires Rust 1.15, due to the usage of Macros 1.1.**

And this is how it works:

```rust
#[macro_use]
extern crate derive_builder;

#[derive(Default, Builder, Debug)]
struct Channel {
    token: i32,
    special_info: i32,
    // .. a whole bunch of other fields ..
}

fn main() {
    // builder pattern, go, go, go!...
    let ch = ChannelBuilder::default()
        .special_info(42u8)
        .token(19124)
        .build()
        .unwrap();
    println!("{:?}", ch);
}
```

Note that we did not write any definition or implementation of `ChannelBuilder`. Instead the `derive_builder` crate acts on `#[derive(Builder)]` and generates the necessary code at compile time.

This is the generated boilerplate code you didn't need to write. :-)

```rust,ignore
#[derive(Clone, Default)]
struct ChannelBuilder {
    token: Option<i32>,
    special_info: Option<i32>,
}

#[allow(dead_code)]
impl ChannelBuilder {
    pub fn token<VALUE: Into<i32>>(&mut self, value: VALUE) -> &mut Self {
        let mut new = self;
        new.token = Some(value.into());
        new
    }
    pub fn special_info<VALUE: Into<i32>>(&mut self, value: VALUE) -> &mut Self {
        let mut new = self;
        new.special_info = Some(value.into());
        new
    }
    fn build(&self) -> Result<Channel, String> {
        Ok(Channel {
            token: Clone::clone(self.token
                .as_ref()
                .ok_or(
                       "token must be initialized")?),
            special_info: Clone::clone(self.special_info
                .as_ref()
                .ok_or("special_info must be initialized")?),
        })
    }
}
```

## Get Started

It's as simple as two steps:

1. Add `derive_builder` to your `Cargo.toml` either manually or
with [cargo-edit](https://github.com/killercup/cargo-edit):

  * `cargo add derive_builder`
2. Annotate your struct with `#[derive(Builder)]`

## Usage and Features

* **Chaining**: The setter calls can be chained, because they consume and return `&mut self` by default.
* **Builder patterns**: You can opt into other builder patterns by preceding your struct (or field) with `#[builder(pattern="owned")]` or `#[builder(pattern="immutable")]`.
* **Extensible**: You can still define your own implementations for the builder struct and define additional methods. Just make sure to name them differently than the setter and build methods.
* **Documentation and attributes**: Setter methods can be documented by simply documenting the corresponding field. Similarly `#[cfg(...)]` and `#[allow(...)]` attributes are also applied to the setter methods.
* **Visibility**: You can opt into private setter by preceding your struct with `#[builder(private)]`.
* **Setter type conversions**: Setter methods are generic over the input types – you can supply every argument that implements the [`Into`][into] trait for the field type.
* **Generic structs**: Are also supported, but you **must not** use a type parameter named `VALUE`, because this is already reserved for the setter-methods.
* **Logging**: If anything works unexpectedly you can enable detailed logs by setting this environment variable before calling cargo `RUST_LOG=derive_builder=trace`.

For more information and examples please take a look at our [documentation][doc].

This is a work in progress. So expect even more features in the future. :-)

## Gotchas

* Tuple structs and unit structs are not supported as they have no field names. We do not intend to support them.
* When defining a generic struct, you cannot use `VALUE` as a generic parameter as this is what all setters are using.

## [Documentation][doc]

The builder pattern is explained [here][builder-pattern], including its variants.

[doc]: https://colin-kiegel.github.io/rust-derive-builder
[rust]: https://www.rust-lang.org/
[builder-pattern]: https://aturon.github.io/ownership/builders.html
[into]: https://doc.rust-lang.org/nightly/std/convert/trait.Into.html

## [Changelog](CHANGELOG.md)

Yes, we keep a changelog.

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
