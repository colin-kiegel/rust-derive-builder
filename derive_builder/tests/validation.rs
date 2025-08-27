#[macro_use]
extern crate derive_builder;

#[derive(Debug, Clone, Builder, PartialEq, Eq)]
#[builder(build_fn(validate = "LoremBuilder::validate"))]
pub struct Lorem {
    /// A percentile. Must be between 0 and 100.
    my_effort: u8,

    /// A percentile. Must be less than or equal to `Lorem::my_effort`.
    #[builder(default = 40)]
    their_effort: u8,

    /// A percentile. Must be between 0 and 100.
    rivals_effort: u8,
}

impl LoremBuilder {
    /// Performs bound checks.
    fn validate(&self) -> Result<(), String> {
        if let Some(ref my_effort) = self.my_effort {
            if *my_effort > 100 {
                return Err("Don't wear yourself out".to_string());
            }
        }

        if let Some(ref their_effort) = self.their_effort {
            if *their_effort > 100 {
                return Err("The game has changed".to_string());
            }
        }

        if let Some(ref rivals_effort) = self.rivals_effort {
            if *rivals_effort > 100 {
                return Err("Your rival is cheating".to_string());
            }
        }

        Ok(())
    }
}

#[test]
fn lorem_out_of_bounds() {
    assert_eq!(
        &LoremBuilder::default()
            .my_effort(120)
            .build()
            .unwrap_err()
            .to_string(),
        "Don't wear yourself out"
    );
    assert_eq!(
        &LoremBuilder::default()
            .rivals_effort(120)
            .build()
            .unwrap_err()
            .to_string(),
        "Your rival is cheating"
    );
}

#[test]
fn lorem_validation_pass() {
    let lorem = LoremBuilder::default()
        .my_effort(90)
        .rivals_effort(89)
        .build()
        .expect("All validations should be passing");

    assert_eq!(
        lorem,
        Lorem {
            my_effort: 90,
            rivals_effort: 89,
            their_effort: 40,
        }
    );
}

#[derive(Debug, Builder, PartialEq, Eq)]
#[builder(build_fn(validate = IpsumBuilder::validate, error = BuildIpsumError))]
pub struct Ipsum {
    /// A percentile. Must be between 0 and 100.
    my_effort: u8,

    /// A percentile. Must be less than or equal to `Ipsum::my_effort`.
    #[builder(default = 40)]
    their_effort: u8,

    /// A percentile. Must be between 0 and 100.
    rivals_effort: u8,
}

#[derive(Debug, PartialEq, Eq)]
enum BuildIpsumError {
    UninitializedField(&'static str),
    EffortOutOfRange(&'static str, u8),
}

impl From<derive_builder::UninitializedFieldError> for BuildIpsumError {
    fn from(e: derive_builder::UninitializedFieldError) -> Self {
        BuildIpsumError::UninitializedField(e.field_name())
    }
}

impl IpsumBuilder {
    fn validate(&self) -> Result<(), BuildIpsumError> {
        if let Some(my_effort) = self.my_effort {
            if my_effort > 100 {
                return Err(BuildIpsumError::EffortOutOfRange("User", my_effort));
            }

            if let Some(their_effort) = self.their_effort {
                if their_effort < my_effort {
                    return Err(BuildIpsumError::EffortOutOfRange("Team", their_effort));
                }
            }
        }

        if let Some(their_effort) = self.their_effort {
            if their_effort > 100 {
                return Err(BuildIpsumError::EffortOutOfRange("Team", their_effort));
            }
        }

        if let Some(rivals_effort) = self.rivals_effort {
            if rivals_effort > 100 {
                return Err(BuildIpsumError::EffortOutOfRange("Opponent", rivals_effort));
            }
        }

        Ok(())
    }
}

#[test]
fn ipsum_out_of_bounds() {
    assert_eq!(
        IpsumBuilder::default().my_effort(120).build().unwrap_err(),
        BuildIpsumError::EffortOutOfRange("User", 120)
    );
    assert_eq!(
        IpsumBuilder::default()
            .my_effort(90)
            .their_effort(120)
            .build()
            .unwrap_err(),
        BuildIpsumError::EffortOutOfRange("Team", 120)
    );
    assert_eq!(
        IpsumBuilder::default()
            .rivals_effort(120)
            .build()
            .unwrap_err(),
        BuildIpsumError::EffortOutOfRange("Opponent", 120)
    );
}

#[test]
fn ipsum_validation_pass() {
    let ipsum = IpsumBuilder::default()
        .my_effort(90)
        .their_effort(80)
        .rivals_effort(70)
        .build()
        .expect("All validations should be passing");

    assert_eq!(
        ipsum,
        Ipsum {
            my_effort: 90,
            their_effort: 80,
            rivals_effort: 70,
        }
    );
}
