#![cfg(feature = "try_from")]
#![feature(try_from)]

#[macro_use]
extern crate derive_builder;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into), try_setter)]
struct Foo {
    hello: String,
}

#[derive(Debug, Builder)]
#[builder(setter(into))]
struct Bar<'a> {
    hello: &'a str,

    #[builder(try_setter)]
    foo: Foo,
}

#[test]
fn simple() {
    FooBuilder::default().build().unwrap_err();
}

#[test]
fn nested() {
    nested_helper("world").unwrap();
}

fn nested_helper<'a>(name: &'a str) -> Result<Bar<'a>, String> {
    BarBuilder::default()
        .hello(name)
        .try_foo(FooBuilder::default().hello(name))?
        .build()
}
