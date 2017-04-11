# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## 0.1.3 - 2017-04-11

### Fixed
- `setter(skip)` honors struct-inherited and explicit defaults #68

## 0.1.2 - 2017-04-10
### Added
- Bindings to abstract over libstd/libcore

### Changed
- Use `bindings: Bindings` instead of `no_std: bool`

### Fixed
- support generic references in structs #55
- no_std support #63

## 0.1.1 - 2017-04-08
### Added
- struct default

## 0.1 - 2017-03-25
### Added
- helper crate `derive_builder_core`:
  Allow `derive_builder` to use its own code generation technique.
- helper structs implementing `quote::ToTokens`:
  Allow unit tests on code generation items.
