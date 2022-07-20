#[macro_use]
extern crate derive_builder;

fn validate_age(age: usize) -> Result<(), Error> {
    if age > 200 {
        Err(Error::UnrealisticAge(age))
    } else {
        Ok(())
    }
}

fn check_person(builder: &PersonBuilder) -> Result<(), Error> {
    if let Some(age) = builder.age {
        validate_age(age)
    } else {
        Ok(())
    }
}

#[derive(Builder)]
#[builder(build_fn(
    validate = "check_person",
    post_build = "Self::post_build",
    error = "Error"
))]
struct Person {
    name: String,
    age: i32,
    #[builder(setter(skip), default = "false")]
    person_exists: bool,
}

impl PersonBuilder {
    fn post_build(target: &mut Person) -> Result<(), String> {
        if person.age < 0 {
            return Err("this person does not exist".to_string());
        }
        Ok(())
    }
}

// note: This error does not implement a conversion from PostBuildError, required if a post build operation is configured, which is a
// compile-blocking mistake.
#[derive(Debug)]
enum Error {
    /// A required field is not filled out.
    MissingData(String),
    UnrealisticAge(usize),
}

impl From<UninitializedFieldError> for Error {
    fn from(ufe: UninitializedFieldError) -> OurLoremError {
        Error::MissingData(ufe.to_string())
    }
}

fn main() {}
