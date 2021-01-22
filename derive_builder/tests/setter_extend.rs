#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Lorem {
    #[builder(extend)]
    foo: String,
    #[builder(extend)]
    bar: Vec<String>,
    #[builder(extend)]
    baz: HashMap<String, i32>,
}

#[test]
fn generic_field() {
    let x = LoremBuilder::default()
        .foo("foo".into())
        .bar_extend_one("bar".into())
        .bar_extend_one("bar bar".into())
        .bar_extend_one("bar bar bar".into())
        .foo_extend_one('-')
        .baz_extend_one(("baz".into(), 1))
        .baz_extend_one(("bazz".into(), 2))
        .baz_extend_one(("bazzz".into(), 3))
        .foo_extend_one("foo")
        .build()
        .unwrap();

    assert_eq!(
        x,
        Lorem {
            foo: "foo-foo".into(),
            bar: vec!["bar".into(), "bar bar".into(), "bar bar bar".into()],
            baz: vec![("baz".into(), 1), ("bazz".into(), 2), ("bazzz".into(), 3)]
                .into_iter()
                .collect(),
        }
    );
}
