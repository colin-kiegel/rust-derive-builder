#[cfg(feature = "std")]
use std::{error::Error, fmt};

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(not(feature = "std"))]
use export::core::String;

/// Runtime error when a `build()` method is called and one or more required fields
/// do not have a value.
#[derive(Debug, Clone)]
pub struct UninitializedFieldError(&'static str);

impl UninitializedFieldError {
    /// Create a new `UnitializedFieldError` for the specified field name.
    pub fn new(field_name: &'static str) -> Self {
        UninitializedFieldError(field_name)
    }

    /// Get the name of the first-declared field that wasn't initialized
    pub fn field_name(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for UninitializedFieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Field not initialized: {}", self.0)
    }
}

#[cfg(feature = "std")]
impl Error for UninitializedFieldError {}

impl From<&'static str> for UninitializedFieldError {
    fn from(field_name: &'static str) -> Self {
        Self::new(field_name)
    }
}

#[derive(Debug, Clone)]
pub struct PostBuildError(String);

impl PostBuildError {
    /// Create a new `UnitializedFieldError` for the specified field name.
    pub fn new(msg: String) -> Self {
        PostBuildError(msg)
    }

    /// Get the name of the first-declared field that wasn't initialized
    #[allow(dead_code)]
    pub fn get_msg(self) -> String {
        self.0
    }
}

impl fmt::Display for PostBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "post build error: {}", self.0)
    }
}

#[cfg(feature = "std")]
impl Error for PostBuildError {}

impl From<&'static str> for PostBuildError {
    fn from(msg: &'static str) -> Self {
        Self::new(msg.to_string())
    }
}
