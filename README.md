[![Build Status](https://travis-ci.org/colin-kiegel/rust-derive-builder.svg?branch=master)](https://travis-ci.org/colin-kiegel/rust-derive-builder)
[![Coverage Status](https://coveralls.io/repos/github/colin-kiegel/rust-derive-builder/badge.svg?branch=master)](https://coveralls.io/github/colin-kiegel/rust-derive-builder?branch=master)
[![Latest Version](https://img.shields.io/crates/v/derive_builder.svg)](https://crates.io/crates/derive_builder)

# Builder pattern derive

[Rust][rust] macro to automatically implement the **builder pattern** for arbitrary structs. A simple `#[derive(Builder)]` will generate code of public setter-methods for all struct fields.

**This is a work in progress.** Use it at your own risk.

And this is how it works:

```rust
#![feature(proc_macro)]
#[macro_use] extern crate derive_builder;

#[derive(Default, Builder)]
struct Channel {
    token: i32,
    special_info: i32,
    // .. a whole bunch of other fields ..
}

impl Channel {
    // All that's left to do for you is writing a method,
    // which actually *does* something. :-)
    pub fn build(&self) -> String {
        format!("The meaning of life is {}.", self.special_info)
    }
}

fn main() {
    // builder pattern, go, go, go!...
    let x = Channel::default().special_info(42u8).build();
    println!("{:?}", x);
}
```

Note that we did not write any implementation of a method called `special_info`. Instead the `derive_builder` crate acts on a `#[derive(Builder)]` and generates the necessary code at compile time.

The automatically generated setter method for the `special_info` field will look like this:

```rust
pub fn special_info<VALUE: Into<i32>>(&mut self, value: VALUE) -> &mut Self {
    self.special_info = value.into();
    self
}
```

## Usage and Features

* **Chaining**: The setter calls can be chained, because they consume and return `&mut self`.
* **Extensible**: You can still define your own implementation of the struct and define additional methods. Just make sure to name them differently than the fields.
* **Setter type conversions**: Setter methods are generic over the input types – you can supply every argument that implements the [`Into`][into] trait for the field type.
* **Generic structs**: Are also supported, but you **must not** use a type parameter named `VALUE`, because this is already reserved for the setter-methods.
* **Documentation and attributes**: Setter methods can be documented by simply documenting the corresponding field. Similarly `#[cfg(...)]` and `#[allow(...)]` attributes are also applied to the setter methods.

## Gotchas

* Tuple structs and unit structs are not supported as they have no field names. We do not intend to support them.
* When defining a generic struct, you cannot use `VALUE` as a generic parameter as this is what all setters are using.
* This crate exports a macro named `Builder!`, make sure you don't use this name for another macro.
* If you hit the macro recursion limit, you can increase it by adding this `#![recursion_limit="128"]` to your crate (default is `64`).

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
