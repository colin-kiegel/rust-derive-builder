# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added
- skip setters, e.g. `#[builder(setter(skip))]`

### Changed
- deprecated syntax `#[builder(setter_prefix="with")]`,
  please use `#[builder(setter(prefix="with"))]` instead

### Fixed
- use full path for result #39
- support `#[deny(missing_docs)]` #37

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

[Unreleased]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.3.0...HEAD
[0.3.0]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.2.1...v0.3.0
[0.2.1]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.2.0...v0.2.1
[0.2.0]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/colin-kiegel/rust-derive-builder/tree/v0.1.0
