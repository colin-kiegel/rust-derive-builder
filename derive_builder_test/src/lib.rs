#[macro_use]
extern crate derive_builder;

// re-export Foo and FooBuilder to check for documentation consistency
// https://github.com/colin-kiegel/rust-derive-builder/issues/46
pub use internal::Foo as Bar;
pub use internal::FooBuilder as BarBuilder;
mod internal {
    #[derive(Builder)]
    pub struct Foo {
        pub a: i32,
    }
}

#[test]
fn test() {
    let x = BarBuilder::default()
        .a(42)
        .build()
        .unwrap();
    println!("{}", x.a);
}
