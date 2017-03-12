// DISABLED ////////////////////////////////////////////////////////////////////////
// summary: this combination causes trouble
// - rust-skeptic
// - cargo check
// - cargo test
// - on a proc_macro crate
//
// => I decided to disable these tests for now,
//    because I use both cargo check and test regularly.
#[ignore]
#[test]
fn expand_skeptic_tests() {
    panic!("weird bugs `error[E0464]: multiple matching crates for `derive_builder`, \
    see e.g. https://github.com/brson/rust-skeptic/issues/18.")

}
////////////////////////////////////////////////////////////////////////////////////

// include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));
