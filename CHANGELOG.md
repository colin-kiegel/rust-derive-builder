# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

- migration to macros 1.1
- different setter pattern, e.g. `#[setters(immutable)]`
- private setters, e.g. `#[setters(private)]`
- additional debug info via env_logger, e.g. `RUST_LOG=derive_builder=trace cargo test`

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

[Unreleased]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.2.0...HEAD
[0.2.0]:  https://github.com/colin-kiegel/rust-derive-builder/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/colin-kiegel/rust-derive-builder/tree/v0.1.0
