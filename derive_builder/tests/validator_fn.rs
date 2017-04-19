#[macro_use]
extern crate derive_builder;

#[derive(Debug, Clone, Builder, PartialEq, Eq)]
#[builder(build_fn(validator="LoremBuilder::validate"))]
pub struct Lorem {
    /// A percentile. Must be between 0 and 100.
    my_effort: u8,
    
    /// A percentile. Must be less than or equal to `Lorem::my_effort`.
    #[builder(default="40")]
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
fn incomplete_fields() {
    let err_msg = LoremBuilder::default().build().unwrap_err();
    
    // In this case, the validator should have run but passed because there was no value set for `my_effort`.
    // Adding private `get` methods which return `Result<T, String>` by pulling in defaults will enable the
    // validation function to be improved here.
    assert!(&err_msg != "Don't wear yourself out");
}

#[test]
fn out_of_bounds() {
    assert_eq!("Don't wear yourself out", &LoremBuilder::default().my_effort(120).build().unwrap_err());
    assert_eq!("Your rival is cheating", &LoremBuilder::default().rivals_effort(120).build().unwrap_err());
}

#[test]
fn validation_pass() {
    let lorem = LoremBuilder::default()
        .my_effort(90)
        .rivals_effort(89)
        .build()
        .expect("All validations should be passing");
        
    assert_eq!(lorem, Lorem {
        my_effort: 90,
        rivals_effort: 89,
        their_effort: 40,
    });
}