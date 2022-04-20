#[cfg(feature = "std")]
use std::{error::Error, fmt};

#[cfg(not(feature = "std"))]
use core::fmt;

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

/// Runtime error used when a sub-field's `build` method failed.
///
/// Represents an error from a sub-structure's builder, when
/// [`#[builder(sub_builder)]`](../index.html#sub-fields-with-builders-nested-builders)
/// is used.
/// Contains an `E`, the error type returned by the sub-structure's builder.
/// See
/// [Errors from the sub-structure builder](../index.html#errors-from-the-sub-structure-builder).
#[derive(Debug, Clone)]
pub struct SubfieldBuildError<E>(&'static str, E);

impl<E> SubfieldBuildError<E> {
    /// Wrap an error in a `SubfieldBuildError`, attaching the specified field name.
    pub fn new(field_name: &'static str, sub_builder_error: E) -> Self {
        SubfieldBuildError(field_name, sub_builder_error)
    }

    /// Get the field name of the sub-field that couldn't be built.
    pub fn field_name(&self) -> &'static str {
        self.0
    }

    /// Get the error that was returned for the sub-field
    pub fn sub_builder_error(&self) -> &E {
        &self.1
    }

    /// Decompose the `SubfieldBuildError` into its constituent parts
    pub fn into_parts(self) -> (&'static str, E) {
        (self.0, self.1)
    }
}

impl<E> fmt::Display for SubfieldBuildError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "in {}: {}", self.0, self.1)
    }
}

#[cfg(feature = "std")]
// We do not implement `cause`.
// Rust EHWG recommend that we *either* provide a cause, *or* include the information in our
// own message.  We really want to do the latter or users without a proper error reporter
// will get a very nugatory message.
impl<E> Error for SubfieldBuildError<E> where E: Error {}
