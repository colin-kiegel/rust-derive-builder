use derive_builder::UninitializedFieldError;

#[macro_use]
extern crate derive_builder;

/// Marker function that tells `derive_builder` to emit a `ValidationError` variant.
///
/// Without this, `derive_builder` would believe that only uninitialized field errors
/// are possible in the `build()` method, and there would be a cryptic error message
/// about a missing conversion from `String`.
fn phantom_validate<T>(_: T) -> Result<(), String> {
    Ok(())
}

#[derive(Builder, PartialEq, Debug)]
#[builder(build_fn(validate = "phantom_validate"))]
struct Lorem {
    ipsum: String,
    #[builder(default = "self.default_dolor()?")]
    dolor: String,
    #[builder(default = "self.default_sit()?")]
    sit: String,
    #[builder(setter(skip), default = "self.default_amet()")]
    amet: String,
}

impl LoremBuilder {
    fn default_dolor(&self) -> Result<String, UninitializedFieldError> {
        self.ipsum
            .clone()
            .ok_or_else(|| UninitializedFieldError::new("ipsum"))
    }

    fn default_sit(&self) -> Result<String, String> {
        match self.ipsum {
            Some(ref x) if x.chars().count() > 3 => Ok(format!("sit {}", x)),
            _ => Err("ipsum must at least 3 chars to build sit".to_string()),
        }
    }

    fn default_amet(&self) -> String {
        if let Some(ref x) = self.ipsum {
            format!("amet {}", x)
        } else {
            "..nothing there".to_string()
        }
    }
}

fn main() {
    let x = LoremBuilder::default()
        .ipsum("ipsum".to_string())
        .build()
        .unwrap();

    assert_eq!(
        x,
        Lorem {
            ipsum: "ipsum".to_string(),
            dolor: "ipsum".to_string(),
            sit: "sit ipsum".to_string(),
            amet: "amet ipsum".to_string(),
        }
    );
}
