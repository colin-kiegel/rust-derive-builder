#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Lorem {
    #[builder(sub_builder)]
    ipsum: Ipsum,

    #[builder(sub_builder(fn_name = "construct"), field(type = "DolorInput"))]
    dolor: DolorTarget,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Ipsum {
    i: usize,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(name = "DolorInput", build_fn(name = "construct"))]
struct DolorTarget {
    d: String,
}

#[test]
fn builder_test() {
    let mut x = LoremBuilder::default();
    x.ipsum.i(42);
    x.dolor.d(format!("dolor"));

    let expected = Lorem {
        ipsum: Ipsum { i: 42 },
        dolor: DolorTarget { d: "dolor".into() },
    };

    assert_eq!(x.build().unwrap(), expected);

    let x = LoremBuilder::default();
    assert_eq!(
        &x.build().unwrap_err().to_string(),
        "in ipsum: `i` must be initialized"
    );
}
