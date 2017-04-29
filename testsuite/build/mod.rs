#[cfg(not(feature = "skeptic_tests"))]
fn main() {}

#[cfg(feature = "skeptic_tests")]
include!("skeptic.rs");
