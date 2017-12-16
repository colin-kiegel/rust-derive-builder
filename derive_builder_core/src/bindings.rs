use RawTokens;

/// Bindings to be used by the generated code.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bindings {
    /// Whether the generated code should comply with `#![no_std]`.
    pub no_std: bool,
}

impl Bindings {
    /// String type.
    pub fn string_ty(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: alloc :: string :: String"
        } else {
            ":: std :: string :: String"
        })
    }

    /// Result type.
    pub fn result_ty(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: result :: Result"
        } else {
            ":: std :: result :: Result"
        })
    }

    /// Option type.
    pub fn option_ty(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: option :: Option"
        } else {
            ":: std :: option :: Option"
        })
    }

    /// PhantomData type.
    pub fn phantom_data_ty(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: marker :: PhantomData"
        } else {
            ":: std :: marker :: PhantomData"
        })
    }

    /// Default trait.
    pub fn default_trait(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: default :: Default"
        } else {
            ":: std :: default :: Default"
        })
    }

    /// Clone trait.
    pub fn clone_trait(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: clone :: Clone"
        } else {
            ":: std :: clone :: Clone"
        })
    }

    /// Into trait.
    #[cfg_attr(feature = "cargo-clippy", allow(wrong_self_convention))]
    pub fn into_trait(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: convert :: Into"
        } else {
            ":: std :: convert :: Into"
        })
    }

    /// TryInto trait.
    pub fn try_into_trait(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
            ":: core :: convert :: TryInto"
        } else {
            ":: std :: convert :: TryInto"
        })
    }
}

#[test]
fn std() {
    let b = Bindings { no_std: false };

    assert_eq!(b.string_ty().to_tokens(), quote!(::std::string::String));

    assert_eq!(b.result_ty().to_tokens(), quote!(::std::result::Result));

    assert_eq!(b.option_ty().to_tokens(), quote!(::std::option::Option));

    assert_eq!(
        b.phantom_data_ty().to_tokens(),
        quote!(::std::marker::PhantomData)
    );

    assert_eq!(
        b.default_trait().to_tokens(),
        quote!(::std::default::Default)
    );

    assert_eq!(b.clone_trait().to_tokens(), quote!(::std::clone::Clone));

    assert_eq!(b.into_trait().to_tokens(), quote!(::std::convert::Into));
}

#[test]
fn no_std() {
    let b = Bindings { no_std: true };

    assert_eq!(
        b.string_ty().to_tokens(),
        quote!(::alloc::string::String)
    );

    assert_eq!(b.result_ty().to_tokens(), quote!(::core::result::Result));

    assert_eq!(b.option_ty().to_tokens(), quote!(::core::option::Option));

    assert_eq!(
        b.phantom_data_ty().to_tokens(),
        quote!(::core::marker::PhantomData)
    );

    assert_eq!(
        b.default_trait().to_tokens(),
        quote!(::core::default::Default)
    );

    assert_eq!(b.clone_trait().to_tokens(), quote!(::core::clone::Clone));

    assert_eq!(b.into_trait().to_tokens(), quote!(::core::convert::Into));
}
