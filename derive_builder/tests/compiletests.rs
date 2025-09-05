#![cfg(compiletests)]

extern crate rustversion;
extern crate trybuild;

#[rustversion::stable(1.89.0)]
#[test]
fn compile_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/*.rs");
    t.compile_fail("tests/compile-fail/*.rs");
}
