# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## Unreleased - YYYY-MM-DD
### Added
- helper crate `derive_builder_core`:
  Allow `derive_builder` to use its own code generation technique.
- helper structs implementing `quote::ToTokens`:
  Allow unit tests on code generation items.
