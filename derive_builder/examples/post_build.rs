// some times, one or more fields of your struct need to be built according to other field's values
// may be you have a layout which needs tpo know all other field's values to be built, or that you have some field that needs to be calculated only once for the struct
// and you don't want to recalculate it every time one of the fields are set in the builder
// There are several situations in which a post build event might be useful.
// 1. You might have a field which needs to be calculated when the struct is built, but deppends on several other field values to calculate.
// 2. You might want to trigger an event whenever a new struct of a given type is built, so that other components of your application can react.
// 3. You might want to perform a complete validation which needs to know values of all fields to be performed.
// The classical ways to solve this problem would be:
// 1. call a method wich accepts &mut self in the target struct as soon as it is built. This method would make validations, trigger events, calculate fields.
// The main issue with this strategy is one can easily forget to call the post build method or function, and even if they don't, this function is effectively finishing to build the struct, so it shouldn't be decoupled from the building process at all. This post build function or  method also shouldn't be part of the public api.
// 2. Customize the setters of all fields affecting the field to be calculated and update it accordingly.
// The main issue with this strategy is that This would make room for repeated code, might run an expensive calculation on all calls to affected setters and would couple the set of the calculated field with the setters of all the affected fields, all things we want to avoid.
// The post_build parameter of build_fn allows you to provide a function or a method which gets called by the build() method of the builder struct as soon as the target struct is built.
// this solves the proposed problem, creates great ergonomy and decouples the set of the calculated field and all other side effects you might want to add with the setters of other fields
// In order to use post_build functionality, you can declare #[builder(build_fn(post_build = "path::to::fn"))] to specify a post build function which will be called as soon as the target struct is built, still inside the builder function. The path does not need to be fully-qualified, and will consider use statements made at module level. It must be accessible from the scope where the target struct is declared.
// The provided function must have the signature (&mut Foo) -> Result<_, String>; the Ok variant is not used by the build method.

use derive_builder::{PostBuildError, UninitializedFieldError};

#[macro_use]
extern crate derive_builder;

#[derive(Debug, Clone, Builder, PartialEq, Eq)]
#[builder(build_fn(post_build = "LoremBuilder::post_build"))]
pub struct Lorem {
    /// a number
    number: i32,

    /// big_number
    /// this will be calculated by the post build function, so it should not (though mothing prevents it from) make a setter available.
    /// It also should have a default value, either specified in the builder directive or delegated to the Default trait of the type
    #[builder(setter(skip), default = "false")]
    big_number: bool,
}

impl LoremBuilder {
    /// performs post build operation
    fn post_build(target: &mut Lorem) -> Result<(), String> {
        // post build validation
        if target.number <= 0 {
            return Err("Number must be greater than 0".to_string());
        }
        // remember that we didn't set the big_number field. We deppend on the number field to decide if this instance has or hasn't a big number.
        // This can only be safely known when the struct has just been built but nobody for sure yet used it
        if target.number > 60 {
            // initialize the big_number field
            target.big_number = true;
        }

        Ok(())
    }
}

// if you are using post build functionality, you need to provide a conversion  from PostBuildError for your custom error type
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

fn main() {
    // post build does not modify the big_number default field value
    let x = LoremBuilder::default().number(20).build().unwrap();
    assert_eq!(x.big_number, false);

    // post build modifies the big_number field value
    let x = LoremBuilder::default().number(80).build().unwrap();
    assert_eq!(x.big_number, true);

    // post build validation fails
    let x = LoremBuilder::default().number(-1).build().unwrap_err();
    let correct_variant = match x {
        LoremBuilderError::UninitializedField(_) => {
            panic!("Should get a post builder error, got a uninitialized field error instead")
        }
        LoremBuilderError::PostBuildError(e) => true,
        LoremBuilderError::ValidationError(_) => {
            panic!("Should get a post builder error, got a validation error instead")
        }
    };
    assert!(correct_variant);

    // should get a custom error instance when using post build with a custom error
    let x = LoremCustomErrorBuilder::default().number(-1).build();

    assert!(x.is_err());
}
