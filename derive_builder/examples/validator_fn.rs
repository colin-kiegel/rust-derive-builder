//! This example illustrates the use of `validate` to add a pre-build validation
//! step.

#[macro_use]
extern crate derive_builder;

#[derive(Builder, Debug, PartialEq)]
#[builder(build_fn(validate="LoremBuilder::validate"))]
struct Lorem {
    #[builder(default="42")]
    pub ipsum: u8,
}

impl LoremBuilder {
    /// Check that `Lorem` is putting in the right amount of effort.
    fn validate(&self) -> Result<(), String> {
        if let Some(ref ipsum) = self.ipsum {
            match *ipsum {
                i if i < 20 => Err("Try harder".to_string()),
                i if i > 100 => Err("You'll tire yourself out".to_string()),
                _ => Ok(())
            }
        } else {
            Ok(())
        }
    }
}

fn main() {
    // If we don't set the field `ipsum`,
    let x = LoremBuilder::default().build().unwrap();

    // .. the custom default will be used for `ipsum`:
    assert_eq!(x, Lorem {
        ipsum: 42,
    });
}