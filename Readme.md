[![Build Status](https://travis-ci.org/colin-kiegel/rust-derive-builder.svg?branch=master)](https://travis-ci.org/colin-kiegel/rust-derive-builder)
[![Coverage Status](https://coveralls.io/repos/github/colin-kiegel/rust-derive-builder/badge.svg?branch=master)](https://coveralls.io/github/colin-kiegel/rust-derive-builder?branch=master)

# Builder pattern derive

[Rust][rust] macro based on [custom_derive][custom_derive] to automatically
implement the **builder pattern** for arbitrary structs. This is achieved by implementing setter-methods for all struct fields automatically. And this is how it works:

```rust
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

custom_derive!{
    #[derive(Debug, Default, Builder)]
    struct Channel {
        token: i32,
        special_info: i32
    }
}

fn main() {
    let ch = Channel::default().special_info(42);
    println!("{:?}", ch);
}
```

Note that we did not write any implementation of `Channel` or method called `special_info`. Instead macro `custom_derive!` scans the `#[derive(..)]` attribute of the struct for non-std Identifiers - in our case `#[derive(Builder)]` and delegates the code genaration to the extension defined in this crate.

The automatically generated setter-method for the `token` field will look like this:
```rust
fn token<VALUE: Into<i32>>(mut self, value: VALUE) -> Self {
    self.token = value.into();
    self
}
```

**This is a work in progress.** Use it at your own risk.

## Usage and Features

* **chaining**: the setter methods can be chained, because they consume and return `self`.
* **extensible**: you can still define your own implementation of the struct and define additional methods. Just make sure to name them differently than the fields.
* **generic setters**: the setter methods are generic and take every argument that implements the `Into` trait for the field type.
* **generic structs**: are also supported, but you **must not** use a type parameter named `VALUE`, because this is already reserved for the setter-methods.

## Gotchas

* Tuple structs and unit structs are not supported as they have no field names. We do not intend to support them.
* When defining a generic struct, you cannot use `VALUE` as a generic parameter as this is what all setters are using.
* This crate exports a macro named `Builder`, make sure you don't use this name for another macro.

## [Documentation][doc]

The builder pattern is explained [here][builder pattern]. But please note that there is a [debate](https://github.com/aturon/aturon.github.io/issues/5) about which variant is preferrable for what reasons.

[doc]: https://colin-kiegel.github.io/rust-derive-builder
[rust]: https://www.rust-lang.org/
[custom_derive]: https://crates.io/crates/custom_derive
[builder pattern]: https://aturon.github.io/ownership/builders.html

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
