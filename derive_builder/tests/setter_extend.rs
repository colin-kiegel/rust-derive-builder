#[macro_use]
extern crate derive_builder;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Lorem {
    #[builder(setter(each = "foo_append"))]
    foo: String,
    #[builder(setter(each = "bar"))]
    bars: Vec<String>,
    #[builder(setter(each = "baz"))]
    bazes: HashMap<String, i32>,
    #[builder(setter(strip_option, each = "qux"))]
    quxes: Option<Vec<String>>,
    #[builder(setter(strip_option, each = "quux"))]
    quuxes: Option<HashMap<String, i32>>,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(pattern = "mutable")]
struct Ipsum {
    #[builder(setter(each(name = "foo_append")))]
    foo: String,
    #[builder(setter(each(name = "bar")))]
    bars: Vec<String>,
    #[builder(setter(each(name = "baz")))]
    bazes: HashMap<String, i32>,
    #[builder(setter(strip_option, each = "qux"))]
    quxes: Option<Vec<String>>,
    #[builder(setter(strip_option, each = "quux"))]
    quuxes: Option<HashMap<String, i32>>,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder]
struct Dolor {
    #[builder(setter(each(name = "foo_append")))]
    foo: String,
    #[builder(setter(each(name = "bar", into)))]
    bars: Vec<String>,
    #[builder(setter(each(name = "baz")))]
    bazes: HashMap<String, i32>,
}

#[test]
fn extend_field() {
    let x = LoremBuilder::default()
        .foo("foo".into())
        .bar("bar".into())
        .bar("bar bar".into())
        .bar("bar bar bar".into())
        .foo_append('-')
        .baz(("baz".into(), 1))
        .baz(("bazz".into(), 2))
        .baz(("bazzz".into(), 3))
        .foo_append("foo")
        .qux("qux".into())
        .qux("qux qux".into())
        .quux(("quux".into(), 1))
        .quux(("quuxx".into(), 2))
        .build()
        .unwrap();

    assert_eq!(
        x,
        Lorem {
            foo: "foo-foo".into(),
            bars: vec!["bar".into(), "bar bar".into(), "bar bar bar".into()],
            bazes: vec![("baz".into(), 1), ("bazz".into(), 2), ("bazzz".into(), 3)]
                .into_iter()
                .collect(),
            quxes: Some(vec!["qux".into(), "qux qux".into()]),
            quuxes: Some(
                vec![("quux".into(), 1), ("quuxx".into(), 2)]
                    .into_iter()
                    .collect()
            ),
        }
    );
}

#[test]
fn extend_field_into() {
    let x = DolorBuilder::default()
        .foo("foo".into())
        .bar("bar")
        .bar("bar bar")
        .bar("bar bar bar")
        .foo_append('-')
        .baz(("baz".into(), 1))
        .baz(("bazz".into(), 2))
        .baz(("bazzz".into(), 3))
        .foo_append("foo")
        .build()
        .unwrap();

    assert_eq!(
        x,
        Dolor {
            foo: "foo-foo".into(),
            bars: vec!["bar".into(), "bar bar".into(), "bar bar bar".into()],
            bazes: vec![("baz".into(), 1), ("bazz".into(), 2), ("bazzz".into(), 3)]
                .into_iter()
                .collect(),
        }
    );
}

#[test]
fn extend_field_mutable() {
    let x = IpsumBuilder::default()
        .foo("foo".into())
        .bar("bar".into())
        .bar("bar bar".into())
        .bar("bar bar bar".into())
        .foo_append('-')
        .baz(("baz".into(), 1))
        .baz(("bazz".into(), 2))
        .baz(("bazzz".into(), 3))
        .foo_append("foo")
        .qux("qux".into())
        .qux("qux qux".into())
        .quux(("quux".into(), 1))
        .quux(("quuxx".into(), 2))
        .build()
        .unwrap();

    assert_eq!(
        x,
        Ipsum {
            foo: "foo-foo".into(),
            bars: vec!["bar".into(), "bar bar".into(), "bar bar bar".into()],
            bazes: vec![("baz".into(), 1), ("bazz".into(), 2), ("bazzz".into(), 3)]
                .into_iter()
                .collect(),
            quxes: Some(vec!["qux".into(), "qux qux".into()]),
            quuxes: Some(
                vec![("quux".into(), 1), ("quuxx".into(), 2)]
                    .into_iter()
                    .collect()
            ),
        }
    );
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(setter(skip))]
struct Sit {
    #[builder(setter(each(name = "foo")))]
    foos: Vec<i32>,
}

#[test]
fn extend_field_enabled() {
    let x = SitBuilder::default().foo(1).foo(2).build().unwrap();
    assert_eq!(x, Sit { foos: vec![1, 2] });
}

#[derive(Debug, PartialEq, Default, Builder)]
#[builder(pattern = "owned")]
struct CustomFieldOwned {
    #[builder(
        field(
            ty = "Vec<i32>",
            build = "self.buggy.into_iter().map(|e| (e, e)).collect()"
        ),
        setter(each(name = "add_buggy", into))
    )]
    buggy: HashMap<i32, i32>,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct CustomFieldMutable {
    #[builder(
        field(
            ty = "Vec<String>",
            build = "self.items.iter().cloned().enumerate().map(|(i, s)| (s, i)).collect()"
        ),
        setter(each(name = "add_item", into))
    )]
    items: HashMap<String, usize>,
    #[builder(
        field(ty = "Option<Vec<i32>>", build = "self.opt_items.clone()"),
        setter(strip_option, each = "add_opt_item")
    )]
    opt_items: Option<Vec<i32>>,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(pattern = "immutable")]
struct CustomFieldImmutable {
    #[builder(
        field(ty = "Vec<i32>", build = "self.nums.iter().copied().sum()"),
        setter(each = "add_num")
    )]
    nums: i32,
}

#[test]
fn extend_custom_field_owned() {
    let empty = CustomFieldOwnedBuilder::default().build().unwrap();
    assert_eq!(
        empty,
        CustomFieldOwned {
            buggy: HashMap::new()
        }
    );

    let built = CustomFieldOwnedBuilder::default()
        .add_buggy::<i32, _>(1i16)
        .add_buggy::<i32, _>(2)
        .build()
        .unwrap();
    assert_eq!(
        built,
        CustomFieldOwned {
            buggy: vec![(1, 1), (2, 2)].into_iter().collect(),
        }
    );
}

#[test]
fn extend_custom_field_mutable_and_strip_option() {
    let default_built = CustomFieldMutableBuilder::default().build().unwrap();
    assert_eq!(
        default_built,
        CustomFieldMutable {
            items: HashMap::new(),
            opt_items: None,
        }
    );

    let built = CustomFieldMutableBuilder::default()
        .add_item("first")
        .add_item("second")
        .add_opt_item(10)
        .add_opt_item(20)
        .build()
        .unwrap();
    assert_eq!(
        built,
        CustomFieldMutable {
            items: vec![("first".to_string(), 0), ("second".to_string(), 1)]
                .into_iter()
                .collect(),
            opt_items: Some(vec![10, 20]),
        }
    );
}

#[test]
fn extend_custom_field_immutable() {
    let base = CustomFieldImmutableBuilder::default().add_num(3);
    let a = base.add_num(4).build().unwrap();
    let b = base.add_num(10).build().unwrap();
    assert_eq!(a, CustomFieldImmutable { nums: 7 });
    assert_eq!(b, CustomFieldImmutable { nums: 13 });
}
