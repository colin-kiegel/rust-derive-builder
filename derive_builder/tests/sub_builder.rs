#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

use derive_builder::UninitializedFieldError;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
struct BuildError {
    message: String,
}

impl Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.message, f)
    }
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(build_fn(error = "BuildError"))]
struct Lorem {
    #[builder(sub_builder)]
    ipsum: Ipsum,

    #[builder(sub_builder(fn_name = "construct"), field(type = "DolorInput"))]
    dolor: DolorTarget,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(build_fn(error = "BuildError"))]
struct Ipsum {
    i: usize,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(name = "DolorInput", build_fn(name = "construct", error = "BuildError"))]
struct DolorTarget {
    d: String,
}

trait ErrorDisplay {}
impl<E> From<E> for BuildError
where
    E: Display + ErrorDisplay,
{
    fn from(e: E) -> BuildError {
        BuildError {
            message: e.to_string(),
        }
    }
}
impl ErrorDisplay for UninitializedFieldError {}

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
        "Field not initialized: i"
    );
}
