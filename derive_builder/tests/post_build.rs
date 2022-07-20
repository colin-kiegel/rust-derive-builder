use derive_builder::{PostBuildError, UninitializedFieldError};

#[macro_use]
extern crate derive_builder;

#[derive(Debug, Clone, Builder, PartialEq, Eq)]
#[builder(build_fn(post_build = "LoremBuilder::post_build"))]
pub struct Lorem {
    /// a number
    number: i32,

    /// big_number
    #[builder(setter(skip), default = "false")]
    big_number: bool,
}

impl LoremBuilder {
    /// performs post build operation
    fn post_build(target: &mut Lorem) -> Result<(), String> {
        if target.number <= 0 {
            return Err("Number must be greater than 0".to_string());
        }
        if target.number > 60 {
            target.big_number = true;
        }

        Ok(())
    }
}

#[derive(Builder, Debug, PartialEq)]
#[builder(build_fn(
    post_build = "LoremCustomErrorBuilder::post_build",
    error = "OurLoremError"
))]
struct LoremCustomError {
    /// a number
    number: i32,

    /// big_number
    #[builder(setter(skip), default = "false")]
    big_number: bool,
}

impl LoremCustomErrorBuilder {
    /// performs post build operation
    fn post_build(target: &mut LoremCustomError) -> Result<(), String> {
        if target.number <= 0 {
            return Err("Number must be greater than 0".to_string());
        }
        if target.number > 60 {
            target.big_number = true;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct OurLoremError(String);

impl From<UninitializedFieldError> for OurLoremError {
    fn from(ufe: UninitializedFieldError) -> OurLoremError {
        OurLoremError(ufe.to_string())
    }
}

impl From<PostBuildError> for OurLoremError {
    fn from(pbe: PostBuildError) -> OurLoremError {
        OurLoremError(format!("Custom build error: {}", pbe.get_msg()))
    }
}

#[test]
#[should_panic(expected = "Number must be greater than 0")]
fn post_build_generates_error() {
    LoremBuilder::default().number(-1).build().unwrap();
}

#[test]
fn post_build_runs_without_modifying_built_struct() {
    let x = LoremBuilder::default().number(20).build().unwrap();
    assert_eq!(x.big_number, false);
}

#[test]
fn post_build_runs_and_modifies_built_struct() {
    let x = LoremBuilder::default().number(80).build().unwrap();
    assert_eq!(x.big_number, true);
}

#[test]
#[should_panic(expected = "Custom build error: Number must be greater than 0")]
fn post_build_generates_error_using_custom_error() {
    LoremCustomErrorBuilder::default()
        .number(-1)
        .build()
        .unwrap();
}
