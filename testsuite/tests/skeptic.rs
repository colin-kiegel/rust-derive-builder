#![cfg(feature = "skeptic_tests")]

// NOTE this combination may cause
//      `error[E0464]: multiple matching crates for `derive_builder`
//
// - rust-skeptic
// - cargo check
// - cargo test
// - on a proc_macro crate
//
// => see https://github.com/brson/rust-skeptic/issues/18

include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));
