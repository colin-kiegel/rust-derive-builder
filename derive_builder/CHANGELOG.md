# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [0.4.3] - 2017-04-11

### Fixed
- `setter(skip)` honors struct-inherited and explicit defaults #68

## [0.4.2] - 2017-04-10

### Fixed
- support generic references in structs #55
- support `#![no_std]` #63

## [0.4.1] - 2017-04-08

### Deprecated
- `#[builder(default)]` and `#[builder(default="...")]` at the struct level will
  change their behaviour in 0.5.0 and construct a default value for the struct,
  instead of all fields individually. To opt into the new behaviour and squelch
  this deprecation warning you can add the `struct_default` feature flag.

## [0.4.0] - 2017-03-25

### Added
- skip setters, e.g. `#[builder(setter(skip))]`
- default values, e.g. `#[builder(default="42")]` or just `#[builder(default)]`

### Changed
- deprecated syntax `#[builder(setter_prefix="with")]`,
  please use `#[builder(setter(prefix="with"))]` instead
- setter conversions are now off by default, you can opt-into via
  `#[builder(setter(into))]`
- logging is behind a feature flag. To activate it, please add
  `features = ["logging"]` to the dependency in `Cargo.toml`. Then you can use
  it like: `RUST_LOG=derive_builder=trace cargo test`.

### Fixed
- use full path for result #39
- support `#[deny(missing_docs)]` #37
- support `#![no_std]` via `#[builder(no_std)]` #41

## [0.3.0] - 2017-02-05

Requires Rust 1.15 or newer.

### Added
- different setter pattern, e.g. `#[builder(pattern="immutable")]`
- private setters, e.g. `#[builder(private)]`
- additional debug info via env_logger, e.g.
  `RUST_LOG=derive_builder=trace cargo test`
- prefixes, e.g. `#[builder(setter_prefix="with")]`
- field specific overrides
- customize builder name, e.g. `#[builder(name="MyBuilder")]`

### Changed
- migration to macros 1.1
- migration to traditional builder pattern
  i.e. seperate `FooBuilder` struct to build `Foo`
=> please refer to the new docs

### Fixed
- missing lifetime support #21

## [0.2.1] - 2016-09-24

### Fixed
- preserve ordering of attributes #27

## [0.2.0] - 2016-08-22
### Added
- struct fields can be public
- struct fields can have attributes
- the following struct-attributes are copied to the setter-method
 - `/// ...`
 - `#[doc = ...]`
 - `#[cfg(...)]`
 - `#[allow(...)]`

### Changed
- setter-methods are non-consuming now -- breaking change
- setter-methods are public now

### Fixed
- automatic documentation does not work #16

## [0.1.0] - 2016-08-07
### Added
- first implementation
 - generate setter methods
 - support for generic structs

[Unreleased]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.4.0...HEAD
[0.4.2]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.4.1...v0.4.2
[0.4.1]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.4.0...v0.4.1
[0.4.0]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.3.0...v0.4.0
[0.3.0]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.2.1...v0.3.0
[0.2.1]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.2.0...v0.2.1
[0.2.0]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/colin-kiegel/rust-derive-builder/tree/v0.1.0
