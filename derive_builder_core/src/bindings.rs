use syn::{self, Path};

/// Bindings to be used by the generated code.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bindings {
    /// Whether the generated code should comply with `#![no_std]`.
    pub no_std: bool,
}

impl Bindings {
    /// String type.
    pub fn string_ty(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::alloc::string::String"
        } else {
            "::std::string::String"
        })
        .unwrap()
    }

    /// Result type.
    pub fn result_ty(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::result::Result"
        } else {
            "::std::result::Result"
        })
        .unwrap()
    }

    /// Option type.
    pub fn option_ty(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::option::Option"
        } else {
            "::std::option::Option"
        })
        .unwrap()
    }

    /// PhantomData type.
    pub fn phantom_data_ty(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::marker::PhantomData"
        } else {
            "::std::marker::PhantomData"
        })
        .unwrap()
    }

    /// Default trait.
    pub fn default_trait(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::default::Default"
        } else {
            "::std::default::Default"
        })
        .unwrap()
    }

    /// Clone trait.
    pub fn clone_trait(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::clone::Clone"
        } else {
            "::std::clone::Clone"
        })
        .unwrap()
    }

    /// Into trait.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_trait(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::convert::Into"
        } else {
            "::std::convert::Into"
        })
        .unwrap()
    }

    /// TryInto trait.
    pub fn try_into_trait(&self) -> Path {
        syn::parse_str(if self.no_std {
            "::core::convert::TryInto"
        } else {
            "::std::convert::TryInto"
        })
        .unwrap()
    }
}
